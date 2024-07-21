use std::intrinsics::transmute;
use std::mem::MaybeUninit;
use std::slice::from_raw_parts;
use crate::{Error, Value};

pub struct Stack<const MAX_SIZE: usize> {
    size: usize,
    buffer: [MaybeUninit<Value>; MAX_SIZE],
}

impl<const MAX_SIZE: usize> Stack<MAX_SIZE> {
    #[inline]
    pub fn new() -> Self {
        Self {
            size: 0,
            buffer: unsafe { MaybeUninit::<[MaybeUninit<Value>; MAX_SIZE]>::uninit().assume_init() }
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Value] {
        unsafe { from_raw_parts(transmute(&self.buffer), self.size) }
    }
    
    #[inline]
    pub fn push(&mut self, value: Value) -> Result<(), Error> {
        self.preallocate(1)?[0].write(value);
        Ok(())
    }
    
    /// Allocates a slot on the stack.
    #[inline]
    pub fn preallocate(&mut self, n: usize) -> Result<&mut [MaybeUninit<Value>], Error> {
        if self.size + n < self.buffer.len() {
            let ptr = &mut self.buffer[self.size..self.size + n];
            self.size += n;
            Ok(ptr)
        } else {
            Err(Error::StackOverflow)
        }
    }

    #[inline]
    pub fn top(&mut self) -> Option<&Value> {
        if self.size == 0 {
            None
        } else {
            Some(unsafe {
                self.buffer[self.size - 1].assume_init_ref()
            })
        }
    }

    #[inline]
    pub fn top_mut(&mut self) -> Option<&mut Value> {
        if self.size == 0 {
            None
        } else {
            Some(unsafe {
                self.buffer[self.size - 1].assume_init_mut()
            })
        }
    }
    
    #[inline]
    pub fn top_offset_ptr(&mut self, n: usize) -> Option<*const Value> {
        if self.size > n {
            Some(unsafe {
                self.buffer[self.size - n - 1].assume_init_ref()
            } as *const Value)
        } else {
            None
        }
    }

    #[inline]
    pub fn top_offset_ptr_mut(&mut self, n: usize) -> Option<*mut Value> {
        if self.size > n {
            Some(unsafe {
                self.buffer[self.size - n - 1].assume_init_mut()
            } as *mut Value)
        } else {
            None
        }
    }
    
    #[inline]
    pub fn pop(&mut self) {
        self.size = self.size.saturating_sub(1);
    }
    
    #[inline]
    pub fn pop_get(&mut self) -> Option<Value> {
        if self.size == 0 {
            return None
        }
        
        self.size -= 1;
        
        Some(unsafe {
            self.buffer[self.size].assume_init_ref().clone()
        })
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}