use crate::chars::CharsIterator;
use crate::quark;
use crate::token::{Symbol, Token, WithSpan, KEYWORDS};
use std::fmt::Debug;
use std::str::Chars;

fn is_line_terminator(char: char) -> bool {
    match char {
        '\n' => true,
        '\r' => true,
        _ => false,
    }
}

#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter(char, UnexpectedCharacterError),
    UnexpectedEndOfInput,
    CannotUseKeywordAsTagNameForMarkupElement(String),
    UnknownEscapeCharacter(char),
}

#[derive(Debug)]
pub enum UnexpectedCharacterError {
    AsStartOfNewToken,
    InvalidAlphanumericCharacterWhileParsingNumber,
    ExpectedRightAngleWhileSelfClosingStartTag,
    ExpectedRightAngleOrSlashOrAlphabeticWhileLexingAttributes,
    ExpectedEqualsWhileLexingMarkupValue,
    ExpectedQuoteOrLeftBraceWhileLexingMarkupValue,
    ExpectedRightAngleWhileClosingEndTag,
    ExpectedSingleQuote
}

/// This enum describes the possibilities of the next token generated.
#[derive(Debug)]
pub enum Layer {
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    Insert,
    EndTag,
    StartTag
}

pub struct Lexer<'a> {
    chars: CharsIterator<'a>,
    /// This is needed for proper detection of markup
    layers: Vec<Layer>,
    potential_markup: bool
}

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: CharsIterator::new(chars),
            layers: Vec::new(),
            potential_markup: true,
        }
    }

    fn parse_number<const BASE: u32>(&mut self, mut number: f64) -> Result<Token, quark::Error> {
        loop {
            let next = match self.chars.next() {
                Some(next) => next,
                option => {
                    self.chars.rollback(option);
                    break;
                }
            };

            if let Some(to_add) = next.to_digit(BASE) {
                number = number * BASE as f64 + to_add as f64;
            } else if next == '.' {
                number = self.parse_number_tail::<BASE>(number)?;
                break;
            } else if next.is_alphanumeric() {
                return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                    next,
                    UnexpectedCharacterError::InvalidAlphanumericCharacterWhileParsingNumber,
                )));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(Token::Number(number))
    }

    fn parse_number_tail<const BASE: u32>(&mut self, mut number: f64) -> Result<f64, quark::Error> {
        let mut multiplier = 1_f64;
        let multiplier_multiplier = 1_f64 / BASE as f64;

        loop {
            let next = match self.chars.next() {
                Some(next) => next,
                option => {
                    self.chars.rollback(option);
                    break;
                }
            };

            if let Some(to_add) = next.to_digit(BASE) {
                multiplier *= multiplier_multiplier;
                number += to_add as f64 * multiplier;
            } else if next.is_alphanumeric() {
                return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                    next,
                    UnexpectedCharacterError::InvalidAlphanumericCharacterWhileParsingNumber,
                )));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(number)
    }

    fn parse_comment(&mut self) -> Result<String, quark::Error> {
        let mut comment = String::new();

        loop {
            match self.chars.next() {
                None => break,
                Some(char) if is_line_terminator(char) => break,
                Some(char) => comment.push(char),
            }
        }

        Ok(comment)
    }

    fn parse_identifier(&mut self) -> String {
        let mut identifier = String::new();

        loop {
            match self.chars.next() {
                Some(char) if char.is_alphanumeric() || char == '_' => identifier.push(char),
                option => {
                    self.chars.rollback(option);
                    break;
                }
            };
        }

        identifier
    }

    fn parse_string(&mut self) -> Result<String, quark::Error> {
        let mut s = String::new();

        loop {
            match self.chars.next() {
                None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                Some('"') => break,
                Some('\\') => s.push({
                    let next_char = self
                        .chars
                        .next()
                        .ok_or(quark::Error::Lexer(Error::UnexpectedEndOfInput))?;
                    self.chars.unescape_char(next_char)?
                }),
                Some(char) => s.push(char),
            }
        }

        Ok(s)
    }

    /// Expects whitespace and then the first char of the identifier.
    fn parse_end_tag(&mut self) -> Result<WithSpan<Token>, quark::Error> {
        let start = self.chars.index();

        self.chars.skip_white_space();

        let tag_name = self.parse_identifier();
        if KEYWORDS.contains_key(tag_name.as_str()) {
            return Err(quark::Error::Lexer(Error::CannotUseKeywordAsTagNameForMarkupElement(tag_name)));
        }

        self.chars.skip_white_space();

        match self.chars.next() {
            None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
            Some('>') => {}
            Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                char,
                UnexpectedCharacterError::ExpectedRightAngleWhileClosingEndTag,
            )))
        }

        Ok(WithSpan {
            start,
            value: Token::MarkupEndTag(tag_name),
            end: self.chars.index(),
        })
    }
    
    fn parse_start_tag(&mut self) -> Result<WithSpan<Token>, quark::Error> {
        let start = self.chars.index();
        
        self.chars.skip_white_space();

        let identifier = self.parse_identifier();
        if KEYWORDS.contains_key(identifier.as_str()) {
            return Err(quark::Error::Lexer(Error::CannotUseKeywordAsTagNameForMarkupElement(identifier)));
        }

        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

        Ok(WithSpan {
            start,
            value: Token::MarkupStartTag(identifier),
            end: self.chars.index(),
        })
    }
    
    pub fn next(&mut self) -> Result<WithSpan<Token>, quark::Error> {
        // println!("\nLAYERS: {:?}\nPOT_M: {}\n", self.layers, self.potential_markup);
        
        // Pop the current layer to be analyzed.
        match self.layers.pop() {
            None => self.next_default_context(),
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.chars.skip_white_space();

                Ok(WithSpan {
                    start: self.chars.index(),
                    value: match self.chars.next() {
                        // Start collecting children
                        Some('>') => {
                            self.layers.push(Layer::TextOrInsert);
                            Token::MarkupStartTagEnd
                        }

                        // Self closing tag
                        Some('/') => {
                            self.chars.skip_white_space();

                            match self.chars.next() {
                                Some('>') => Token::MarkupClose,
                                None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                                Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                                    char,
                                    UnexpectedCharacterError::ExpectedRightAngleWhileSelfClosingStartTag,
                                ))),
                            }
                        }
                        
                        // Key
                        Some(char) if char.is_alphabetic() => {
                            self.chars.rollback(Some(char));
                            self.layers.push(Layer::Value);
                            Token::MarkupKey(self.parse_identifier())
                        }
                        
                        // Reject other characters
                        Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                            char,
                            UnexpectedCharacterError::ExpectedRightAngleOrSlashOrAlphabeticWhileLexingAttributes,
                        ))),
                        None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                    },
                    end: self.chars.index(),
                })
            }
            Some(Layer::Value) => {
                self.chars.skip_white_space();

                match self.chars.next() {
                    Some('=') => {}
                    None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                    Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                        char,
                        UnexpectedCharacterError::ExpectedEqualsWhileLexingMarkupValue,
                    ))),
                }

                self.chars.skip_white_space();
                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                Ok(WithSpan {
                    start: self.chars.index(),
                    value: match self.chars.next() {
                        // HTML style attribute value
                        Some('"') => Token::String(self.parse_string()?),

                        // JSX style attribute value
                        Some('{') => {
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;

                            // Generate normal token
                            Token::Symbol(Symbol::LeftBrace)
                        },

                        // Reject other chars
                        Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                            char,
                            UnexpectedCharacterError::ExpectedQuoteOrLeftBraceWhileLexingMarkupValue,
                        ))),
                        None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                    },
                    end: self.chars.index(),
                })
            }
            Some(Layer::Insert) => {
                self.layers.push(Layer::Insert);
                
                // Generate normal token
                let token = self.next_default_context()?;
                
                match &token.value {
                    Token::Symbol(Symbol::LeftBrace) => {
                        // Push new.
                        self.layers.push(Layer::Insert);
                    },
                    Token::Symbol(Symbol::RightBrace) => {
                        self.layers.pop();
                    },
                    _ => {}
                }
                
                Ok(token)
            },
            Some(Layer::TextOrInsert) => {
                self.chars.skip_white_space();
                
                Ok(WithSpan {
                    start: self.chars.index(),
                    value: match self.chars.next() {
                        None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                        Some('<') => {
                            self.chars.skip_white_space();

                            match self.chars.next() {
                                None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),

                                // End tag
                                Some('/') => self.parse_end_tag()?.value,

                                // Nested element
                                option => {
                                    self.layers.push(Layer::TextOrInsert);
                                    self.chars.rollback(option);
                                    self.parse_start_tag()?.value
                                },
                            }
                        },
                        Some('{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;
                            
                            Token::Symbol(Symbol::LeftBrace)
                        }
                        Some(char) => {
                            self.layers.push(Layer::TextOrInsert);
                            let mut text = String::from(char);

                            loop {
                                match self.chars.next() {
                                    None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                                    Some('<') => {
                                        self.chars.skip_white_space();

                                        match self.chars.next() {
                                            None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                                            
                                            // End tag
                                            Some('/') => {
                                                // Switch context
                                                self.layers.pop();
                                                self.layers.push(Layer::EndTag);
                                            },

                                            // Nested element
                                            option => {
                                                self.chars.rollback(option);
                                                
                                                // Add context
                                                self.layers.push(Layer::StartTag);
                                            }
                                        }

                                        // Text may contain a trailing space.
                                        match text.pop() {
                                            None => unreachable!("Text \"{text}\" should not be empty!"),
                                            Some(' ') => {}
                                            Some(char) => text.push(char),
                                        }

                                        break;
                                    }
                                    Some(char) if char.is_whitespace() => {
                                        self.chars.skip_white_space();
                                        text.push(' ');
                                    }
                                    Some('{') => {
                                        self.chars.rollback(Some('{'));
                                        break;
                                    }
                                    Some(char) => text.push(char),
                                }
                            }

                            Token::MarkupText(text)
                        }
                    },
                    end: self.chars.index(),
                })
            },
            Some(Layer::EndTag) => self.parse_end_tag(),
            Some(Layer::StartTag) => self.parse_start_tag(),
        }
    }

    fn next_default_context(&mut self) -> Result<WithSpan<Token>, quark::Error> {
        self.chars.skip_white_space();
        
        if self.potential_markup {
            self.potential_markup = false;
            
            return match self.chars.next() {
                None => Ok(WithSpan {
                    start: self.chars.index(),
                    value: Token::EndOfInput,
                    end: self.chars.index(),
                }),
                Some('<') => self.parse_start_tag(),
                option => {
                    self.chars.rollback(option);
                    self.next_default_context()
                }
            }
        }
        
        macro_rules! opt_eq {
            ($symbol: expr, $eq: expr) => {
                match self.chars.next() {
                    Some('=') => $eq,
                    option => {
                        self.chars.rollback(option);
                        $symbol
                    }
                }
            };
        }

        let start = self.chars.index();
        
        let token = match self.chars.next() {
            None => {
                self.chars.rollback(None);
                Token::EndOfInput
            }
            Some('0') => match self.chars.next() {
                Some('x') => self.parse_number::<16>(0.)?,
                Some('o') => self.parse_number::<8>(0.)?,
                Some('b') => self.parse_number::<2>(0.)?,
                Some('_') => self.parse_number::<10>(0.)?,
                Some('.') => Token::Number(self.parse_number_tail::<10>(0.)?),
                Some('0') => self.parse_number::<10>(0.)?,
                Some('1') => self.parse_number::<10>(1.)?,
                Some('2') => self.parse_number::<10>(2.)?,
                Some('3') => self.parse_number::<10>(3.)?,
                Some('4') => self.parse_number::<10>(4.)?,
                Some('5') => self.parse_number::<10>(5.)?,
                Some('6') => self.parse_number::<10>(6.)?,
                Some('7') => self.parse_number::<10>(7.)?,
                Some('8') => self.parse_number::<10>(8.)?,
                Some('9') => self.parse_number::<10>(9.)?,
                option => {
                    self.chars.rollback(option);
                    Token::Number(0.)
                }
            },
            Some('1') => self.parse_number::<10>(1.)?,
            Some('2') => self.parse_number::<10>(2.)?,
            Some('3') => self.parse_number::<10>(3.)?,
            Some('4') => self.parse_number::<10>(4.)?,
            Some('5') => self.parse_number::<10>(5.)?,
            Some('6') => self.parse_number::<10>(6.)?,
            Some('7') => self.parse_number::<10>(7.)?,
            Some('8') => self.parse_number::<10>(8.)?,
            Some('9') => self.parse_number::<10>(9.)?,
            Some('=') => {
                self.potential_markup = true;

                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::EqualsEquals,
                    Some('>') => Symbol::EqualsRightAngle,
                    option => {
                        self.chars.rollback(option);
                        Symbol::Equals
                    }
                })
            }
            Some('<') => {
                self.potential_markup = true;

                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::LeftAngleEquals,
                    Some('<') => {
                        opt_eq!(Symbol::LeftAngleLeftAngle, Symbol::LeftAngleLeftAngleEquals)
                    }
                    option => {
                        self.chars.rollback(option);
                        Symbol::LeftAngle
                    }
                })
            }
            Some('>') => {
                self.potential_markup = true;

                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::RightAngleEquals,
                    Some('<') => opt_eq!(
                        Symbol::RightAngleRightAngle,
                        Symbol::RightAngleRightAngleEquals
                    ),
                    option => {
                        self.chars.rollback(option);
                        Symbol::RightAngle
                    }
                })
            }
            Some('+') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals))
            }
            Some('-') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Minus, Symbol::MinusEquals))
            }
            Some('*') => {
                self.potential_markup = true;
                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::StarEquals,
                    Some('*') => opt_eq!(Symbol::StarStar, Symbol::StarStarEquals),
                    option => {
                        self.chars.rollback(option);
                        Symbol::Star
                    }
                })
            }
            Some('/') => {
                self.potential_markup = true;

                match self.chars.next() {
                    Some('=') => Token::Symbol(Symbol::SlashEquals),
                    Some('/') => match self.chars.next() {
                        Some('/') => Token::DocComment(self.parse_comment()?),
                        option => {
                            self.chars.rollback(option);
                            Token::LineComment(self.parse_comment()?)
                        }
                    },
                    option => {
                        self.chars.rollback(option);
                        Token::Symbol(Symbol::Slash)
                    }
                }
            }
            Some('%') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals))
            }
            Some('|') => {
                self.potential_markup = true;
                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::PipeEquals,
                    Some('|') => opt_eq!(Symbol::PipePipe, Symbol::PipePipeEquals),
                    option => {
                        self.chars.rollback(option);
                        Symbol::Pipe
                    }
                })
            }
            Some('&') => {
                self.potential_markup = true;

                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::AmpersandEquals,
                    Some('&') => {
                        opt_eq!(Symbol::AmpersandAmpersand, Symbol::AmpersandAmpersandEquals)
                    }
                    option => {
                        self.chars.rollback(option);
                        Symbol::Ampersand
                    }
                })
            }
            Some('^') => {
                self.potential_markup = true;
                Token::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals))
            }
            Some('(') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::LeftParenthesis)
            }
            Some(')') => Token::Symbol(Symbol::RightParenthesis),
            Some('[') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::LeftBracket)
            }
            Some(']') => Token::Symbol(Symbol::RightBracket),
            Some('{') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::LeftBrace)
            }
            Some('}') => Token::Symbol(Symbol::RightBrace),
            Some('.') => Token::Symbol(Symbol::Dot),
            Some(',') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::Comma)
            },
            Some(';') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::Semicolon)
            }
            Some(':') => {
                self.potential_markup = true;
                Token::Symbol(Symbol::Colon)
            }
            Some('!') => match self.chars.next() {
                Some('=') => {
                    self.potential_markup = true;
                    Token::Symbol(Symbol::ExclamationMarkEquals)
                }
                option => {
                    self.chars.rollback(option);
                    Token::Symbol(Symbol::ExclamationMark)
                }
            },
            Some('?') => Token::Symbol(match self.chars.next() {
                Some('.') => Symbol::QuestionMarkDot,
                option => {
                    self.chars.rollback(option);
                    Symbol::QuestionMark
                }
            }),
            Some('\'') => Token::Char({
                let char = match self.chars.next() {
                    None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                    Some('\\') => {
                        let next_char = self
                            .chars
                            .next()
                            .ok_or(quark::Error::Lexer(Error::UnexpectedEndOfInput))?;
                        self.chars.unescape_char(next_char)?
                    }
                    Some(char) => char,
                };
                
                match self.chars.next() {
                    Some('\'') => {}
                    None => return Err(quark::Error::Lexer(Error::UnexpectedEndOfInput)),
                    Some(char) => return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                        char,
                        UnexpectedCharacterError::ExpectedSingleQuote
                    )))
                }
                
                char
            }),
            Some('"') => Token::String(self.parse_string()?),
            Some(char) if char.is_alphabetic() || char == '_' => {
                self.chars.rollback(Some(char));

                let identifier = self.parse_identifier();

                if let Ok(keyword) = KEYWORDS.get(identifier.as_str()).copied().ok_or(()) {
                    Token::Keyword(keyword)
                } else {
                    Token::Identifier(identifier)
                }
            }
            Some(char) => {
                return Err(quark::Error::Lexer(Error::UnexpectedCharacter(
                    char,
                    UnexpectedCharacterError::AsStartOfNewToken,
                )));
            }
        };

        Ok(WithSpan {
            start,
            end: self.chars.index(),
            value: token,
        })
    }
}
