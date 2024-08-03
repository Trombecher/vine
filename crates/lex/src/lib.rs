//! # Vine Lexer Module
//! 
//! This module contains the code to iterate over the tokens of a buffer (string).
//! 
//! **The lexer will NOT produce two adjacent line break tokens.**

#![feature(ptr_sub_ptr)]
#![feature(str_from_raw_parts)]
#![feature(const_ptr_sub_ptr)]

use std::fmt::Debug;
use std::hint::unreachable_unchecked;
use std::str::from_raw_parts;
use parse_tools::bytes::Cursor;

use error::Error;
use token::{KEYWORDS, Symbol, Token};
use warning::Warning;
use crate::token::{TokenIterator, unescape_char, UnprocessedString};

mod tests;
pub mod token;

pub enum Layer {
    /// This layer expects `key=`, `/>` or `>`.
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    EndTag,
    Insert,
    StartTag,
}

/// Links the value back to a slice of the source file.
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
    
    #[inline]
    pub fn map<U: Debug + Clone>(self, map: impl FnOnce(T) -> U) -> Span<'a, U> {
        Span {
            value: map(self.value),
            source: self.source,
        }
    }
}

pub struct Lexer<'a> {
    /// The start of the slice.
    start: *const u8,

    /// The underlying iterator over the bytes.
    cursor: Cursor<'a>,

    /// Signals [Layer::Insert] or no layer that the following '<'
    /// may be interpreted as the start of a markup element.
    potential_markup: bool,

    /// A stack of layers to manage 
    layers: Vec<Layer>,

    pub warnings: Vec<Span<'a, Warning>>
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
    /// Constructs a new [Lexer].
    pub const fn new(slice: &'a [u8]) -> Self {
        Self {
            start: slice.as_ptr(),
            potential_markup: false,
            layers: Vec::new(),
            cursor: Cursor::new(slice),
            warnings: Vec::new(),
        }
    }

    pub const fn cursor_offset(&self) -> usize {
        unsafe { self.cursor.cursor().sub_ptr(self.start) }
    }

    /// Parses a decimal number.
    ///
    /// # Safety
    ///
    /// Expects a next byte.
    #[inline]
    fn parse_number_dec(&mut self, mut number: f64) -> Result<Token<'a>, Error> {
        unsafe { self.cursor.advance_unchecked() }

        loop {
            match self.cursor.peek() {
                Some(b'_') => unsafe { self.cursor.advance_unchecked() },
                Some(x) if matches!(x, b'0'..=b'9') => {
                    unsafe { self.cursor.advance_unchecked() }
                    number = number * 10. + (x - b'0') as f64
                }
                Some(b'.') => {
                    number = self.parse_number_dec_tail(number)?;
                    break;
                }
                Some(x) if x.is_ascii_alphabetic() => return Err(Error::E0017),
                _ => break,
            }
        }

        Ok(Token::Number(number))
    }

    /// # Safety
    ///
    /// Expects `peek()` to output `Some(b'.')` on function call.
    #[inline]
    fn parse_number_dec_tail(&mut self, mut number: f64) -> Result<f64, Error> {
        match self.cursor.peek_n(1) {
            Some(b'0') => {}
            Some(b'1') => number += 0.1,
            Some(b'2') => number += 0.2,
            Some(b'3') => number += 0.3,
            Some(b'4') => number += 0.4,
            Some(b'5') => number += 0.5,
            Some(b'6') => number += 0.6,
            Some(b'7') => number += 0.7,
            Some(b'8') => number += 0.8,
            Some(b'9') => number += 0.9,
            _ => return Ok(number)
        }

        unsafe { self.cursor.advance_unchecked() } // TODO: maybe merge?
        unsafe { self.cursor.advance_unchecked() }

        let mut multiplier = 0.1;

        loop {
            number += match self.cursor.next_lfn() {
                Some(b'_') => continue,
                Some(b'0') => {
                    multiplier *= 0.1;
                    continue;
                }
                Some(b'1') => {
                    multiplier *= 0.1;
                    1.0
                }
                Some(b'2') => {
                    multiplier *= 0.1;
                    2.0
                }
                Some(b'3') => {
                    multiplier *= 0.1;
                    3.0
                }
                Some(b'4') => {
                    multiplier *= 0.1;
                    4.0
                }
                Some(b'5') => {
                    multiplier *= 0.1;
                    5.0
                }
                Some(b'6') => {
                    multiplier *= 0.1;
                    6.0
                }
                Some(b'7') => {
                    multiplier *= 0.1;
                    7.0
                }
                Some(b'8') => {
                    multiplier *= 0.1;
                    8.0
                }
                Some(b'9') => {
                    multiplier *= 0.1;
                    9.0
                }
                Some(x) if x.is_ascii_alphabetic() => return Err(Error::E0029), // TODO: Expand this to unicode alphabetic
                _ => break,
            } * multiplier;
        }

        Ok(number)
    }

    /// Expects the next byte to be after the quote.
    pub(crate) fn parse_string(&mut self) -> Result<UnprocessedString<'a>, Error> {
        let first = self.cursor.cursor();

        loop {
            match self.cursor.peek() {
                None => break Err(Error::E0019),
                Some(b'"') => {
                    let end = self.cursor.cursor();

                    unsafe { self.cursor.advance_unchecked() }

                    break Ok(unsafe {
                        UnprocessedString::from_raw(from_raw_parts(first, end.sub_ptr(first)))
                    })
                }
                Some(b'\\') => {
                    unsafe { self.cursor.advance_unchecked() }
                    self.cursor.advance();
                }
                Some(_) => {
                    self.cursor.advance_char().map_err(|e| e.into())?
                }
            }
        }
    }

    /*
    pub(crate) fn parse_string(&mut self) -> Result<String, Error> {
        let mut s = String::with_capacity(32);

        loop {
            match self.cursor.peek() {
                Some(b'"') => {
                    unsafe {
                        self.cursor.advance_unchecked();
                    }
                    break Ok(s);
                }
                Some(b'\\') => {
                    unsafe {
                        self.cursor.advance_unchecked();
                    }
                    s.push(self.unescape_char()?);
                }
                None => break Err(Error::E0019),
                Some(_) => {
                    let start = self.cursor.cursor();

                    if let Err(e) = self.cursor.advance_char() {
                        break Err(e.into());
                    }

                    unsafe {
                        s.push_str(transmute(slice_from_raw_parts(
                            start,
                            self.cursor.cursor().sub_ptr(start),
                        )))
                    }
                }
            }
        }
    }
     */

    pub(crate) fn parse_id(&mut self) -> Result<&'a str, Error> {
        let first = self.cursor.cursor();

        loop {
            match self.cursor.peek() {
                Some(x) if x.is_ascii_alphanumeric() || x == b'_' => unsafe {
                    self.cursor.advance_unchecked()
                }
                Some(128..=255) => return Err(Error::E0020),
                _ => break,
            }
        }

        Ok(unsafe {
            from_raw_parts(first, self.cursor.cursor().sub_ptr(first))
        })
    }

    fn parse_comment(&mut self) -> Result<&'a str, Error> {
        let first = self.cursor.cursor();

        loop {
            match self.cursor.peek() {
                None | Some(b'\n') | Some(b'\r') => break,
                Some(128..=255) => {
                    self.cursor.advance_char().map_err(|e| e.into())?;
                }
                _ => unsafe { self.cursor.advance_unchecked() }
            }
        }

        Ok(unsafe {
            from_raw_parts(first, self.cursor.cursor().sub_ptr(first))
        })
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            match self.cursor.peek() {
                Some(x) if x.is_ascii_whitespace() => unsafe {
                    self.cursor.advance_unchecked()
                },
                _ => break,
            }
        }
    }

    /// Skips whitespace until a line break or a non-whitespace character was encountered.
    /// It normalizes all line termination sequences (LF, CRLF, CR).
    ///
    /// In case a line break was encountered, it consumes the line break returns [Some] with the line feed token;
    /// else it does not consume the character and returns [None].
    #[must_use]
    pub fn skip_whitespace_line(&mut self) -> Option<Span<'a, Token<'a>>> {
        loop {
            match self.cursor.peek() {
                Some(b'\r') => {
                    let first = self.cursor.cursor();
                    unsafe { self.cursor.advance_unchecked() }

                    if let Some(b'\n') = self.cursor.peek() {
                        unsafe { self.cursor.advance_unchecked() }
                    }
                    
                    let result = Some(unsafe {
                        Span::from_ends(
                            Token::LineBreak,
                            first,
                            self.cursor.cursor()
                        )
                    });
                    
                    self.skip_whitespace(); // Skip additional whitespace

                    break result
                }
                Some(b'\n') => {
                    let source = unsafe {
                        from_raw_parts(self.cursor.cursor(), 1)
                    };

                    unsafe { self.cursor.advance_unchecked() }
                    
                    self.skip_whitespace(); // Skip additional whitespace
                    
                    break Some(Span { value: Token::LineBreak, source })
                }
                Some(x) if x.is_ascii_whitespace() => unsafe {
                    self.cursor.advance_unchecked()
                },
                _ => break None,
            }
        }
    }

    /// Parses a binary number.
    ///
    /// # Safety
    ///
    /// Expects a next byte.
    fn parse_number_bin(&mut self) -> Result<Token<'a>, Error> {
        unsafe { self.cursor.advance_unchecked() }

        let mut number = 0_f64;
        let mut multiplier = 0.1_f64;

        // TODO: Optimizations regarding floating point arithmetic.

        loop {
            match self.cursor.peek() {
                Some(b'0' | b'_') => unsafe { self.cursor.advance_unchecked() },
                Some(b'1') => {
                    unsafe { self.cursor.advance_unchecked() }
                    number += multiplier;
                    multiplier /= 2.0;
                }
                Some(digit) if digit.is_ascii_alphanumeric() => return Err(Error::E0017),
                _ => return Ok(Token::Number(number)),
            }
        }
    }

    pub fn next_token_default(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        macro_rules! opt_eq {
            ($symbol: expr, $eq: expr) => {{
                unsafe { self.cursor.advance_unchecked() };
                match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        $eq
                    }
                    _ => $symbol,
                }
            }};
        }

        let start = self.cursor.cursor();

        let token: Token = match self.cursor.peek() {
            None => Token::EndOfInput,
            Some(b'0') => {
                unsafe { self.cursor.advance_unchecked() };

                match self.cursor.peek() {
                    Some(b'x') => todo!("hex numbers"),
                    Some(b'o') => todo!("octal numbers"),
                    Some(b'b') => self.parse_number_bin()?,
                    Some(b'_') => self.parse_number_dec(0.)?,
                    Some(b'.') => Token::Number(self.parse_number_dec_tail(0.)?),
                    Some(b'0') => self.parse_number_dec(0.)?,
                    Some(b'1') => self.parse_number_dec(1.)?,
                    Some(b'2') => self.parse_number_dec(2.)?,
                    Some(b'3') => self.parse_number_dec(3.)?,
                    Some(b'4') => self.parse_number_dec(4.)?,
                    Some(b'5') => self.parse_number_dec(5.)?,
                    Some(b'6') => self.parse_number_dec(6.)?,
                    Some(b'7') => self.parse_number_dec(7.)?,
                    Some(b'8') => self.parse_number_dec(8.)?,
                    Some(b'9') => self.parse_number_dec(9.)?,
                    Some(_) => Token::Number(0.), // TODO: major bug here!!
                    None => Token::Number(0.),
                }
            }
            Some(d) if matches!(d, b'1'..=b'9') => {
                self.parse_number_dec((d - b'0') as f64)?
            }
            Some(b'<') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::LeftAngleEquals
                    }
                    Some(b'<') => {
                        opt_eq!(Symbol::LeftAngleLeftAngle, Symbol::LeftAngleLeftAngleEquals)
                    }
                    _ => Symbol::LeftAngle,
                })
            }
            Some(b'>') => {
                unsafe { self.cursor.advance_unchecked() };

                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::RightAngleEquals
                    }
                    Some(b'>') => opt_eq!(
                        Symbol::RightAngleRightAngle,
                        Symbol::RightAngleRightAngleEquals
                    ),
                    _ => Symbol::RightAngle,
                })
            }
            Some(b'=') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Equals, Symbol::EqualsEquals))
            }
            Some(b'+') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals))
            }
            Some(b'-') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::MinusEquals
                    }
                    Some(b'>') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::MinusRightAngle
                    }
                    _ => Symbol::Minus,
                })
            }
            Some(b'*') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::StarEquals
                    }
                    Some(b'*') => opt_eq!(Symbol::StarStar, Symbol::StarStarEquals),
                    _ => Symbol::Star,
                })
            }
            Some(b'/') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Token::Symbol(Symbol::SlashEquals)
                    }
                    Some(b'/') => {
                        unsafe { self.cursor.advance_unchecked() };

                        match self.cursor.peek() {
                            Some(b'/') => {
                                unsafe { self.cursor.advance_unchecked() };
                                Token::DocComment(self.parse_comment()?)
                            }
                            _ => Token::LineComment(self.parse_comment()?),
                        }
                    }
                    _ => Token::Symbol(Symbol::Slash),
                }
            }
            Some(b'%') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals))
            }
            Some(b'|') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::PipeEquals
                    }
                    Some(b'|') => opt_eq!(Symbol::PipePipe, Symbol::PipePipeEquals),
                    _ => Symbol::Pipe,
                })
            }
            Some(b'&') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::AmpersandEquals
                    }
                    Some(b'&') => {
                        opt_eq!(Symbol::AmpersandAmpersand, Symbol::AmpersandAmpersandEquals)
                    }
                    _ => Symbol::Ampersand,
                })
            }
            Some(b'^') => {
                self.potential_markup = true;

                Token::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals))
            }
            Some(b'(') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(Symbol::LeftParenthesis)
            }
            Some(b')') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(Symbol::RightParenthesis)
            }
            Some(b'[') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(Symbol::LeftBracket)
            }
            Some(b']') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(Symbol::RightBracket)
            }
            Some(b'{') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(Symbol::LeftBrace)
            }
            Some(b'}') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(Symbol::RightBrace)
            }
            Some(b'.') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(Symbol::Dot)
            }
            Some(b',') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(Symbol::Comma)
            }
            Some(b';') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(Symbol::Semicolon)
            }
            Some(b':') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(match self.cursor.peek() {
                    Some(b':') => return Err(Error::E0027),
                    _ => Symbol::Colon,
                })
            }
            Some(b'!') => {
                unsafe { self.cursor.advance_unchecked() };

                match self.cursor.peek() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        self.potential_markup = true;

                        Token::Symbol(Symbol::ExclamationMarkEquals)
                    }
                    _ => Token::Symbol(Symbol::ExclamationMark),
                }
            }
            Some(b'?') => {
                unsafe { self.cursor.advance_unchecked() };

                Token::Symbol(match self.cursor.peek() {
                    Some(b'.') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::QuestionMarkDot
                    }
                    _ => Symbol::QuestionMark,
                })
            }
            Some(b'\'') => {
                unsafe { self.cursor.advance_unchecked() };
                
                let char = match self.cursor.next_lfn() {
                    None => return Err(Error::E0030),
                    Some(b'\\') => unescape_char(&mut self.cursor)?,
                    Some(b'\n') => todo!(),
                    Some(char) => char.into(),
                };

                match self.cursor.next_lfn() {
                    Some(b'\'') => {}
                    None => return Err(Error::E0031),
                    _ => return Err(Error::E0032),
                }

                Token::Char(char)
            }
            Some(b'@') => {
                unsafe { self.cursor.advance_unchecked() };
                Token::Symbol(Symbol::At)
            }
            Some(b'"') => {
                unsafe { self.cursor.advance_unchecked() };
                Token::String(self.parse_string()?)
            }
            Some(b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
                let str = self.parse_id()?;

                if let Some(kw) = KEYWORDS.get(str) {
                    Token::Keyword(*kw)
                } else {
                    Token::Identifier(str)
                }
            }
            Some(_) => return Err(Error::E0028),
        };

        Ok(unsafe {
            Span::from_ends(
                token,
                start,
                self.cursor.cursor(),
            )
        })
    }

    /// ```text
    ///  v
    /// <tag...
    /// ```
    ///
    /// Assumes that the next byte is the id.
    pub fn parse_start_tag(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        let first = self.cursor.cursor();

        self.skip_whitespace();

        let identifier = self.parse_id()?;

        if KEYWORDS.contains_key(identifier) {
            return Err(Error::E0015);
        }

        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

        Ok(unsafe {
            Span::from_ends(
                Token::MarkupStartTag(identifier),
                first,
                self.cursor.cursor(),
            )
        })
    }

    pub fn parse_end_tag(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        let first = self.cursor.cursor();

        self.skip_whitespace();

        let tag_name = self.parse_id()?;
        if KEYWORDS.contains_key(tag_name) {
            return Err(Error::E0016);
        }

        self.skip_whitespace();

        match self.cursor.next_lfn() {
            Some(b'>') => Ok(unsafe {
                Span::from_ends(
                    Token::MarkupEndTag(tag_name),
                    first,
                    self.cursor.cursor(),
                )
            }),
            _ => Err(Error::E0026),
        }
    }
}

impl<'a> TokenIterator<'a> for Lexer<'a> {
    fn next_token(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        match self.layers.pop() {
            None => {
                if let Some(line_break) = self.skip_whitespace_line() {
                    return Ok(line_break)
                }

                if self.potential_markup {
                    self.potential_markup = false;

                    if self.cursor.peek() == Some(b'<') {
                        unsafe { self.cursor.advance_unchecked() }
                        self.parse_start_tag()
                    } else {
                        self.next_token_default()
                    }
                } else {
                    self.next_token_default()
                }
            }
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.skip_whitespace();

                let start = self.cursor.cursor();

                let token = match self.cursor.peek() {
                    Some(b'>') => {
                        unsafe {
                            self.cursor.advance_unchecked();
                        }
                        self.layers.push(Layer::TextOrInsert);
                        Token::MarkupStartTagEnd
                    }
                    Some(b'/') => {
                        unsafe {
                            self.cursor.advance_unchecked();
                        }
                        self.skip_whitespace();

                        match self.cursor.next_lfn() {
                            Some(b'>') => Token::MarkupClose,
                            _ => return Err(Error::E0033),
                        }
                    }
                    Some(char) if char.is_ascii_alphabetic() || char == b'_' => {
                        self.layers.push(Layer::Value);
                        Token::MarkupKey(self.parse_id()?)
                    }
                    _ => return Err(Error::E0034),
                };

                Ok(unsafe {
                    Span::from_ends(
                        token,
                        start,
                        self.cursor.cursor(),
                    )
                })
            }
            Some(Layer::Value) => {
                self.skip_whitespace();

                match self.cursor.next_lfn() {
                    Some(b'=') => {}
                    _ => return Err(Error::E0035),
                }

                self.skip_whitespace();

                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                let start = self.cursor.cursor();

                let token = match self.cursor.next_lfn() {
                    Some(b'"') => Token::String(self.parse_string()?),
                    Some(b'{') => {
                        self.layers.push(Layer::Insert);
                        self.potential_markup = true;
                        Token::Symbol(Symbol::LeftBrace)
                    }
                    _ => return Err(Error::E0036),
                };

                Ok(unsafe {
                    Span::from_ends(
                        token,
                        start,
                        self.cursor.cursor(),
                    )
                })
            }
            Some(Layer::Insert) => {
                let token = self.next_token_default()?;

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
                self.skip_whitespace();

                let first = self.cursor.cursor();

                loop {
                    match self.cursor.peek() {
                        Some(b'<' | b'{') => break,
                        Some(_) => self.cursor.advance_char().map_err(|e| e.into())?,
                        None => return Err(Error::E0037),
                    }
                }

                let text = unsafe {
                    from_raw_parts(first, self.cursor.cursor().sub_ptr(first))
                };

                if text.len() == 0 {
                    // If there is no text, we have to yield something.

                    match self.cursor.next_lfn() {
                        // Yield an end tag or a nested element start
                        Some(b'<') => {
                            self.skip_whitespace();

                            match self.cursor.peek() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe { self.cursor.advance_unchecked() }
                                    self.parse_end_tag()
                                }

                                // Nested element
                                Some(_) => {
                                    self.layers.push(Layer::TextOrInsert);
                                    self.parse_start_tag()
                                }
                            }
                        }

                        // Yield an insert start
                        Some(b'{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;

                            Ok(unsafe {
                                Span::from_ends(
                                    Token::Symbol(Symbol::LeftBrace),
                                    self.cursor.cursor().sub(1),
                                    self.cursor.cursor(),
                                )
                            })
                        }

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() },
                    }
                } else {
                    // There is some text, so we yield the text and prepare the next layer state.

                    match self.cursor.peek() {
                        Some(b'<') => {
                            unsafe {
                                self.cursor.advance_unchecked();
                            }

                            self.skip_whitespace();

                            match self.cursor.peek() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe {
                                        self.cursor.advance_unchecked();
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

                    Ok(Span {
                        value: Token::MarkupText(text),
                        source: text,
                    })
                }
            }
            Some(Layer::EndTag) => self.parse_end_tag(),
            Some(Layer::StartTag) => self.parse_start_tag(),
        }
    }

    #[inline]
    fn warnings(&self) -> &[Span<'a, Warning>] {
        &self.warnings
    }

    #[inline]
    fn warnings_mut(&mut self) -> &mut Vec<Span<'a, Warning>> {
        &mut self.warnings
    }

    #[inline]
    fn consume_warnings(self) -> Vec<Span<'a, Warning>> {
        self.warnings
    }
}