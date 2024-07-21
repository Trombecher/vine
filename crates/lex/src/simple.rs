use std::intrinsics::transmute;
use std::ptr::slice_from_raw_parts;
use std::str::from_raw_parts;
use parse_tools::bytes::Cursor;

use error::Error;

use crate::{Span, try_to_hex};
use crate::token::{KEYWORDS, Symbol, Token};

pub(crate) struct SimpleLexer<'a> {
    pub(crate) cursor: Cursor<'a>,
    pub(crate) potential_markup: bool,
}

impl<'a> SimpleLexer<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            cursor,
            potential_markup: false,
        }
    }

    fn unescape_char(&mut self) -> Result<char, Error> {
        match self.cursor.next_lfn() {
            None => Err(Error::E0013),
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
                match self.cursor.next_lfn() {
                    None => Err(Error::E0021),
                    Some(x) => match try_to_hex(x) {
                        None => Err(Error::E0022),
                        Some(8..) => Err(Error::E0023),
                        Some(x) => match self.cursor.next_lfn() {
                            None => Err(Error::E0024),
                            Some(y) => match try_to_hex(y) {
                                None => Err(Error::E0025),
                                Some(y) => Ok(((x << 4) + y) as char),
                            },
                        },
                    },
                }
            }
            Some(b'u') => {
                // Unicode code point
                match self.cursor.next_lfn() {
                    Some(b'{') => {}
                    _ => todo!(),
                }

                let code_point = 0_u32;

                Ok(char::try_from(code_point).map_err(|_| todo!())?)
            }
            _ => Err(Error::E0014),
        }
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
                    number = number * 10. + x as f64
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
                    
                    break Some(unsafe {
                        Span::from_ends(
                            Token::LineBreak,
                            first,
                            self.cursor.cursor()
                        )
                    })
                }
                Some(b'\n') => {
                    let source = unsafe {
                        from_raw_parts(self.cursor.cursor(), 1)
                    };
                    
                    unsafe { self.cursor.advance_unchecked() }
                    
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

    pub fn next_token(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
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

        let token = match self.cursor.peek() {
            None => Token::EndOfInput,
            Some(b'\r') => {
                unsafe { self.cursor.advance_unchecked() }
                if let Some(b'\n') = self.cursor.peek() {}
                Token::LineBreak
            }
            Some(b'\n') => {
                unsafe { self.cursor.advance_unchecked() }
                Token::LineBreak
            }
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
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(d as f64)?
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
                    Some(b'\\') => self.unescape_char()?,
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
}