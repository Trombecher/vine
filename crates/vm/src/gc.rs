use std::cell::{Cell, UnsafeCell};
use std::collections::HashSet;
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::num::NonZeroU8;
use std::ops::{Deref};
use std::ptr::slice_from_raw_parts_mut;
use crate::{Object, Value};

/// This is the first (and last) reference the object will ever have.
///
/// It owns the object, but permits immutable and mutable borrows
/// as the object is never accessed though it.
///
/// This reference does not prevent deallocation (is not a strong reference).
/// Deallocation destroys this reference.
#[derive(Eq, PartialEq, Hash)]
struct SourceRef {
    inner: *const Object,
}

impl Deref for SourceRef {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl Drop for SourceRef {
    fn drop(&mut self) {
        let len = unsafe { self.inner.as_ref_unchecked() }.cached_size.get().get() as usize + 1;

        let _ = unsafe {
            Box::<[Value]>::from_raw(slice_from_raw_parts_mut(
                self.inner as usize as *mut Value,
                len
            ))
        };

        // box gets dropped
    }
}

pub struct GC<'types> {
    types: &'types [u8],
    allocated_objects: UnsafeCell<HashSet<SourceRef>>,
}

impl<'types> GC<'types> {
    #[inline]
    pub fn new(types: &'types [u8]) -> Self {
        Self {
            types,
            allocated_objects: UnsafeCell::new(HashSet::new()),
        }
    }
    
    /// Returns the number of objects currently allocated.
    #[inline]
    pub fn count(&self) -> usize {
        unsafe { (&*self.allocated_objects.get()).len() }
    }
    
    #[inline]
    pub fn upgrade(&self, weak_ref: *const Object) -> Option<&Object> {
        unsafe {
            if (&*self.allocated_objects.get()).contains(transmute(weak_ref)) {
                Some(transmute(weak_ref))
            } else {
                None
            }
        }
    }

    pub fn allocate(&self, type_index: u32) -> &Object {
        let size: NonZeroU8 = (*self.types.get(type_index as usize).expect("out of bounds"))
            .try_into()
            .expect("zst");

        let object = Box::leak(Box::<[Value]>::new_uninit_slice(size.get() as usize + 1));

        unsafe {
            transmute::<_, &mut MaybeUninit<Object>>(object.as_ptr()).write(Object {
                is_locked: Cell::new(false),
                has_strong_refs: Cell::new(false),
                cached_size: Cell::new(size),
                ty: Cell::new(0),
                data: UnsafeCell::new(PhantomData),
            });
        }

        // Fill values with zeroes
        for i in 1..size.get() + 1 {
            unsafe {
                object.get_unchecked_mut(i as usize).write(0_u64.into());
            }
        }

        unsafe {
            (&mut *self.allocated_objects.get()).insert(SourceRef {
                inner: object.as_ptr() as usize as *const Object,
            });
        }

        unsafe {
            &*(object.as_ptr() as usize as *const Object)
        }
    }
    
    /// Deallocates all unused objects.
    ///
    /// # Safety
    ///
    /// Assumes all `is_used` fields of [Object] are `false`.
    pub fn mark_and_sweep<'heap>(&self, roots: impl Iterator<Item = Value<'heap>>) {
        // Mark
        mark(roots);

        // Sweep
        unsafe { &mut *self.allocated_objects.get() }.retain(|object_ref| {
            if object_ref.has_strong_refs.get() {
                object_ref.has_strong_refs.set(false); // reset for next GC
                true
            } else {
                false
            }
        });
    }
}

fn mark<'heap>(roots: impl Iterator<Item = Value<'heap>>) {
    for root in roots.filter_map(Value::get_object)  {
        let lock = if let Some(lock) = root.try_lock() {
            lock
        } else {
            continue
        };

        root.has_strong_refs.set(true);
        mark(lock.iter().copied())
    }
}