pub mod token;
pub mod error;
mod tests;

use std::hint::unreachable_unchecked;
use std::mem::transmute;
use std::ptr::slice_from_raw_parts;
use super::chars::Cursor;
use token::{KEYWORDS, Symbol, Token};
use crate::lex::token::TokenIterator;
use super::{Error, Span};

pub enum Layer {
    /// This layer expects `key=`, `/>` or `>`.
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    EndTag,
    Insert,
    StartTag
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    potential_markup: bool,
    layers: Vec<Layer>
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
            cursor,
            potential_markup: false,
            layers: Vec::new(),
        }
    }
    
    fn unescape_char(&mut self) -> Result<char, Error> {
        match self.cursor.next() {
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
            Some(b'x') => {             // Hexadecimal code
                match self.cursor.next() {
                    None => Err(Error::E0021),
                    Some(x) => match try_to_hex(x) {
                        None => Err(Error::E0022),
                        Some(8..) => Err(Error::E0023),
                        Some(x) => {
                            match self.cursor.next() {
                                None => Err(Error::E0024),
                                Some(y) => match try_to_hex(y) {
                                    None => Err(Error::E0025),
                                    Some(y) => Ok(((x << 4) + y) as char)
                                }
                            }
                        }
                    }
                }
            }
            Some(b'u') => {             // Unicode code point
                match self.cursor.next() {
                    None => todo!(),
                    Some(b'{') => {}
                    _ => todo!(),
                }
                
                let code_point = 0_u32;
                
                Ok(char::try_from(code_point).map_err(|_| todo!())?)
            }
            _ => Err(Error::E0014),
        }
    }
    
    #[inline]
    fn parse_number_dec(&mut self, mut number: f64) -> Result<Token<'a>, Error> {
        loop {
            match self.cursor.next_raw() {
                Some(b'_') => {}
                Some(b'0') => number *= 10.,
                Some(b'1') => number = number * 10. + 1.,
                Some(b'2') => number = number * 10. + 2.,
                Some(b'3') => number = number * 10. + 3.,
                Some(b'4') => number = number * 10. + 4.,
                Some(b'5') => number = number * 10. + 5.,
                Some(b'6') => number = number * 10. + 6.,
                Some(b'7') => number = number * 10. + 7.,
                Some(b'8') => number = number * 10. + 8.,
                Some(b'9') => number = number * 10. + 9.,
                Some(b'.') => {
                    number = self.parse_number_dec_tail(number)?;
                    break
                },
                Some(x) if x.is_ascii_alphabetic() => return Err(Error::E0017),
                Some(_) => {
                    self.cursor.rewind_u8();
                    break
                }
                None => break,
            }
        }
        
        Ok(Token::Number(number))
    }
    
    #[inline]
    fn parse_number_dec_tail(&mut self, mut number: f64) -> Result<f64, Error> {
        match self.cursor.next_raw() {
            Some(b'0') => {},
            Some(b'1') => number += 0.1,
            Some(b'2') => number += 0.2,
            Some(b'3') => number += 0.3,
            Some(b'4') => number += 0.4,
            Some(b'5') => number += 0.5,
            Some(b'6') => number += 0.6,
            Some(b'7') => number += 0.7,
            Some(b'8') => number += 0.8,
            Some(b'9') => number += 0.9,
            _ => {
                self.cursor.rewind_u8();
                self.cursor.rewind_u8();

                return Ok(number)
            },
        }

        let mut multiplier = 0.1;

        loop {
            number += match self.cursor.next() {
                Some(b'_') => continue,
                Some(b'0') => {
                    multiplier *= 0.1;
                    continue
                },
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
    fn parse_string(&mut self) -> Result<String, Error> {
        let mut s = String::with_capacity(32);

        loop {
            match self.cursor.peek_raw() {
                Some(b'"') => {
                    unsafe { self.cursor.advance_unchecked(); }
                    break Ok(s)
                },
                Some(b'\\') => {
                    unsafe { self.cursor.advance_unchecked(); }
                    s.push(self.unescape_char()?);
                }
                None => break Err(Error::E0019),
                Some(_) => {
                    let start = self.cursor.cursor();
                    
                    if let Err(e) = self.cursor.advance_char() {
                        break Err(e)
                    }
                    
                    unsafe {
                        s.push_str(transmute(slice_from_raw_parts(start, self.cursor.cursor().sub_ptr(start))))
                    }
                }
            }
        }
    }
    
    fn parse_id(&mut self) -> Result<&'a str, Error> {
        let recorder = self.cursor.begin_recording();

        loop {
            match recorder.cursor.next() {
                Some(x) if x.is_ascii_alphanumeric() || x == b'_' => {}
                Some(128..=255) => return Err(Error::E0020),
                None => break,
                _ => {
                    recorder.cursor.rewind_ascii();
                    break
                },
            }
        }

        Ok(recorder.stop())
    }
    
    fn parse_comment(&mut self) -> Result<&'a str, Error> {
        let recorder = self.cursor.begin_recording();

        loop {
            match recorder.cursor.next() {
                None | Some(b'\n') => break,
                Some(128..=255) => {
                    recorder.cursor.rewind_ascii();
                    recorder.cursor.advance_char()?;
                }
                _ => {}
            }
        }
        
        Ok(recorder.stop())
    }
    
    /// Assumes that the next byte is the id.
    pub fn parse_start_tag(&mut self) -> Result<Span<Token<'a>>, Error> {
        let start = self.cursor.index();
        
        self.skip_white_space();
        
        let identifier = self.parse_id()?;
        if KEYWORDS.contains_key(identifier) {
            return Err(Error::E0015);
        }
        
        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);
        
        Ok(Span {
            value: Token::MarkupStartTag(identifier),
            start,
            end: self.cursor.index(),
        })
    }
    
    pub fn parse_end_tag(&mut self) -> Result<Span<Token<'a>>, Error> {
        let start = self.cursor.index();
        
        self.skip_white_space();
        
        let tag_name = self.parse_id()?;
        if KEYWORDS.contains_key(tag_name) {
            return Err(Error::E0016);
        }
        
        self.skip_white_space();
        
        match self.cursor.next() {
            Some(b'>') => Ok(Span {
                value: Token::MarkupEndTag(tag_name),
                start,
                end: self.cursor.index(),
            }),
            _ => Err(Error::E0026)
        }
    }
    
    fn skip_white_space(&mut self) {
        loop {
            match self.cursor.peek_raw() {
                Some(x) if x.is_ascii_whitespace() => {
                    unsafe { self.cursor.advance_unchecked() }
                }
                _ => break,
            }
        }
    }
    
    pub fn next_default_context(&mut self) -> Result<Span<Token<'a>>, Error> {
        macro_rules! opt_eq {
            ($symbol: expr, $eq: expr) => {{
                unsafe { self.cursor.advance_unchecked() };
                match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        $eq
                    },
                    _ => $symbol
                }
            }};
        }

        let start = self.cursor.index();
        
        let token = match self.cursor.peek_raw() {
            None => Token::EndOfInput,
            Some(b'0') => {
                unsafe { self.cursor.advance_unchecked() };
                
                match self.cursor.next() {
                    Some(b'x') => todo!("hex numbers"), // self.parse_number::<16>(0.)?,
                    Some(b'o') => todo!("octal numbers"), // self.parse_number::<8>(0.)?,
                    Some(b'b') => todo!("binary numbers"), // self.parse_number::<2>(0.)?,
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
                    Some(_) => {
                        self.cursor.rewind_ascii();
                        Token::Number(0.)
                    }
                    None => Token::Number(0.)
                }
            }
            Some(b'1') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(1.)?
            },
            Some(b'2') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(2.)?
            }
            Some(b'3') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(3.)?
            }
            Some(b'4') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(4.)?
            }
            Some(b'5') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(5.)?
            }
            Some(b'6') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(6.)?
            }
            Some(b'7') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(7.)?
            }
            Some(b'8') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(8.)?
            }
            Some(b'9') => {
                unsafe { self.cursor.advance_unchecked() };
                self.parse_number_dec(9.)?
            }
            Some(b'<') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::LeftAngleEquals
                    },
                    Some(b'<') => opt_eq!(Symbol::LeftAngleLeftAngle, Symbol::LeftAngleLeftAngleEquals),
                    _ => Symbol::LeftAngle
                })
            }
            Some(b'>') => {
                unsafe { self.cursor.advance_unchecked() };
                
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::RightAngleEquals
                    },
                    Some(b'>') => opt_eq!(Symbol::RightAngleRightAngle, Symbol::RightAngleRightAngleEquals),
                    _ => Symbol::RightAngle
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
                
                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::MinusEquals
                    },
                    Some(b'>') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::MinusRightAngle
                    }
                    _ => Symbol::Minus
                })
            }
            Some(b'*') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;
                
                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::StarEquals
                    },
                    Some(b'*') => opt_eq!(Symbol::StarStar, Symbol::StarStarEquals),
                    _ => Symbol::Star,
                })
            }
            Some(b'/') => {
                self.potential_markup = true;
                
                match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Token::Symbol(Symbol::SlashEquals)
                    },
                    Some(b'/') => {
                        unsafe { self.cursor.advance_unchecked() };
                        
                        match self.cursor.peek_raw() {
                            Some(b'/') => {
                                unsafe { self.cursor.advance_unchecked() };
                                Token::DocComment(self.parse_comment()?)
                            },
                            _ => Token::LineComment(self.parse_comment()?)
                        }
                    },
                    _ => Token::Symbol(Symbol::Slash)
                }
            }
            Some(b'%') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals))
            }
            Some(b'|') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;
                
                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::PipeEquals
                    }
                    Some(b'|') => opt_eq!(Symbol::PipePipe, Symbol::PipePipeEquals),
                    _ => Symbol::Pipe
                })
            }
            Some(b'&') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;

                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::AmpersandEquals
                    }
                    Some(b'&') => opt_eq!(Symbol::AmpersandAmpersand, Symbol::AmpersandAmpersandEquals),
                    _ => Symbol::Ampersand
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
            },
            Some(b')') => {
                unsafe { self.cursor.advance_unchecked() };
                
                Token::Symbol(Symbol::RightParenthesis)
            },
            Some(b'[') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;
                
                Token::Symbol(Symbol::LeftBracket)
            },
            Some(b']') => {
                unsafe { self.cursor.advance_unchecked() };
                
                Token::Symbol(Symbol::RightBracket)
            },
            Some(b'{') => {
                unsafe { self.cursor.advance_unchecked() };
                self.potential_markup = true;
                
                Token::Symbol(Symbol::LeftBrace)
            },
            Some(b'}') => {
                unsafe { self.cursor.advance_unchecked() };
                
                Token::Symbol(Symbol::RightBrace)
            },
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

                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b':') => return Err(Error::E0027),
                    _ => Symbol::Colon
                })
            }
            Some(b'!') => {
                unsafe { self.cursor.advance_unchecked() };
                
                match self.cursor.peek_raw() {
                    Some(b'=') => {
                        unsafe { self.cursor.advance_unchecked() };
                        self.potential_markup = true;
                        
                        Token::Symbol(Symbol::ExclamationMarkEquals)
                    }
                    _ => Token::Symbol(Symbol::ExclamationMark)
                }
            },
            Some(b'?') => {
                unsafe { self.cursor.advance_unchecked() };
                
                Token::Symbol(match self.cursor.peek_raw() {
                    Some(b'.') => {
                        unsafe { self.cursor.advance_unchecked() };
                        Symbol::QuestionMarkDot
                    },
                    _ => Symbol::QuestionMark
                })
            },
            Some(b'\'') => {
                unsafe { self.cursor.advance_unchecked() };
                
                let char = match self.cursor.next() {
                    None => return Err(Error::E0030),
                    Some(b'\\') => {
                        self.unescape_char()?
                    }
                    Some(char) => char.into(),
                };

                match self.cursor.next() {
                    Some(b'\'') => {}
                    None => return Err(Error::E0031),
                    _ => return Err(Error::E0032)
                }

                Token::Char(char)
            },
            Some(b'@') => {
                unsafe { self.cursor.advance_unchecked() };
                Token::Symbol(Symbol::At)
            },
            Some(b'"') => {
                unsafe { self.cursor.advance_unchecked() };
                Token::String(self.parse_string()?)
            },
            Some(b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
                let str = self.parse_id()?;
                
                if let Some(kw) = KEYWORDS.get(str) {
                    Token::Keyword(*kw)
                } else {
                    Token::Identifier(str)
                }
            }
            Some(_) => return Err(Error::E0028)
        };
        
        Ok(Span {
            start,
            value: token,
            end: self.cursor.index(),
        })
    }
}

impl<'a> TokenIterator<'a> for Lexer<'a> {
    fn next_token(&mut self) -> Result<Span<Token<'a>>, Error> {
        match self.layers.pop() {
            None => {
                // Skip whitespace
                self.skip_white_space();

                if self.potential_markup {
                    self.potential_markup = false;

                    if self.cursor.peek_raw() == Some(b'<') {
                        unsafe { self.cursor.advance_unchecked() }
                        self.parse_start_tag()
                    } else {
                        self.next_default_context()
                    }
                } else {
                    self.next_default_context()
                }
            },
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.skip_white_space();

                Ok(Span {
                    start: self.cursor.index(),
                    value: match self.cursor.peek_raw() {
                        Some(b'>') => {
                            unsafe { self.cursor.advance_unchecked(); }
                            self.layers.push(Layer::TextOrInsert);
                            Token::MarkupStartTagEnd
                        }
                        Some(b'/') => {
                            unsafe { self.cursor.advance_unchecked(); }
                            self.skip_white_space();

                            match self.cursor.next() {
                                Some(b'>') => Token::MarkupClose,
                                _ => return Err(Error::E0033)
                            }
                        }
                        Some(char) if char.is_ascii_alphabetic() || char == b'_' => {
                            self.layers.push(Layer::Value);
                            Token::MarkupKey(self.parse_id()?)
                        }
                        _ => return Err(Error::E0034)
                    },
                    end: self.cursor.index(),
                })
            },
            Some(Layer::Value) => {
                self.skip_white_space();

                match self.cursor.next() {
                    Some(b'=') => {}
                    _ => return Err(Error::E0035)
                }

                self.skip_white_space();

                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                Ok(Span {
                    start: self.cursor.index(),
                    value: match self.cursor.next() {
                        Some(b'"') => Token::String(self.parse_string()?),
                        Some(b'{') => {
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;
                            Token::Symbol(Symbol::LeftBrace)
                        }
                        _ => return Err(Error::E0036)
                    },
                    end: self.cursor.index(),
                })
            },
            Some(Layer::Insert) => {
                let token = self.next_default_context()?;

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
                self.skip_white_space();

                let start = self.cursor.index();

                let text = self.cursor.begin_recording();
                loop {
                    match text.cursor.peek_raw() {
                        Some(b'<' | b'{') => break,
                        Some(_) => text.cursor.advance_char()?,
                        None => return Err(Error::E0037),
                    }
                }
                let text = text.stop();

                Ok(if text.len() == 0 {
                    // If there is no text, we have to yield something.

                    match self.cursor.next() {
                        // Yield an end tag or a nested element start
                        Some(b'<') => {
                            self.skip_white_space();

                            match self.cursor.peek_raw() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe { self.cursor.advance_unchecked() }
                                    self.parse_end_tag()?
                                },

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
                            self.potential_markup = true;

                            Span {
                                start: self.cursor.index() - 1,
                                value: Token::Symbol(Symbol::LeftBrace),
                                end: self.cursor.index(),
                            }
                        }

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() }
                    }
                } else {
                    // There is some text, so we yield the text and prepare the next layer state.

                    match self.cursor.peek_raw() {
                        Some(b'<') => {
                            unsafe { self.cursor.advance_unchecked(); }

                            self.skip_white_space();

                            match self.cursor.peek_raw() {
                                None => return Err(Error::E0038),

                                // End tag
                                Some(b'/') => {
                                    unsafe { self.cursor.advance_unchecked(); }
                                    self.layers.push(Layer::EndTag)
                                },

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
                        _ => unsafe { unreachable_unchecked() }
                    }

                    Span {
                        start,
                        value: Token::MarkupText(text),
                        end: self.cursor.index(),
                    }
                })
            },
            Some(Layer::EndTag) => self.parse_end_tag(),
            Some(Layer::StartTag) => self.parse_start_tag(),
        }
    }
}