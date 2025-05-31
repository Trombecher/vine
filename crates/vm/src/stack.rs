use crate::{Error, Value};
use std::intrinsics::transmute;
use std::mem::MaybeUninit;
use std::slice::from_raw_parts;

pub struct Stack<'heap, const MAX_SIZE: usize> {
    size: usize,
    buffer: [MaybeUninit<Value<'heap>>; MAX_SIZE],
}

impl<'heap, const MAX_SIZE: usize> Stack<'heap, MAX_SIZE> {
    #[inline]
    pub fn new() -> Self {
        Self {
            size: 0,
            buffer: unsafe {
                MaybeUninit::<[MaybeUninit<Value<'heap>>; MAX_SIZE]>::uninit().assume_init()
            },
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Value<'heap>] {
        unsafe { from_raw_parts(transmute(&self.buffer), self.size) }
    }

    #[inline]
    pub fn push(&mut self, value: Value<'heap>) -> Result<(), Error> {
        self.preallocate(1)?[0].write(value);
        Ok(())
    }

    /// Allocates a slot on the stack.
    #[inline]
    pub fn preallocate(&mut self, n: usize) -> Result<&mut [MaybeUninit<Value<'heap>>], Error> {
        if self.size + n < self.buffer.len() {
            let ptr = &mut self.buffer[self.size..self.size + n];
            self.size += n;
            Ok(ptr)
        } else {
            Err(Error::StackOverflow)
        }
    }

    #[inline]
    pub fn top(&mut self) -> Option<&Value<'heap>> {
        if self.size == 0 {
            None
        } else {
            Some(unsafe { self.buffer[self.size - 1].assume_init_ref() })
        }
    }

    #[inline]
    pub fn top_mut(&mut self) -> Option<&mut Value<'heap>> {
        if self.size == 0 {
            None
        } else {
            Some(unsafe { self.buffer[self.size - 1].assume_init_mut() })
        }
    }

    #[inline]
    pub fn top_offset_ptr(&mut self, n: usize) -> Option<*const Value<'heap>> {
        if self.size > n {
            Some(unsafe { self.buffer[self.size - n - 1].assume_init_ref() } as *const Value<'heap>)
        } else {
            None
        }
    }

    #[inline]
    pub fn top_offset_ptr_mut(&mut self, n: usize) -> Option<*mut Value<'heap>> {
        if self.size > n {
            Some(unsafe { self.buffer[self.size - n - 1].assume_init_mut() } as *mut Value<'heap>)
        } else {
            None
        }
    }

    #[inline]
    pub fn pop(&mut self) {
        self.size = self.size.saturating_sub(1);
    }

    #[inline]
    pub fn pop_get(&mut self) -> Option<Value<'heap>> {
        if self.size == 0 {
            return None;
        }

        self.size -= 1;

        Some(unsafe { self.buffer[self.size].assume_init_ref().clone() })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}
