mod tests;

use std::hint::unreachable_unchecked;
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::slice_from_raw_parts;

#[derive(Debug, PartialEq)]
pub enum Error {
    UTF8InvalidFirstByte,
    UTF8Expected2Of2,
    UTF8Invalid2Of2,
    UTF8Expected2Of3,
    UTF8Invalid2Of3,
    UTF8Expected3Of3,
    UTF8Invalid3Of3,
    UTF8Expected2Of4,
    UTF8Invalid2Of4,
    UTF8Expected3Of4,
    UTF8Invalid3Of4,
    UTF8Expected4Of4,
    UTF8Invalid4Of4,
}

pub struct Cursor<'a> {
    /// The pointer to the first element.
    first: *const u8,
    
    /// The pointer to the next element.
    cursor: *const u8,
    
    /// The pointer to the past-the-end element.
    end: *const u8,
    
    /// The number of chars consumed.
    index: u64,
    
    _marker: PhantomData<&'a [u8]>,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub fn new(slice: &[u8]) -> Self {
        Self {
            first: slice.as_ptr(),
            cursor: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            index: 0,
            _marker: Default::default(),
        }
    }
    
    /// Gets the next byte. Normalizes line terminators by mapping to a line feed.
    #[inline]
    pub fn next(&mut self) -> Option<u8> {
        match self.peek_raw() {
            None => None,
            Some(b'\r') => {
                unsafe { self.advance_unchecked() }
                
                if self.peek_raw() == Some(b'\n') {
                    // SAFETY: Because of `Some(...)` there is a next byte.
                    self.cursor = unsafe { self.cursor.add(1) };
                }
                Some(b'\n')
            }
            x => {
                unsafe { self.advance_unchecked() }
                x
            }
        }
    }
    
    /// Gets the next byte. Does not normalize line terminators.
    #[inline]
    pub fn next_raw(&mut self) -> Option<u8> {
        if self.cursor == self.end {
            None
        } else {
            let byte = unsafe { *self.cursor };
            unsafe { self.advance_unchecked() };
            Some(byte)
        }
    }

    /// Peeks the next byte. Does not advance the iterator.
    #[inline]
    pub fn peek_raw(&self) -> Option<u8> {
        if self.cursor == self.end {
            None
        } else {
            Some(unsafe { *self.cursor })
        }
    }

    #[inline]
    pub fn rewind_ascii(&mut self) {
        if self.cursor != self.first {
            self.index -= 1;
            self.cursor = unsafe { self.cursor.sub(1) };
            
            if unsafe { *self.cursor } == b'\n'
                && self.cursor != self.first
                && unsafe { *self.cursor.sub(1) } == b'\r' {
                self.cursor = unsafe { self.cursor.sub(1) };
            }
        }
    }
    
    /// Rewinds a byte.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the previous byte was a non line-feed ascii character.
    #[inline]
    pub fn rewind_u8(&mut self) {
        if self.cursor != self.first {
            self.index -= 1;
            self.cursor = unsafe { self.cursor.sub(1) };
        }
    }

    /// Advances the cursor one byte.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor is not at the end.
    #[inline]
    pub unsafe fn advance_unchecked(&mut self) {
        self.index += 1;
        self.cursor = self.cursor.add(1)
    }

    #[inline]
    pub fn advance_char(&mut self) -> Result<(), crate::Error> {
        self.index += 1;
        
        let first_byte = match self.next_raw() {
            Some(x) => x,
            None => return Ok(()),
        };

        macro_rules! next {
            ($e:expr,$i:expr) => {
                match self.next() {
                    None => return Err(crate::Error::Chars($e)),
                    Some(x) if x & 0b1100_0000 != 0b1000_0000 => return Err(crate::Error::Chars($i)),
                    _ => {},
                }
            };
        }

        match UTF8_CHAR_WIDTH[first_byte as usize] {
            0 => Err(crate::Error::Chars(Error::UTF8InvalidFirstByte)),
            1 => {
                if first_byte == b'\r' && self.peek_raw() == Some(b'\n')  {
                    unsafe { self.advance_unchecked() }
                }
                Ok(())
            },
            2 => {
                next!(Error::UTF8Expected2Of2, Error::UTF8Invalid2Of2);
                Ok(())
            }
            3 => {
                next!(Error::UTF8Expected2Of3, Error::UTF8Invalid2Of3);
                next!(Error::UTF8Expected3Of3, Error::UTF8Invalid3Of3);
                Ok(())
            }
            4 => {
                next!(Error::UTF8Expected2Of4, Error::UTF8Invalid2Of4);
                next!(Error::UTF8Expected3Of4, Error::UTF8Invalid3Of4);
                next!(Error::UTF8Expected4Of4, Error::UTF8Invalid4Of4);
                Ok(())
            }
            _ => unsafe { unreachable_unchecked() }
        }
    }

    #[inline]
    pub fn begin_recording<'c>(&'c mut self) -> Recorder<'a, 'c> {
        Recorder {
            start: self.cursor,
            cursor: self,
        }
    }
    
    #[inline]
    pub fn index(&mut self) -> u64 {
        self.index
    }
}

pub struct Recorder<'a, 'c> {
    pub cursor: &'c mut Cursor<'a>,
    start: *const u8,
}

impl<'a, 'c> Recorder<'a, 'c> {
    #[inline]
    pub fn stop(self) -> &'a str {
        unsafe { transmute(slice_from_raw_parts(
            self.start,
            self.start.offset_from(self.cursor.cursor).unsigned_abs()
        )) }
    }
}

const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];