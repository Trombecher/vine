use crate::ion;
use crate::token::{Symbol, Token, WithSpan, KEYWORDS};
use std::fmt::Debug;
use std::str::Chars;
use crate::chars::CharsIterator;

fn is_line_terminator(char: char) -> bool {
    match char {
        '\n' => true,
        '\r' => true,
        _ => false,
    }
}

#[derive(Copy, Clone)]
pub enum Context {
    Default,
    PotentialMarkup,
    Markup {
        element_depth: usize,
        context: MarkupContext,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum MarkupContext {
    Attributes,
    Value,
    InsertValue(usize),
    TextOrInsert,
    Insert(usize),
}

#[derive(Debug)]
pub enum Error {
    UnclosedChar,
    UnclosedString,
    UnknownEscapeCharacter,
    UnexpectedCharacter,
    ExpectedIdentifierFoundKeyword,
    UnclosedMarkupElement,
    CannotUseKeywordAsTagName,
    CannotUseKeywordAsKey,
    TagNamesDoNotMatch,
}

pub struct Lexer<'a> {
    chars: CharsIterator<'a>,
    /// This is needed for proper detection of markup
    context: Context,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars: CharsIterator::new(chars),
            context: Context::PotentialMarkup,
        }
    }

    fn parse_number<const BASE: u32>(&mut self, mut number: f64) -> Result<Token, ion::Error> {
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
                return Err(ion::Error::Lexer(Error::UnexpectedCharacter));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(Token::Number(number))
    }

    fn parse_number_tail<const BASE: u32>(&mut self, mut number: f64) -> Result<f64, ion::Error> {
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
                return Err(ion::Error::Lexer(Error::UnexpectedCharacter));
            } else if next != '_' {
                self.chars.rollback(Some(next));
                break;
            }
        }

        Ok(number)
    }

    fn parse_comment(&mut self) -> Result<String, ion::Error> {
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

    fn parse_string(&mut self) -> Result<String, ion::Error> {
        let mut s = String::new();

        loop {
            match self.chars.next() {
                None => return Err(ion::Error::Lexer(Error::UnclosedString)),
                Some('"') => break,
                Some('\\') => s.push({
                    let next_char = self.chars
                        .next()
                        .ok_or(ion::Error::Lexer(Error::UnclosedString))?;
                    self.chars.unescape_char(next_char)?
                }),
                Some(char) => s.push(char),
            }
        }

        Ok(s)
    }

    pub fn next(&mut self) -> Result<WithSpan<Token>, ion::Error> {
        match &mut self.context {
            Context::Default => self.next_default_context(),
            Context::PotentialMarkup => self.next_potential_markup(),
            Context::Markup {
                context,
                element_depth,
            } => {
                match context {
                    MarkupContext::Attributes => {
                        self.chars.skip_white_space();
                        
                        let start = self.chars.index();
                        
                        let token = match self.chars.next() {
                            None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                            Some('>') => Token::MarkupStartTagEnd,
                            Some('/') => {
                                self.chars.skip_white_space();
                                
                                match self.chars.next() {
                                    Some('>') => {
                                        if *element_depth == 0 {
                                            self.context = Context::Default;
                                        }
                                        Token::MarkupClose
                                    },
                                    None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                                    _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter))
                                }
                            },
                            Some(char) if char.is_alphabetic() => {
                                self.chars.rollback(Some(char));
                                *context = MarkupContext::Value;
                                Token::MarkupKey(self.parse_identifier())
                            }
                            _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter))
                        };
                        
                        return Ok(WithSpan {
                            start,
                            end: self.chars.index(),
                            value: token,
                        });
                    },
                    MarkupContext::Value => {
                        self.chars.skip_white_space();
                        
                        match self.chars.next() {
                            Some('=') => {}
                            None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                            _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter))
                        }
                        
                        self.chars.skip_white_space();
                        
                        *context = MarkupContext::Attributes;

                        Ok(WithSpan {
                            start: self.chars.index(),
                            value: match self.chars.next() {
                                None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                                Some('"') => Token::String(self.parse_string()?),
                                Some('{') => todo!("insert values"),
                                _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter))
                            },
                            end: self.chars.index(),
                        })
                    },
                    MarkupContext::InsertValue(_) => todo!(),
                    MarkupContext::TextOrInsert => todo!(),
                    MarkupContext::Insert(_) => todo!(),
                }
            }
        }
    }

    fn next_potential_markup(&mut self) -> Result<WithSpan<Token>, ion::Error> {
        self.chars.skip_white_space();
        
        return Ok(match self.chars.next() {
            None => WithSpan {
                start: self.chars.index(),
                value: Token::EndOfInput,
                end: self.chars.index(),
            },
            Some('<') => {
                self.chars.skip_white_space();

                let start = self.chars.index();
                
                let identifier = self.parse_identifier();
                if KEYWORDS.contains_key(identifier.as_str()) {
                    return Err(ion::Error::Lexer(Error::CannotUseKeywordAsTagName));
                }

                self.context = Context::Markup {
                    element_depth: 0,
                    context: MarkupContext::Attributes,
                };

                WithSpan {
                    start,
                    value: Token::MarkupStartTag(identifier),
                    end: self.chars.index(),
                }
            }
            option => {
                self.chars.rollback(option);
                self.context = Context::Default;
                return self.next_default_context();
            }
        });
    }

    fn next_default_context(&mut self) -> Result<WithSpan<Token>, ion::Error> {
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

        loop {
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
                    self.context = Context::PotentialMarkup;

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
                    self.context = Context::PotentialMarkup;

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
                    self.context = Context::PotentialMarkup;

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
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals))
                }
                Some('-') => {
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(opt_eq!(Symbol::Minus, Symbol::MinusEquals))
                }
                Some('*') => {
                    self.context = Context::PotentialMarkup;
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
                    self.context = Context::PotentialMarkup;

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
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals))
                },
                Some('|') => {
                    self.context = Context::PotentialMarkup;
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
                    self.context = Context::PotentialMarkup;
                    
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
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals))
                },
                Some('(') => {
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(Symbol::LeftParenthesis)
                }
                Some(')') => Token::Symbol(Symbol::RightParenthesis),
                Some('[') => {
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(Symbol::LeftBracket)
                }
                Some(']') => Token::Symbol(Symbol::RightBracket),
                Some('{') => {
                    self.context = Context::PotentialMarkup;
                    Token::Symbol(Symbol::LeftBrace)
                }
                Some('}') => Token::Symbol(Symbol::RightBrace),
                Some('.') => Token::Symbol(Symbol::Dot),
                Some(',') => Token::Symbol(Symbol::Colon),
                Some(';') => Token::Symbol(Symbol::Semicolon),
                Some(':') => Token::Symbol(Symbol::Colon),
                Some('!') => match self.chars.next() {
                    Some('=') => {
                        self.context = Context::PotentialMarkup;
                        Token::Symbol(Symbol::CaretEquals)
                    }
                    option => {
                        self.chars.rollback(option);
                        Token::Symbol(Symbol::Caret)
                    }
                },
                Some('?') => Token::Symbol(match self.chars.next() {
                    Some('.') => Symbol::QuestionMarkDot,
                    option => {
                        self.chars.rollback(option);
                        Symbol::QuestionMark
                    }
                }),
                Some('\'') => Token::Char(match self.chars.next() {
                    None => return Err(ion::Error::Lexer(Error::UnclosedChar)),
                    Some('\\') => {
                        let next_char = self.chars
                            .next()
                            .ok_or(ion::Error::Lexer(Error::UnclosedString))?;
                        self.chars.unescape_char(next_char)?
                    }
                    Some(char) => char,
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
                Some(char) if char.is_whitespace() => continue,
                _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
            };

            return Ok(WithSpan {
                start,
                end: self.chars.index(),
                value: token,
            });
        }
    }
}
