use std::cell::{Cell, UnsafeCell};
use std::fmt::{Debug, Formatter};
use std::intrinsics::transmute;
use std::marker::PhantomData;
use std::num::NonZeroU8;
use std::ops::{Deref, DerefMut};
use std::slice::from_raw_parts;
use crate::{Value, GC};

#[repr(align(8), C)]
pub struct Object {
    pub(crate) is_locked: Cell<bool>,
    pub has_strong_refs: Cell<bool>,
    pub cached_size: Cell<NonZeroU8>,
    pub ty: Cell<u32>,
    
    /// The start of the data which is `[Value<'?>]` (length [Object::size]);
    pub(crate) data: UnsafeCell<PhantomData<()>>,
}

impl Object {
    #[inline]
    pub fn size(&self) -> usize {
        self.cached_size.get().get() as usize
    }

    pub fn lock(&self) -> DataView {
        self.try_lock().expect("Double lock!")
    }

    #[inline]
    pub fn try_lock(&self) -> Option<DataView> {
        if self.is_locked.get() {
            None
        } else {
            self.is_locked.set(true);

            Some(DataView {
                slice: unsafe {
                    transmute(from_raw_parts(transmute::<_, *const Value>(&self.data), self.size()))
                },
            })
        }
    }
}

/// A guard of an exclusive view of the value of an object.
///
/// Dereferences to a slice; the lock on [StrongRef] is released when this drops.
pub struct DataView<'heap> {
    slice: &'heap UnsafeCell<[Value<'heap>]>,
}

impl<'heap> DataView<'heap> {
    pub fn display<'input: 'heap>(self, gc: &'heap GC<'input>) -> DataViewDisplay<'input, 'heap> {
        DataViewDisplay {
            view: self,
            gc,
        }
    }
}

impl<'heap> Deref for DataView<'heap> {
    type Target = [Value<'heap>];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.slice.get() }
    }
}

impl<'heap> DerefMut for DataView<'heap> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.slice.get() }
    }
}

impl<'heap> Drop for DataView<'heap> {
    fn drop(&mut self) {
        // Magic :) (unlocks lock)
        unsafe {
            &*((&*self.slice.get()).as_ptr().sub(1) as usize as *const Cell<bool>)
        }.set(false);
    }
}

pub struct DataViewDisplay<'types: 'heap, 'heap> {
    view: DataView<'heap>,
    gc: &'heap GC<'types>
}

impl<'types: 'heap, 'heap> Debug for DataViewDisplay<'types, 'heap> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, value) in self.view.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            value.display(self.gc).fmt(f)?;
        }
        
        Ok(())
    }
}