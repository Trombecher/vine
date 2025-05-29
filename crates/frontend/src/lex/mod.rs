//! # Vine Lexer Module
//!
//! This module contains the code to iterate over the tokens of a buffer (string).
//!
//! **The lexer will NOT produce two adjacent line break tokens.**

use alloc::vec::Vec;
use byte_reader::Cursor;
use core::alloc::Allocator;
use core::str::from_utf8_unchecked;
use ecow::EcoString;
use errors::*;
use fallible_iterator::FallibleIterator;
use span::{Index, Span};

mod tests;
mod token;
mod unit_tests;

pub use token::*;

#[derive(Copy, Clone, PartialEq)]
pub enum Layer {
    /// This layer expects `key=`, `/>` or `>`.
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    EndTag,
    Insert,
    StartTag,
}

pub struct Lexer<'source, A: Allocator> {
    /// The underlying iterator over the bytes.
    cursor: Cursor<'source>,

    /// Signals [Layer::Insert] or no layer that the following '<'
    /// may be interpreted as the start of a markup element.
    potential_markup: bool,

    /// A stack of layers to manage
    layers: Vec<Layer, A>,
}

#[inline]
pub(super) fn unescape_char(cursor: &mut Cursor) -> Result<char, Error> {
    match cursor.next_lfn() {
        Some(b'0') => Ok('\0'),     // Null character
        Some(b'\\') => Ok('\\'),    // Backslash
        Some(b'f') => Ok('\u{0c}'), // Form feed
        Some(b't') => Ok('\t'),     // Horizontal tab
        Some(b'r') => Ok('\r'),     // Carriage return
        Some(b'n') => Ok('\n'),     // Line feed / new line
        Some(b'b') => Ok('\u{07}'), // Bell
        Some(b'v') => Ok('\u{0b}'), // Vertical tab
        Some(b'"') => Ok('"'),      // Double quote
        Some(b'\'') => Ok('\''),    // Single quote
        Some(b'[') => Ok('\u{1B}'), // Escape
        Some(b'x') => {
            // Hexadecimal code
            match cursor.next_lfn().and_then(try_to_hex) {
                None => error!("Expected hexadecimal digit in escape sequence"),
                Some(x) => match cursor.next_lfn().and_then(try_to_hex) {
                    None => error!("Expected a second hexadecimal digit in escape sequence"),
                    Some(y) => Ok(((x << 4) + y) as char),
                },
            }
        }
        Some(b'u') => match cursor.next_lfn() {
            None => error!("Expected '{' in unicode escape sequence"),
            _ => error!("TODO"),
        },
        _ => error!("Invalid escape sequence"),
    }
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

impl<'source, A: Allocator> Lexer<'source, A> {
    /// Constructs a new [Lexer].
    pub const fn new(slice: &'source [u8], alloc: A) -> Self {
        Self {
            potential_markup: false,
            layers: Vec::new_in(alloc),
            cursor: Cursor::new(slice),
        }
    }

    /// Returns a reference to the underlying cursor.
    #[inline]
    pub const fn cursor(&self) -> &Cursor<'source> {
        &self.cursor
    }

    /// Parses a decimal number.
    ///
    /// # Safety
    ///
    /// Expects a next byte.
    #[inline]
    fn parse_number_dec(&mut self, mut number: f64) -> Result<Token<'source>, Error> {
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
                Some(x) if x.is_ascii_alphabetic() => {
                    return error!("Invalid digit in hexadecimal number")
                }
                _ => break,
            }
        }

        Ok(Token::Number(number))
    }

    /// # Safety
    ///
    /// Expects `peek()` to output `Some(b'.')` on function call.
    #[inline]
    pub(crate) fn parse_number_dec_tail(&mut self, mut number: f64) -> Result<f64, Error> {
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
            _ => return Ok(number),
        }

        unsafe { self.cursor.advance_n_unchecked(2) }

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
                Some(x) if x.is_ascii_alphabetic() => {
                    return error!("Expected a digit in decimal part")
                } // TODO: Expand this to unicode alphabetic
                _ => break,
            } * multiplier;
        }

        Ok(number)
    }

    /// Expects the next byte to be after the quote.
    pub(crate) fn parse_string(&mut self) -> Result<Token<'source>, Error> {
        let start = self.cursor.position();

        loop {
            match self.cursor.peek() {
                None => break error!("Unterminated string"),
                Some(b'"') => {
                    let string_bytes =
                        unsafe { from_utf8_unchecked(start.slice_to(self.cursor.position())) };

                    unsafe { self.cursor.advance_unchecked() }

                    break Ok(Token::String(EcoString::from(string_bytes)));
                }
                Some(b'\\') => {
                    unsafe { self.cursor.advance_unchecked() }
                    self.cursor.advance();
                }
                Some(_) => {
                    if let Err(_) = self.cursor.advance_char() {
                        return error!("UTF-8 Error: {}");
                    }
                }
            }
        }
    }

    pub(crate) fn parse_id(&mut self) -> Result<&'source str, Error> {
        let start_position = self.cursor.position();

        loop {
            match self.cursor.peek() {
                Some(x) if x.is_ascii_alphanumeric() || x == b'_' => unsafe {
                    self.cursor.advance_unchecked()
                },
                Some(128..=255) => return error!("Invalid character in identifier"),
                _ => break,
            }
        }

        Ok(unsafe {
            // SAFETY: slice contains UTF-8 because of the loop.
            from_utf8_unchecked(start_position.slice_to(self.cursor.position()))
        })
    }

    #[inline]
    pub(crate) fn skip_whitespace(&mut self) {
        loop {
            match self.cursor.peek() {
                Some(x) if x.is_ascii_whitespace() => unsafe { self.cursor.advance_unchecked() },
                _ => break,
            }
        }
    }

    /// Skips whitespace and line comments. It returns the first line break encountered, if any.
    #[must_use]
    pub fn skip_whitespace_line(&mut self) -> Option<Span<Token<'source>>> {
        let mut lb: Option<Span<Token<'source>>> = None;

        macro_rules! handle_lf {
            () => {{
                if lb.is_none() {
                    let start = self.cursor.bytes_consumed();
                    unsafe { self.cursor.advance_unchecked() }

                    lb = Some(Span {
                        value: Token::LineBreak,
                        source: start as Index..self.cursor.bytes_consumed() as Index,
                    });
                } else {
                    unsafe { self.cursor.advance_unchecked() }
                }
            }};
        }

        macro_rules! handle_crlf {
            () => {{
                if lb.is_none() {
                    let start = self.cursor.bytes_consumed();
                    unsafe { self.cursor.advance_unchecked() }

                    if let Some(b'\n') = self.cursor.peek() {
                        unsafe { self.cursor.advance_unchecked() }
                    }

                    lb = Some(Span {
                        value: Token::LineBreak,
                        source: start as Index..self.cursor.bytes_consumed() as Index,
                    });
                } else {
                    unsafe { self.cursor.advance_unchecked() }

                    if let Some(b'\n') = self.cursor.peek() {
                        unsafe { self.cursor.advance_unchecked() }
                    }
                }
            }};
        }

        loop {
            match self.cursor.peek() {
                Some(b'\r') => handle_crlf!(),
                Some(b'\n') => handle_lf!(),
                Some(x) if x.is_ascii_whitespace() => unsafe { self.cursor.advance_unchecked() },
                Some(b'/')
                    if Some(b'/') == self.cursor.peek_n(1)
                        && Some(b'/') != self.cursor.peek_n(2) =>
                {
                    // Skip comment

                    unsafe { self.cursor.advance_n_unchecked(2) }

                    loop {
                        match self.cursor.peek() {
                            Some(b'\n') => {
                                handle_lf!();
                                break;
                            }
                            Some(b'\r') => {
                                handle_crlf!();
                                break;
                            }
                            None => break,
                            _ => unsafe { self.cursor.advance_unchecked() },
                        }
                    }
                }
                _ => break,
            }
        }

        lb
    }

    /// Parses a binary number.
    ///
    /// # Safety
    ///
    /// Expects a next byte.
    fn parse_number_bin(&mut self) -> Result<Token<'source>, Error> {
        unsafe { self.cursor.advance_unchecked() }

        // Skip initial underscores
        loop {
            match self.cursor.peek() {
                Some(b'_') => unsafe { self.cursor.advance_unchecked() },
                _ => break,
            }
        }

        // A number must have a digit
        let mut number: u128 = match self.cursor.peek() {
            Some(b'0') => 0,
            Some(b'1') => 1,
            _ => return error!("Expected binary digit"),
        };

        unsafe { self.cursor.advance_unchecked() }

        loop {
            match self.cursor.peek() {
                Some(b'0') => number <<= 1,
                Some(b'1') => {
                    number <<= 1;
                    number |= 1;
                }
                Some(b'_') => {}
                Some(digit) if digit.is_ascii_alphanumeric() => {
                    return error!("Expected binary digit")
                }
                _ => return Ok(Token::Number(number as f64)),
            }

            unsafe { self.cursor.advance_unchecked() }
        }
    }

    fn next_token_default(&mut self) -> Result<Option<Span<Token<'source>>>, Error> {
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

        // Skip simple line comments, since they are not useful to the compiler.
        // But we have to yield something (recursion or a loop are not viable options),
        // so we yield a Token::LineBreak and make sure that all immediate following
        // line breaks are being skipped.
        if Some(b'/') == self.cursor.peek()
            && Some(b'/') == self.cursor.peek_n(1)
            && Some(b'/') != self.cursor.peek_n(2)
        {
            unsafe {
                self.cursor.advance_n_unchecked(2);
            }

            return loop {
                match self.cursor.peek() {
                    None => {
                        break Ok(None);
                    }
                    Some(b'\n') => {
                        let start = self.cursor.bytes_consumed() as Index;

                        unsafe {
                            self.cursor.advance_unchecked();
                        }

                        // Skip additional whitespace and line breaks.
                        self.skip_whitespace();

                        break Ok(Some(Span {
                            value: Token::LineBreak,
                            source: start..start + 1,
                        }));
                    }
                    Some(b'\r') => {
                        let start = self.cursor.bytes_consumed() as Index;

                        unsafe {
                            self.cursor.advance_unchecked();
                        }

                        break Ok(Some(if let Some(b'\n') = self.cursor.peek() {
                            unsafe {
                                self.cursor.advance_unchecked();
                            }

                            // Skip additional whitespace and line breaks.
                            self.skip_whitespace();

                            Span {
                                value: Token::LineBreak,
                                source: start..start + 2,
                            }
                        } else {
                            // Skip additional whitespace and line breaks.
                            self.skip_whitespace();

                            Span {
                                value: Token::LineBreak,
                                source: start..start + 1,
                            }
                        }));
                    }
                    _ => unsafe { self.cursor.advance_unchecked() },
                }
            };
        }

        let start = self.cursor.bytes_consumed() as Index;

        if let Some(byte) = self.cursor.peek() {
            let token: Token = match byte {
                b'0' => {
                    unsafe { self.cursor.advance_unchecked() };

                    match self.cursor.peek() {
                        Some(b'x') => {
                            return error!("TODO: hexadecimal numbers ought to be implemented")
                        }
                        Some(b'o') => return error!("TODO: Octal numbers ought to be implemented"),
                        Some(b'b') => self.parse_number_bin()?,
                        Some(b'_') => self.parse_number_dec(0.)?,
                        Some(b'.') => Token::Number(self.parse_number_dec_tail(0.)?),
                        Some(x) if matches!(x, b'0'..=b'9') => {
                            self.parse_number_dec((x - b'0') as f64)?
                        }
                        Some(_) => Token::Number(0.), // TODO: major bug here!!
                        None => Token::Number(0.),
                    }
                }
                d @ b'1'..=b'9' => self.parse_number_dec((d - b'0') as f64)?,
                b'<' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    match self.cursor.peek() {
                        Some(b'=') => {
                            unsafe { self.cursor.advance_unchecked() };
                            Token::Symbol(Symbol::LeftAngleEquals)
                        }
                        Some(b'<') => Token::Symbol(opt_eq!(
                            Symbol::LeftAngleLeftAngle,
                            Symbol::LeftAngleLeftAngleEquals
                        )),
                        _ => Token::Symbol(Symbol::LeftAngle),
                    }
                }
                b'>' => {
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
                b'=' => {
                    self.potential_markup = true;
                    Token::Symbol(opt_eq!(Symbol::Equals, Symbol::EqualsEquals))
                }
                b'+' => {
                    self.potential_markup = true;
                    Token::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals))
                }
                b'-' => {
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
                b'*' => {
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
                b'/' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    match self.cursor.peek() {
                        Some(b'=') => {
                            unsafe { self.cursor.advance_unchecked() };
                            Token::Symbol(Symbol::SlashEquals)
                        }
                        Some(b'/') => {
                            // SAFETY: We can skip two bytes here, since the "//?" case
                            // is already covered while skipping single line comments.
                            // Therefore, instead of any byte except for '/', the byte must be '/',
                            // which means there is a byte, which justifies skipping two bytes here
                            // instead of just one.
                            unsafe { self.cursor.advance_n_unchecked(2) };

                            let start = self.cursor.position();
                            let end;

                            loop {
                                match self.cursor.peek() {
                                    None => {
                                        end = self.cursor.position();
                                        break;
                                    }
                                    Some(b'\n') => {
                                        end = self.cursor.position();
                                        unsafe { self.cursor.advance_unchecked() }

                                        break;
                                    }
                                    Some(b'\r') => {
                                        end = self.cursor.position();
                                        unsafe { self.cursor.advance_unchecked() }

                                        if Some(b'\n') == self.cursor.peek() {
                                            unsafe { self.cursor.advance_unchecked() }
                                        }
                                        break;
                                    }
                                    Some(128..=255) => {
                                        if let Err(_) = self.cursor.advance_char() {
                                            return error!("Some UTF-8 error");
                                        }
                                    }
                                    _ => unsafe { self.cursor.advance_unchecked() },
                                }
                            }

                            Token::DocComment(unsafe {
                                // SAFETY: validated during loop.
                                from_utf8_unchecked(start.slice_to(end))
                            })
                        }
                        _ => Token::Symbol(Symbol::Slash),
                    }
                }
                b'%' => {
                    self.potential_markup = true;
                    Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals))
                }
                b'|' => {
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
                b'&' => {
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
                b'^' => {
                    self.potential_markup = true;

                    Token::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals))
                }
                b'(' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    Token::Symbol(Symbol::LeftParenthesis)
                }
                b')' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(Symbol::RightParenthesis)
                }
                b'[' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    Token::Symbol(Symbol::LeftBracket)
                }
                b']' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(Symbol::RightBracket)
                }
                b'{' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    Token::Symbol(Symbol::LeftBrace)
                }
                b'}' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(Symbol::RightBrace)
                }
                b'.' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(match self.cursor.peek() {
                        Some(b'.') => {
                            unsafe { self.cursor.advance_unchecked() };

                            match self.cursor.peek() {
                                Some(b'=') => {
                                    unsafe { self.cursor.advance_unchecked() };
                                    Symbol::DotDotEquals
                                }
                                Some(b'.') => {
                                    unsafe { self.cursor.advance_unchecked() };
                                    Symbol::DotDotDot
                                }
                                _ => Symbol::DotDot,
                            }
                        }
                        _ => Symbol::Dot,
                    })
                }
                b',' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    Token::Symbol(Symbol::Comma)
                }
                b';' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.potential_markup = true;

                    Token::Symbol(Symbol::Semicolon)
                }
                b':' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(match self.cursor.peek() {
                        Some(b':') => {
                            return error!(
                                "Encountered Rust-style path separator '::'. Use '.' instead"
                            )
                        }
                        _ => Symbol::Colon,
                    })
                }
                b'!' => {
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
                b'?' => {
                    unsafe { self.cursor.advance_unchecked() };

                    Token::Symbol(Symbol::QuestionMark)
                }
                b'\'' => {
                    unsafe { self.cursor.advance_unchecked() };

                    let char = match self.cursor.next_lfn() {
                        None => return error!("Expected character"),
                        Some(b'\\') => unescape_char(&mut self.cursor)?,
                        Some(b'\n') => {
                            return error!(
                                "Character literals cannot span multiple lines.\
                        If you meant to write a string, use a string literal \"\".\
                        If you meant to write the newline character, use '\\n' instead"
                            )
                        }
                        Some(char) => char.into(),
                    };

                    match self.cursor.next_lfn() {
                        Some(b'\'') => {}
                        _ => return error!("Unterminated character literal"),
                    }

                    Token::Char(char)
                }
                b'@' => {
                    unsafe { self.cursor.advance_unchecked() };
                    Token::Symbol(match self.cursor.peek() {
                        Some(b'!') => {
                            unsafe { self.cursor.advance_unchecked() };
                            Symbol::AtExclamationMark
                        }
                        _ => Symbol::At,
                    })
                }
                b'"' => {
                    unsafe { self.cursor.advance_unchecked() };
                    self.parse_string()?
                }
                b'A'..=b'Z' | b'_' | b'a'..=b'z' => {
                    let str = self.parse_id()?;

                    if let Some(kw) = KEYWORDS.get(str) {
                        Token::Keyword(*kw)
                    } else {
                        Token::Identifier(str)
                    }
                }
                _ => return error!("Encountered unexpected character"),
            };

            Ok(Some(Span {
                value: token,
                source: start..self.cursor.bytes_consumed() as Index,
            }))
        } else {
            Ok(None)
        }
    }

    /// ```text
    ///  v
    /// <tag...
    /// ```
    ///
    /// Assumes that the next byte is the id.
    pub fn parse_start_tag(&mut self) -> Result<Span<Token<'source>>, Error> {
        let start = self.cursor.bytes_consumed();

        self.skip_whitespace();

        let identifier = self.parse_id()?;

        if KEYWORDS.contains_key(identifier) {
            return error!("Cannot use a reserved keyword as a start tag identifier");
        }

        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

        Ok(Span {
            value: Token::MarkupStartTag(identifier),
            source: start as Index..self.cursor.bytes_consumed() as Index,
        })
    }

    pub fn parse_end_tag(&mut self) -> Result<Span<Token<'source>>, Error> {
        let start = self.cursor.bytes_consumed();

        self.skip_whitespace();

        let tag_name = self.parse_id()?;
        if KEYWORDS.contains_key(tag_name) {
            return error!("Cannot use a reserved keyword as an end tag identifier");
        }

        self.skip_whitespace();

        match self.cursor.next_lfn() {
            Some(b'>') => Ok(Span {
                value: Token::MarkupEndTag(tag_name),
                source: start as Index..self.cursor.bytes_consumed() as Index,
            }),
            _ => error!("Unterminated end tag"),
        }
    }
}

impl<'source, A: Allocator> FallibleIterator for Lexer<'source, A> {
    type Item = Span<Token<'source>>;
    type Error = Error;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        match self.layers.pop() {
            None => {
                if let Some(line_break) = self.skip_whitespace_line() {
                    return Ok(Some(line_break));
                }

                if self.potential_markup {
                    self.potential_markup = false;

                    if self.cursor.peek() == Some(b'<') {
                        unsafe { self.cursor.advance_unchecked() }
                        self.parse_start_tag().map(Some)
                    } else {
                        self.next_token_default()
                    }
                } else {
                    self.next_token_default()
                }
            }
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.skip_whitespace();

                let start = self.cursor.bytes_consumed() as Index;

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
                            _ => return error!("Unterminated self-closing tag"),
                        }
                    }
                    Some(char) if char.is_ascii_alphabetic() || char == b'_' => {
                        self.layers.push(Layer::Value);
                        Token::MarkupKey(self.parse_id()?)
                    }
                    _ => return error!("Expected '>' or '/' or a char"),
                };

                Ok(Some(Span {
                    value: token,
                    source: start..self.cursor.bytes_consumed() as Index,
                }))
            }
            Some(Layer::Value) => {
                self.skip_whitespace();

                match self.cursor.next_lfn() {
                    Some(b'=') => {}
                    _ => return error!("Expected '='"),
                }

                self.skip_whitespace();

                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                let start = self.cursor.bytes_consumed() as Index;

                let token = match self.cursor.next_lfn() {
                    Some(b'"') => self.parse_string()?,
                    Some(b'{') => {
                        self.layers.push(Layer::Insert);
                        self.potential_markup = true;
                        Token::Symbol(Symbol::LeftBrace)
                    }
                    _ => {
                        return error!(
                            "Expected a html-style value \"...\" or a JSX-style value {...}"
                        )
                    }
                };

                Ok(Some(Span {
                    value: token,
                    source: start..self.cursor.bytes_consumed() as Index,
                }))
            }
            Some(Layer::Insert) => {
                if let Some(line_break) = self.skip_whitespace_line() {
                    return Ok(Some(line_break));
                }

                let token = if self.potential_markup {
                    self.potential_markup = false;

                    if self.cursor.peek() == Some(b'<') {
                        unsafe { self.cursor.advance_unchecked() }
                        Some(self.parse_start_tag()?)
                    } else {
                        self.next_token_default()?
                    }
                } else {
                    self.next_token_default()?
                };

                match token.as_ref() {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftBrace),
                        ..
                    }) => {
                        self.layers.push(Layer::Insert);
                        self.layers.push(Layer::Insert);
                    }
                    Some(Span {
                        value: Token::Symbol(Symbol::RightBrace),
                        ..
                    }) => {}
                    _ => self.layers.push(Layer::Insert),
                }

                Ok(token)
            }
            Some(Layer::TextOrInsert) => {
                self.skip_whitespace();

                let source = self.cursor.bytes_consumed() as Index;
                let first = self.cursor.position();

                loop {
                    match self.cursor.peek() {
                        Some(b'<' | b'{') => break,
                        Some(_) => {
                            if let Err(_) = self.cursor.advance_char() {
                                return error!("UTF-8 error");
                            }
                        }
                        None => return error!("Markup element is unterminated"),
                    }
                }

                let source = source..self.cursor.bytes_consumed() as Index;

                let text = unsafe {
                    // SAFETY: UTF-8 validated in the loop.
                    from_utf8_unchecked(self.cursor.position().slice_to(first))
                };

                if text.len() == 0 {
                    // If there is no text, we have to yield something.

                    match self.cursor.next_lfn() {
                        // Yield an end tag or a nested element start
                        Some(b'<') => {
                            self.skip_whitespace();

                            match self.cursor.peek() {
                                None => error!("Unterminated start tag"),

                                // End tag
                                Some(b'/') => {
                                    unsafe { self.cursor.advance_unchecked() }
                                    self.parse_end_tag().map(Some)
                                }

                                // Nested element
                                Some(_) => {
                                    self.layers.push(Layer::TextOrInsert);
                                    self.parse_start_tag().map(Some)
                                }
                            }
                        }

                        // Yield an insert start
                        Some(b'{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;

                            let end = self.cursor.bytes_consumed() as Index;

                            Ok(Some(Span {
                                value: Token::Symbol(Symbol::LeftBrace),
                                source: end - 1..end,
                            }))
                        }

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unreachable!(),
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
                                None => return error!("Expected something"),

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
                        _ => unreachable!(),
                    }

                    Ok(Some(Span {
                        value: Token::MarkupText(text),
                        source,
                    }))
                }
            }
            Some(Layer::EndTag) => self.parse_end_tag().map(Some),
            Some(Layer::StartTag) => self.parse_start_tag().map(Some),
        }
    }
}
