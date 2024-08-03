#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]
#![feature(ptr_sub_ptr)]
#![no_std]

use core::mem::MaybeUninit;

/// Returns the index in the underlying buffer for a given logical element index.
#[inline]
fn wrap_index<const CAPACITY: usize>(logical_index: usize) -> usize {
    debug_assert!(
        (logical_index == 0 && CAPACITY == 0)
            || logical_index < CAPACITY
            || (logical_index - CAPACITY) < CAPACITY
    );

    if logical_index >= CAPACITY {
        logical_index - CAPACITY
    } else {
        logical_index
    }
}

pub struct Queue<T, const CAPACITY: usize> {
    buffer: [MaybeUninit<T>; CAPACITY],
    head: usize,
    len: usize
}

impl<T, const CAPACITY: usize> Queue<T, CAPACITY> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: 0,
            len: 0,
            buffer: MaybeUninit::uninit_array(),
        }
    }

    /// Returns `true` if the buffer is at full capacity; otherwise `false`.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    #[inline]
    const fn wrap_add(&self, index: usize, addend: usize) -> usize {
        wrap_index::<{ CAPACITY }>(index.wrapping_add(addend))
    }

    #[inline]
    const fn to_physical_index(&self, index: usize) -> usize {
        self.wrap_add(self.head, index)
    }

    #[inline]
    pub const fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe {
                self.buffer.as_ptr().add(self.to_physical_index(index)).assume_init_ref()
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            Some(unsafe {
                self.buffer.as_ptr().add(self.to_physical_index(index)).assume_init_mut()
            })
        } else {
            None
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        todo!()
    }

    pub fn truncate(&mut self, len: usize) {
        todo!()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    pub fn contains(&mut self) {
        todo!()
    }

    /// Returns the capacity.
    pub const fn capacity() -> usize {
        CAPACITY
    }

    pub fn pop_front(&mut self) -> Option<T> {
        todo!()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        todo!()
    }

    pub fn push_front(&mut self, value: T) {
        todo!()
    }
    
    /// Tries to prepend a value to the front of the queue.
    ///
    /// Returns [Some] if the push succeeded, [None] otherwise.
    pub fn try_push_front(&mut self, value: T) -> Option<()> {
        todo!()
    }
    
    pub fn force_push_front(&mut self, value: T) -> Option<()> {
        todo!()
    }

    #[inline]
    pub fn push_back(&mut self, value: T) {
        self.try_push_back(value).expect("queue is full")
    }

    #[inline]
    pub fn try_push_back(&mut self, value: T) -> Option<()> {
        if self.len >= CAPACITY {
            None
        } else {
            unsafe {
                self.start.add(self.len).write(value);
            }

            self.len += 1;
            Some(())
        }
    }

    #[inline]
    pub fn force_push_back(&mut self, value: T) {
        if self.len >= CAPACITY {
            // The queue is full, therefore the start element contains an initialized value.
            // We drop the value and write the new value.
            unsafe {
                *self.start.assume_init_mut() = value;
                self.advance();
            }
        } else {
            unsafe {
                self.start.add(self.len).write(value);
            }

            self.len += 1;
        }
    }
    
    /// Returns an optional reference to the first value.
    #[inline]
    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }
    
    /// Returns an optional reference to the last value.
    #[inline]
    pub fn back(&self) -> Option<&T> {
        self.get(self.len.wrapping_sub(1))
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len.wrapping_sub(1))
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        todo!()
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = &mut T> {
        todo!(9)
    }
}