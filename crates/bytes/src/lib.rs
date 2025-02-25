#![feature(ptr_sub_ptr)]
#![no_std]

//! # Bytes
//!
//! This crate provides functionality to step and iterate through a slice of bytes.

mod tests;

use core::fmt::Debug;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::ops::Range;
use core::slice;

/// An iterator over a slice. It does not allow to go backwards.
pub struct Cursor<'a> {
    /// The pointer to the first element. Dangling if the initial slice is of length zero.
    start: *const u8,

    /// The pointer to the next element.
    cursor: *const u8,
    
    /// The pointer to the past-the-end element.
    end: *const u8,
    
    /// The marker for ownership of `&[u8]`.
    _marker: PhantomData<&'a [u8]>,
}

/// All errors this crate can produce.
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
    /// Returns the slice wrapped by this cursor.
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(
            self.start,
            self.end as usize - self.start as usize
        ) }
    }

    /// Returns the index of the next byte. Therefore, it is equal to the number of bytes consumed.
    #[inline]
    pub fn index(&self) -> Index {
        unsafe { self.cursor.sub_ptr(self.start) as Index }
    }

    /// Skips ASCII whitespace (meaning spaces, line breaks, carriage returns, vertical and
    /// horizontal tabs).
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

    /// Returns the current cursor position.
    #[inline]
    pub const fn position(&self) -> Position<'a> {
        unsafe { Position::new(self.cursor) }
    }

    /// Returns the slice from the given position to the current position.
    #[inline]
    pub fn slice_from(&self, pos: Position<'a>) -> &'a [u8] {
        unsafe { slice::from_raw_parts(
            pos.0,
            self.cursor.sub_ptr(pos.0)
        ) }
    }
    
    // #[inline]
    // pub const fn end(&self) -> *const u8 { self.end }

    /// Constructs a new [Cursor].
    #[inline]
    pub const fn new(slice: &[u8]) -> Self {
        Self {
            start: slice.as_ptr(),
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
        if self.has_next() {
            Some(unsafe { self.peek_unchecked() })
        } else {
            None
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
    
    /// Peeks into the next byte. Does not advance.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the cursor has a next byte.
    #[inline]
    pub unsafe fn peek_unchecked(&self) -> u8 {
        *self.cursor
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
                match self.next() {
                    None => return Err($e),
                    Some(x) if x & 0b1100_0000 != 0b1000_0000 => return Err($i),
                    _ => {},
                }
            };
        }

        match UTF8_CHAR_WIDTH[first_byte as usize] {
            0 => Err(Error::EncounteredContinuationByte),
            1 => Ok(()),
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

pub fn get_lines_and_columns(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut lines = 0;
    let mut columns = 0;
    let mut i = 0;

    let mut cursor = Cursor::new(source.as_bytes());

    loop {
        if byte_offset <= i { break }

        match cursor.peek() {
            Some(b'\r') => {
                lines += 1;
                columns = 0;

                unsafe { cursor.advance_unchecked() }

                if let Some(b'\n') = cursor.peek() {
                    unsafe { cursor.advance_unchecked() }
                    i += 1;
                }
            }
            Some(b'\n') => {
                lines += 1;
                columns = 0;

                unsafe { cursor.advance_unchecked() }
            }
            Some(_) => {
                columns += 1;
                unsafe { cursor.advance_char_unchecked() }
            }
            None => break
        }

        i += 1;
    }

    (lines, columns)
}

/// A cursor position.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position<'a>(*const u8, PhantomData<&'a u8>);

impl<'a> Position<'a> {
    /// Creates a new position.
    #[inline]
    const unsafe fn new(ptr: *const u8) -> Self {
        Self(ptr, PhantomData)
    }
    
    /// Returns the slice bounded by `self` and the parameter `next`.
    /// 
    /// **Panics if `self > next`**.
    #[inline]
    pub fn slice_to(self, next: Position<'a>) -> &'a [u8] {
        let size = unsafe { next.0.offset_from(self.0) };
        
        if size < 0 {
            panic!("Next position is previous");
        }
        
        unsafe { slice::from_raw_parts(
            self.0,
            size as usize
        ) }
    }
}