#![feature(ptr_sub_ptr)]
#![feature(str_from_raw_parts)]

use std::fmt::Debug;
use std::hint::unreachable_unchecked;
use std::str::from_raw_parts;

use parse_tools::bytes::Cursor;

use error::Error;
use token::{KEYWORDS, Symbol, Token};
use crate::simple::SimpleLexer;

mod tests;
pub mod token;
mod simple;

pub enum Layer {
    /// This layer expects `key=`, `/>` or `>`.
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    EndTag,
    Insert,
    StartTag,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Span<'a, T: Debug + Clone> {
    pub value: T,
    pub source: &'a str,
}

impl<'a, T: Debug + Clone> Span<'a, T> {
    /// Constructs a [Span] from two pointers, linking the value back to its source.
    pub unsafe fn from_ends(value: T, first: *const u8, end: *const u8) -> Self {
        Self {
            value,
            source: from_raw_parts(first, end.sub_ptr(first)),
        }
    }
}

pub struct Lexer<'a> {
    simple: SimpleLexer<'a>,
    layers: Vec<Layer>,
}

#[inline]
const fn try_to_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0' => Some(0),
        b'1' => Some(1),
        b'2' => Some(2),
        b'3' => Some(3),
        b'4' => Some(4),
        b'5' => Some(5),
        b'6' => Some(6),
        b'7' => Some(7),
        b'8' => Some(8),
        b'9' => Some(9),
        b'A' | b'a' => Some(10),
        b'B' | b'b' => Some(11),
        b'C' | b'c' => Some(12),
        b'D' | b'd' => Some(13),
        b'E' | b'e' => Some(14),
        b'F' | b'f' => Some(15),
        _ => None,
    }
}

impl<'a> Lexer<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            simple: SimpleLexer::new(cursor),
            layers: Vec::new(),
        }
    }

    /// ```text
    ///  v
    /// <tag...
    /// ```
    /// 
    /// Assumes that the next byte is the id.
    pub fn parse_start_tag(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        let first = self.simple.cursor.cursor();

        self.simple.skip_whitespace();

        let identifier = self.simple.parse_id()?;
        if KEYWORDS.contains_key(identifier) {
            return Err(Error::E0015);
        }

        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

        Ok(unsafe {
            Span::from_ends(
                Token::MarkupStartTag(identifier),
                first,
                self.simple.cursor.cursor(),
            )
        })
    }

    pub fn parse_end_tag(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        let first = self.simple.cursor.cursor();

        self.simple.skip_whitespace();

        let tag_name = self.simple.parse_id()?;
        if KEYWORDS.contains_key(tag_name) {
            return Err(Error::E0016);
        }

        self.simple.skip_whitespace();

        match self.simple.cursor.next_lfn() {
            Some(b'>') => Ok(unsafe {
                Span::from_ends(
                    Token::MarkupEndTag(tag_name),
                    first,
                    self.simple.cursor.cursor(),
                )
            }),
            _ => Err(Error::E0026),
        }
    }

    pub fn next_token(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        match self.layers.pop() {
            None => {
                if let Some(line_break) = self.simple.skip_whitespace_line() {
                    return Ok(line_break)
                }

                if self.simple.potential_markup {
                    self.simple.potential_markup = false;

                    if self.simple.cursor.peek() == Some(b'<') {
                        unsafe { self.simple.cursor.advance_unchecked() }
                        self.parse_start_tag()
                    } else {
                        self.simple.next_token()
                    }
                } else {
                    self.simple.next_token()
                }
            }
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.simple.skip_whitespace();

                let start = self.simple.cursor.cursor();

                let token = match self.simple.cursor.peek() {
                    Some(b'>') => {
                        unsafe {
                            self.simple.cursor.advance_unchecked();
                        }
                        self.layers.push(Layer::TextOrInsert);
                        Token::MarkupStartTagEnd
                    }
                    Some(b'/') => {
                        unsafe {
                            self.simple.cursor.advance_unchecked();
                        }
                        self.simple.skip_whitespace();

                        match self.simple.cursor.next_lfn() {
                            Some(b'>') => Token::MarkupClose,
                            _ => return Err(Error::E0033),
                        }
                    }
                    Some(char) if char.is_ascii_alphabetic() || char == b'_' => {
                        self.layers.push(Layer::Value);
                        Token::MarkupKey(self.simple.parse_id()?)
                    }
                    _ => return Err(Error::E0034),
                };

                Ok(unsafe {
                    Span::from_ends(
                        token,
                        start,
                        self.simple.cursor.cursor(),
                    )
                })
            }
            Some(Layer::Value) => {
                self.simple.skip_whitespace();

                match self.simple.cursor.next_lfn() {
                    Some(b'=') => {}
                    _ => return Err(Error::E0035),
                }

                self.simple.skip_whitespace();

                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                let start = self.simple.cursor.cursor();

                let token = match self.simple.cursor.next_lfn() {
                    Some(b'"') => Token::String(self.simple.parse_string()?),
                    Some(b'{') => {
                        self.layers.push(Layer::Insert);
                        self.simple.potential_markup = true;
                        Token::Symbol(Symbol::LeftBrace)
                    }
                    _ => return Err(Error::E0036),
                };

                Ok(unsafe {
                    Span::from_ends(
                        token,
                        start,
                        self.simple.cursor.cursor(),
                    )
                })
            }
            Some(Layer::Insert) => {
                let token = self.simple.next_token()?;

                match &token.value {
                    Token::Symbol(Symbol::LeftBrace) => {
                        self.layers.push(Layer::Insert);
                        self.layers.push(Layer::Insert);
                    }
                    Token::Symbol(Symbol::RightBrace) => {}
                    _ => self.layers.push(Layer::Insert),
                }

                Ok(token)
            }
            Some(Layer::TextOrInsert) => {
                self.simple.skip_whitespace();

                let first = self.simple.cursor.cursor();

                loop {
                    match self.simple.cursor.peek() {
                        Some(b'<' | b'{') => break,
                        Some(_) => self.simple.cursor.advance_char().map_err(|e| e.into())?,
                        None => return Err(Error::E0037),
                    }
                }
                
                let text = unsafe {
                    from_raw_parts(first, self.simple.cursor.cursor().sub_ptr(first))
                };

                Ok(if text.len() == 0 {
                    // If there is no text, we have to yield something.

                    match self.simple.cursor.next_lfn() {
                        // Yield an end tag or a nested element start
                        Some(b'<') => {
                            self.simple.skip_whitespace();

                            match self.simple.cursor.peek() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe { self.simple.cursor.advance_unchecked() }
                                    self.parse_end_tag()?
                                }

                                // Nested element
                                Some(_) => {
                                    self.layers.push(Layer::TextOrInsert);
                                    self.parse_start_tag()?
                                }
                            }
                        }

                        // Yield an insert start
                        Some(b'{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.layers.push(Layer::Insert);
                            self.simple.potential_markup = true;

                            unsafe {
                                Span::from_ends(
                                    Token::Symbol(Symbol::LeftBrace),
                                    self.simple.cursor.cursor().sub(1),
                                    self.simple.cursor.cursor(),
                                )
                            }
                        }

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() },
                    }
                } else {
                    // There is some text, so we yield the text and prepare the next layer state.

                    match self.simple.cursor.peek() {
                        Some(b'<') => {
                            unsafe {
                                self.simple.cursor.advance_unchecked();
                            }

                            self.simple.skip_whitespace();

                            match self.simple.cursor.peek() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe {
                                        self.simple.cursor.advance_unchecked();
                                    }
                                    self.layers.push(Layer::EndTag)
                                }

                                // Nested element
                                _ => {
                                    // Add context
                                    self.layers.push(Layer::TextOrInsert);
                                    self.layers.push(Layer::StartTag);
                                }
                            }
                        }
                        Some(b'{') => self.layers.push(Layer::TextOrInsert),

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() },
                    }

                    unsafe {
                        Span::from_ends(
                            Token::MarkupText(text),
                            first,
                            self.simple.cursor.cursor(),
                        )
                    }
                })
            }
            Some(Layer::EndTag) => self.parse_end_tag(),
            Some(Layer::StartTag) => self.parse_start_tag(),
        }
    }
}