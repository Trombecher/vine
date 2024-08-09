mod tests;

use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

/// An iterator over a slice.
pub struct Cursor<'a> {
    /// The pointer to the next element.
    cursor: *const u8,
    
    /// The pointer to the past-the-end element.
    end: *const u8,
    
    /// The marker for ownership of `&[u8]`.
    _marker: PhantomData<&'a [u8]>,
}

/// All errors [crate::bytes] can produce.
#[repr(u8)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    /// Encountered a continuation byte where the byte 1 was expected.
    EncounteredContinuationByte,
    
    /// The input ended while decoding the second byte of a two byte sequence.
    Missing2ndOf2,
    
    /// The second byte of a two byte sequence is not a continuation byte.
    Invalid2ndOf2,

    /// The input ended while decoding the second byte of a three byte sequence.
    Missing2ndOf3,

    /// The second byte of a three byte sequence is not a continuation byte.
    Invalid2ndOf3,

    /// The input ended while decoding the third byte of a three byte sequence.
    Missing3rdOf3,
    
    /// The third byte of a three byte sequence is not a continuation byte.
    Invalid3rdOf3,
    
    /// The input ended while decoding the second byte of a four byte sequence.
    Missing2ndOf4,
    
    /// The second byte of a four byte sequence is not a continuation byte.
    Invalid2ndOf4,

    /// The input ended while decoding the third byte of a four byte sequence.
    Missing3rdOf4,
    
    /// The third byte of a four byte sequence is not a continuation byte.
    Invalid3rdOf4,

    /// The input ended while decoding the fourth byte of a four byte sequence.
    Missing4thOf4,
    
    /// The fourth byte of a four byte sequence is not a continuation byte.
    Invalid4thOf4,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub fn skip_ascii_whitespace(&mut self) {
        loop {
            match self.peek() {
                None => break,
                Some(x) if !x.is_ascii_whitespace() => break,
                Some(_) => {
                    unsafe { self.advance_unchecked() }
                }
            }
        }
    }
    
    #[inline]
    pub const fn cursor(&self) -> *const u8 {
        self.cursor
    }
    
    #[inline]
    pub const fn new(slice: &[u8]) -> Self {
        Self {
            cursor: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            _marker: PhantomData,
        }
    }
    
    /// Gets the next byte. Normalizes line terminators by mapping CR, CRLF and LF sequences to LF.
    #[inline]
    pub fn next_lfn(&mut self) -> Option<u8> {
        match self.peek() {
            None => None,
            Some(b'\r') => {
                unsafe { self.advance_unchecked() }
                
                if self.peek() == Some(b'\n') {
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
    pub fn next(&mut self) -> Option<u8> {
        if self.has_next() {
            let byte = unsafe { self.peek_unchecked() };
            unsafe { self.advance_unchecked() };
            Some(byte)
        } else {
            None
        }
    }

    /// Gets the next byte. Does not normalize line terminators.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor has a next byte.
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> u8 {
        let byte = self.peek_unchecked();
        self.advance_unchecked();
        byte
    }

    /// Peeks into the next byte. Does not advance the iterator.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if !self.has_next() {
            None
        } else {
            Some(unsafe { self.peek_unchecked() })
        }
    }

    /// Peeks into the nth byte, first byte is n=0. Does not advance.
    #[inline]
    pub fn peek_n(&self, n: usize) -> Option<u8> {
        let desired_byte_ptr = unsafe { self.cursor.add(n) };

        if desired_byte_ptr < self.end {
            Some(unsafe { *desired_byte_ptr })
        } else {
            None
        }
    }
    
    /// Checks if the cursor has a next byte.
    #[inline]
    pub fn has_next(&self) -> bool {
        self.cursor < self.end
    }
    
    /// Peeks into the next byte. Does not advance the iterator.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor has a next byte.
    #[inline]
    pub unsafe fn peek_unchecked(&self) -> u8 {
        *self.cursor
    }

    /*
    #[inline]
    #[deprecated]
    pub fn rewind_lfn(&mut self) {
        if self.can_rewind() {
            self.cursor = unsafe { self.cursor.sub(1) };
            
            if unsafe { *self.cursor } == b'\n'
                && self.cursor != self.first
                && unsafe { *self.cursor.sub(1) } == b'\r' {
                self.cursor = unsafe { self.cursor.sub(1) };
            }
        }
    }
    
    /// Checks if the cursor can be rewinded.
    #[inline]
    #[deprecated]
    pub fn can_rewind(&mut self) -> bool {
        self.cursor > self.first
    }
    
    /// Rewinds one byte. Saturates at the lower boundary.
    #[inline]
    #[deprecated]
    pub fn rewind(&mut self) {
        if self.can_rewind() {
            unsafe { self.rewind_unchecked(); }
        }
    }
    */
    
    /// Rewinds one byte.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor can rewind.
    #[inline]
    pub unsafe fn rewind_unchecked(&mut self) {
        self.cursor = self.cursor.sub(1);
    }
    
    /// Advances one char, saturates at the upper boundary.
    #[inline]
    pub fn advance(&mut self) {
        if self.has_next() {
            unsafe { self.advance_unchecked(); }
        }
    }
    
    /// Advances the cursor by one byte.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor is not at the end.
    #[inline]
    pub unsafe fn advance_unchecked(&mut self) {
        self.cursor = self.cursor.add(1)
    }

    /// Advances the cursor by n bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the next n bytes are valid to skip over.
    #[inline]
    pub unsafe fn advance_n_unchecked(&mut self, n: usize) {
        self.cursor = self.cursor.add(n);
    }

    #[inline]
    pub unsafe fn advance_char_unchecked(&mut self) {
        self.cursor = self.cursor.add(UTF8_CHAR_WIDTH[self.peek_unchecked() as usize] as usize);
    }
    
    /// Advances the cursor by one char encoded as UTF-8.
    #[inline]
    pub fn advance_char(&mut self) -> Result<(), Error> {
        let first_byte = match self.next() {
            Some(x) => x,
            None => return Ok(()),
        };

        macro_rules! next {
            ($e:expr,$i:expr) => {
                match self.next_lfn() {
                    None => return Err($e),
                    Some(x) if x & 0b1100_0000 != 0b1000_0000 => return Err($i),
                    _ => {},
                }
            };
        }

        match UTF8_CHAR_WIDTH[first_byte as usize] {
            0 => Err(Error::EncounteredContinuationByte),
            1 => {
                if first_byte == b'\r' && self.peek() == Some(b'\n')  {
                    unsafe { self.advance_unchecked() }
                }
                Ok(())
            },
            2 => {
                next!(Error::Missing2ndOf2, Error::Invalid2ndOf2);
                Ok(())
            }
            3 => {
                next!(Error::Missing2ndOf3, Error::Invalid2ndOf3);
                next!(Error::Missing3rdOf3, Error::Invalid3rdOf3);
                Ok(())
            }
            4 => {
                next!(Error::Missing2ndOf4, Error::Invalid2ndOf4);
                next!(Error::Missing3rdOf4, Error::Invalid3rdOf4);
                next!(Error::Missing4thOf4, Error::Invalid4thOf4);
                Ok(())
            }
            _ => unsafe { unreachable_unchecked() }
        }
    }

    // #[inline]
    // pub const fn offset(&self) -> usize {
    //     unsafe { self.cursor.sub_ptr(self.first) }
    // }
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