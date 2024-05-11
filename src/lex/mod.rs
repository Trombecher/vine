pub mod token;
pub mod error;
mod tests;
mod benchmark;

use std::hint::unreachable_unchecked;
use super::chars::Cursor;
use error::{Error, UnexpectedCharacterError, UnexpectedEndOfInputError};
use token::{KEYWORDS, Symbol, Token};
use super::Span;

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

impl<'a> Lexer<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self {
            cursor,
            potential_markup: false,
            layers: Vec::new(),
        }
    }
    
    fn unescape_char(&mut self) -> Result<char, crate::Error> {
        // TODO: unicode escapes (use of self)
        match self.cursor.next() {
            None => Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                UnexpectedEndOfInputError::EndTag
            ))),
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
            _ => Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                UnexpectedCharacterError::InvalidStringEscape
            ))),
        }
    }
    
    fn parse_number_dec(&mut self, mut number: f64) -> Result<Token<'a>, crate::Error> {
        loop {
            match self.cursor.next() {
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
                Some(b'.') => todo!("fp numbers"),
                Some(x) if x.is_ascii_alphabetic() => todo!(),
                Some(_) => {
                    self.cursor.rewind_ascii();
                    break
                }
                None => break,
            }
        }
        
        Ok(Token::Number(number))
    }
    
    /// Expects the next byte to be after the quote.
    fn parse_string(&mut self) -> Result<&'a str, crate::Error> {
        let mut recorder = self.cursor.begin_recording();

        loop {
            match recorder.cursor.peek_raw() {
                Some(b'"') => {
                    unsafe { recorder.cursor.advance_unchecked(); }
                    break Ok(recorder.stop())
                },
                Some(b'\\') => {
                    unsafe { recorder.cursor.advance_unchecked(); }
                    if let None = recorder.cursor.peek_raw() {
                        todo!()
                    }
                    unsafe { recorder.cursor.advance_unchecked(); }
                }
                None => todo!(),
                Some(_) => if let Err(e) = recorder.cursor.advance_char() {
                    break Err(e)
                }
            }
        }
    }
    
    fn parse_id(&mut self) -> Result<&'a str, crate::Error> {
        let recorder = self.cursor.begin_recording();

        loop {
            match recorder.cursor.next() {
                None => todo!(),
                Some(x) if x.is_ascii_alphanumeric() || x == b'_' => {}
                Some(128..=255) => todo!(),
                _ => {
                    recorder.cursor.rewind_ascii();
                    break
                },
            }
        }

        Ok(recorder.stop())
    }
    
    fn parse_comment(&mut self) -> Result<&'a str, crate::Error> {
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
    
    /// Assumes that the next byte is `<`.
    pub fn parse_start_tag(&mut self) -> Result<Span<Token<'a>>, crate::Error> {
        let start = self.cursor.index();

        unsafe { self.cursor.advance_unchecked(); }
        
        self.skip_white_space();
        
        let identifier = self.parse_id()?;
        if KEYWORDS.contains_key(identifier) {
            return Err(crate::Error::Lexer(Error::CannotUseKeywordAsTagName));
        }
        
        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);
        
        Ok(Span {
            value: Token::MarkupStartTag(identifier),
            start,
            end: self.cursor.index(),
        })
    }
    
    pub fn parse_end_tag(&mut self) -> Result<Span<Token<'a>>, crate::Error> {
        let start = self.cursor.index();
        
        self.skip_white_space();
        
        let tag_name = self.parse_id()?;
        if KEYWORDS.contains_key(tag_name) {
            return Err(crate::Error::Lexer(Error::CannotUseKeywordAsTagName));
        }
        
        self.skip_white_space();
        
        match self.cursor.next() {
            Some(b'>') => {}
            _ => todo!()
        }
        
        Ok(Span {
            value: Token::MarkupEndTag(tag_name),
            start,
            end: self.cursor.index(),
        })
    }
    
    pub fn next(&mut self) -> Result<Span<Token<'a>>, crate::Error> {
        match self.layers.pop() {
            None => {
                // Skip whitespace
                self.skip_white_space();
                
                if self.potential_markup {
                    self.potential_markup = false;
                    
                    if self.cursor.peek_raw() == Some(b'<') {
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
                                _ => todo!()
                            }
                        }
                        Some(char) if char.is_ascii_alphabetic() || char == b'_' => {
                            self.layers.push(Layer::Value);
                            Token::MarkupKey(self.parse_id()?)
                        }
                        _ => todo!()
                    },
                    end: self.cursor.index(),
                })
            },
            Some(Layer::Value) => {
                self.skip_white_space();
                
                match self.cursor.next() {
                    Some(b'=') => {}
                    _ => todo!()
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
                        _ => todo!()
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
                
                let mut text = self.cursor.begin_recording();
                loop {
                    match text.cursor.peek_raw() {
                        Some(b'<' | b'{') => break,
                        Some(_) => unsafe { text.cursor.advance_char()?; }
                        None => todo!(),
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
                                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                                    UnexpectedEndOfInputError::NestedMarkupStart,
                                ))),

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
                                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                                    UnexpectedEndOfInputError::NestedMarkupStart,
                                ))),

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
    
    pub fn next_default_context(&mut self) -> Result<Span<Token<'a>>, crate::Error> {
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
                    Some(b'.') => todo!("fp numbers"), // Token::Number(self.parse_number_tail::<10>(0.)?),
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
                self.potential_markup = true;
                
                Token::Symbol(Symbol::Colon)
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
                    None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                        UnexpectedEndOfInputError::CharLiteralContent
                    ))),
                    Some(b'\\') => {
                        self.unescape_char()?
                    }
                    Some(char) => char.into(),
                };

                match self.cursor.next() {
                    Some(b'\'') => {}
                    None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                        UnexpectedEndOfInputError::CharLiteralQuote
                    ))),
                    _ => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                        UnexpectedCharacterError::CharLiteralQuote
                    )))
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
            Some(x) => todo!("Char: {}", x)
        };
        
        Ok(Span {
            start,
            value: token,
            end: self.cursor.index(),
        })
    }
}