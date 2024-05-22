

use chars::CharsIterator;
use std::fmt::Debug;
use std::hint::unreachable_unchecked;
use std::str::Chars;
use token::{Symbol, Token, KEYWORDS};

use error::*;
use crate::Span;

fn is_line_terminator(char: char) -> bool {
    match char {
        '\n' | '\r' => true,
        _ => false,
    }
}

/// This enum describes the possibilities of the next token generated.
#[derive(Debug)]
pub enum Layer {
    KeyOrStartTagEndOrSelfClose,
    Value,
    TextOrInsert,
    Insert,
    EndTag,
    StartTag,
}

pub struct Lexer<'s> {
    chars: CharsIterator<'s>,
    /// This is needed for proper detection of markup
    layers: Vec<Layer>,
    potential_markup: bool,
}

impl<'s> Lexer<'s> {
    #[inline]
    pub fn new(chars: Chars<'s>) -> Self {
        Self {
            chars: chars.into(),
            layers: Vec::new(),
            potential_markup: true,
        }
    }
    
    #[inline]
    pub fn chars(&self) -> &CharsIterator<'s> {
        &self.chars
    }
    
    pub fn collect(mut self) -> Result<Vec<Span<Token<'s>>>, crate::Error> {
        let mut tokens = Vec::new();

        loop {
            let next_token = self.next()?;
            if let Token::EndOfInput = next_token.value {
                break;
            }

            tokens.push(next_token);
        }
        
        Ok(tokens)
    }
    
    fn parse_number<const BASE: u32>(&mut self, mut number: f64) -> Result<Token<'s>, crate::Error> {
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
                return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                    UnexpectedCharacterError::Number
                )));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(Token::Number(number))
    }

    fn parse_number_tail<const BASE: u32>(&mut self, mut number: f64) -> Result<f64, crate::Error> {
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
                return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                    UnexpectedCharacterError::NumberTail
                )));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(number)
    }

    #[inline]
    fn parse_comment(&mut self) -> &'s str {
        let mut x = self.chars.begin_extraction();
        x.skip_until(|c| is_line_terminator(c));
        x.finish()
    }

    #[inline]
    fn parse_identifier(&mut self) -> &'s str {
        let mut x = self.chars.begin_extraction();
        x.skip_until(|c| !(c.is_alphanumeric() || c == '_'));
        x.finish()
    }

    fn parse_string(&mut self) -> Result<&'s str, crate::Error> {
        let mut x = self.chars.begin_extraction();
        
        loop {
            match x.next() {
                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                    UnexpectedEndOfInputError::String
                ))),
                Some('"') => break,
                Some('\\') => if let None = x.next() {
                    return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                        UnexpectedCharacterError::StringEscape
                    )))
                },
                _ => {},
            }
        }

        Ok(x.finish())
    }

    /// Expects whitespace and then the first char of the identifier.
    fn parse_end_tag(&mut self) -> Result<Span<Token<'s>>, crate::Error> {
        let start = self.chars.index();

        self.chars.skip_white_space();

        let tag_name = self.parse_identifier();
        if KEYWORDS.contains_key(tag_name) {
            return Err(crate::Error::Lexer(Error::CannotUseKeywordAsTagName));
        }

        self.chars.skip_white_space();

        match self.chars.next() {
            None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                UnexpectedEndOfInputError::EndTag
            ))),
            Some('>') => {}
            _ => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                UnexpectedCharacterError::EndTag
            )))
        }

        Ok(Span {
            start,
            value: Token::MarkupEndTag(tag_name),
            end: self.chars.index(),
        })
    }

    fn parse_start_tag(&mut self) -> Result<Span<Token<'s>>, crate::Error> {
        let start = self.chars.index();

        self.chars.skip_white_space();

        let identifier = self.parse_identifier();
        if KEYWORDS.contains_key(identifier) {
            return Err(crate::Error::Lexer(Error::CannotUseKeywordAsTagName));
        }

        self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

        Ok(Span {
            start,
            value: Token::MarkupStartTag(identifier),
            end: self.chars.index(),
        })
    }

    pub fn next(&mut self) -> Result<Span<Token<'s>>, crate::Error> {
        // Pop the current layer to be analyzed.
        match self.layers.pop() {
            None => self.next_default_context(),
            Some(Layer::KeyOrStartTagEndOrSelfClose) => {
                self.chars.skip_white_space();

                Ok(Span {
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
                                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                                    UnexpectedEndOfInputError::SelfClosingStartTag
                                ))),
                                _ => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                                    UnexpectedCharacterError::SelfClosingStartTag,
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
                        Some(_) => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                            UnexpectedCharacterError::StartTagKeys
                        ))),
                        None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                            UnexpectedEndOfInputError::StartTagKeys
                        ))),
                    },
                    end: self.chars.index(),
                })
            }
            Some(Layer::Value) => {
                self.chars.skip_white_space();

                match self.chars.next() {
                    Some('=') => {}
                    None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                        UnexpectedEndOfInputError::MarkupEquals
                    ))),
                    _ => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                        UnexpectedCharacterError::MarkupEquals
                    ))),
                }

                self.chars.skip_white_space();
                self.layers.push(Layer::KeyOrStartTagEndOrSelfClose);

                Ok(Span {
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
                        }

                        // Reject other chars
                        Some(_) => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                            UnexpectedCharacterError::MarkupValue,
                        ))),
                        None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                            UnexpectedEndOfInputError::MarkupValue
                        ))),
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
                    }
                    Token::Symbol(Symbol::RightBrace) => {
                        self.layers.pop();
                    }
                    _ => {}
                }

                Ok(token)
            }
            Some(Layer::TextOrInsert) => {
                self.chars.skip_white_space();

                let mut x = self.chars.begin_extraction();
                x.skip_until(|c| c == '<' || c == '=');
                let text = x.finish();
                
                let token = if text.len() == 0 {
                    // If there is no text, we have to yield something.
                    
                    match self.chars.next() {
                        None => todo!(),
                        
                        // Yield an end tag or a nested element start
                        Some('<') => {
                            self.chars.skip_white_space();
                            
                            match self.chars.next() {
                                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                                    UnexpectedEndOfInputError::NestedMarkupStart,
                                ))),
                                
                                // End tag
                                Some('/') => self.parse_end_tag()?.value,
                                
                                // Nested element
                                option => {
                                    self.layers.push(Layer::TextOrInsert);
                                    self.chars.rollback(option);
                                    self.parse_start_tag()?.value
                                }
                            }
                        }
                        
                        // Yield an insert start
                        Some('{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.layers.push(Layer::Insert);
                            self.potential_markup = true;

                            Token::Symbol(Symbol::LeftBrace)
                        }

                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() }
                    }
                } else {
                    // There is some text, so we yield the text and prepare the next layer state.
                    
                    match self.chars.next() {
                        None => todo!(),
                        Some('<') => {
                            self.chars.skip_white_space();

                            match self.chars.next() {
                                None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                                    UnexpectedEndOfInputError::NestedMarkupStart,
                                ))),
                                
                                // End tag
                                Some('/') => self.layers.push(Layer::EndTag),

                                // Nested element
                                option => {
                                    self.chars.rollback(option);

                                    // Add context
                                    self.layers.push(Layer::TextOrInsert);
                                    self.layers.push(Layer::StartTag);
                                }
                            }
                        }
                        Some('{') => {
                            self.layers.push(Layer::TextOrInsert);
                            self.chars.rollback(Some('{'))
                        },
                        
                        // SAFETY: Unreachable, since `x.skip_until(...)` leaves the next char as None, '<' or '{'.
                        _ => unsafe { unreachable_unchecked() }
                    }
                    
                    Token::MarkupText(text)
                };
                
                Ok(Span {
                    start: self.chars.index(),
                    value: token,
                    end: self.chars.index(),
                })
            }
            Some(Layer::EndTag) => self.parse_end_tag(),
            Some(Layer::StartTag) => self.parse_start_tag(),
        }
    }

    fn next_default_context(&mut self) -> Result<Span<Token<'s>>, crate::Error> {
        self.chars.skip_white_space();

        if self.potential_markup {
            self.potential_markup = false;

            return match self.chars.next() {
                None => Ok(Span {
                    start: self.chars.index(),
                    value: Token::EndOfInput,
                    end: self.chars.index(),
                }),
                Some('<') => self.parse_start_tag(),
                option => {
                    self.chars.rollback(option);
                    self.next_default_context()
                }
            };
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
                Token::Symbol(opt_eq!(Symbol::Equals, Symbol::EqualsEquals))
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
                    Some('>') => opt_eq!(
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
                Token::Symbol(match self.chars.next() {
                    Some('=') => Symbol::MinusEquals,
                    Some('>') => Symbol::MinusRightAngle,
                    option => {
                        self.chars.rollback(option);
                        Symbol::Minus
                    }
                })
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
                        Some('/') => Token::DocComment(self.parse_comment()),
                        option => {
                            self.chars.rollback(option);
                            Token::LineComment(self.parse_comment())
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
            }
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
                    None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                        UnexpectedEndOfInputError::CharLiteralContent
                    ))),
                    Some('\\') => {
                        let next_char = self.chars
                            .next()
                            .ok_or(crate::Error::Lexer(Error::UnexpectedCharacter(
                                UnexpectedCharacterError::CharLiteralEscape
                            )))?;
                        self.chars.unescape_char(next_char)?
                    }
                    Some(char) => char,
                };

                match self.chars.next() {
                    Some('\'') => {}
                    None => return Err(crate::Error::Lexer(Error::UnexpectedEndOfInput(
                        UnexpectedEndOfInputError::CharLiteralQuote
                    ))),
                    _ => return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                        UnexpectedCharacterError::CharLiteralQuote
                    )))
                }

                char
            }),
            Some('@') => Token::Symbol(Symbol::At),
            Some('"') => Token::String(self.parse_string()?),
            Some(char) if char.is_alphabetic() || char == '_' => {
                self.chars.rollback(Some(char));

                let identifier = self.parse_identifier();
                
                if let Ok(keyword) = KEYWORDS.get(identifier).copied().ok_or(()) {
                    Token::Keyword(keyword)
                } else {
                    Token::Identifier(identifier)
                }
            }
            Some(_) => {
                return Err(crate::Error::Lexer(Error::UnexpectedCharacter(
                    UnexpectedCharacterError::Token
                )));
            }
        };

        Ok(Span {
            start,
            end: self.chars.index(),
            value: token,
        })
    }
}
