use std::ops::{Deref, DerefMut};
use std::ptr::slice_from_raw_parts;
use std::str::{Chars, from_utf8_unchecked};
use crate::lex::Error;

pub struct CharsIterator<'a> {
    chars: Chars<'a>,
    force_take: Option<char>,
    index: usize,
}

impl<'s> From<Chars<'s>> for CharsIterator<'s> {
    #[inline]
    fn from(value: Chars<'s>) -> Self {
        CharsIterator {
            chars: value,
            force_take: None,
            index: 0,
        }
    }
}

impl<'s> CharsIterator<'s> {
    #[inline]
    pub fn new(chars: Chars<'s>) -> Self {
        chars.into()
    }
    
    pub fn unescape_char(&mut self, char: char) -> Result<char, crate::Error> {
        // TODO: unicode escapes (use of self)
        match char {
            '0' => Ok('\0'), // Null character
            '\\' => Ok('\\'),
            'f' => Ok('\u{0c}'), // Form feed
            't' => Ok('\t'),     // Horizontal tab
            'r' => Ok('\r'),     // Carriage return
            'n' => Ok('\n'),     // Line feed / new line
            'b' => Ok('\u{07}'), // Bell
            'v' => Ok('\u{0b}'), // Vertical tab
            '"' => Ok('"'),
            '\'' => Ok('\''),
            '[' => Ok('\u{1B}'), // Escape
            char => Err(crate::Error::Lexer(Error::UnknownEscapeCharacter(char))),
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<char> {
        self.index += 1;
        self.force_take.take().or_else(|| self.chars.next())
    }

    #[inline]
    pub fn rollback(&mut self, option: Option<char>) {
        self.index -= 1;
        self.force_take = option;
    }

    #[inline]
    pub fn skip_white_space(&mut self) {
        self.skip_until(|c| !c.is_whitespace());
    }
    
    #[inline]
    pub fn skip_until_line_break(&mut self) {
        self.skip_until(|c| !c.is_whitespace() && c != '\n' && c != '\r')
    }
    
    /// Fast forwards to the char that makes this predicate true.
    #[inline]
    pub fn skip_until<F: Fn(char) -> bool>(&mut self, predicate: F) {
        loop {
            match self.next() {
                Some(char) if !predicate(char) => {}
                option => {
                    self.rollback(option);
                    break;
                }
            }
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn begin_extraction<'c>(&'c mut self) -> Recorder<'c, 's> {
        Recorder {
            ptr: unsafe { self.chars.as_str().as_ptr()
                .offset(-self.force_take.map_or(0, |c| c.len_utf8() as isize)) },
            chars: self,
        }
    }
}

/// Allows [CharsIterator] to extract string slices.
pub struct Recorder<'c, 's> {
    chars: &'c mut CharsIterator<'s>,
    ptr: *const u8
}

impl<'c, 's> Recorder<'c, 's> {
    #[inline]
    pub fn finish(self) -> &'s str {
        let new_ptr = self.chars.chars.as_str().as_ptr();
        
        unsafe { from_utf8_unchecked(&*slice_from_raw_parts::<u8>(
            self.ptr,
            new_ptr.offset_from(self.ptr) as usize
                - self.chars.force_take.map_or(0, |c| c.len_utf8())
        )) }
    }
}

impl<'a, 'b> Deref for Recorder<'a, 'b> {
    type Target = &'a mut CharsIterator<'b>;

    fn deref(&self) -> &Self::Target {
        &self.chars
    }
}

impl<'a, 'b> DerefMut for Recorder<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chars
    }
}