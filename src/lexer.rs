use crate::markup::Child;
use crate::token::{Symbol, Token, WithSpan, KEYWORDS};
use crate::{ion, markup};
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
    chars: Chars<'a>,
    force_take: Option<char>,
    index: usize,
    /// This is needed for proper detection of markup
    potential_markup: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            force_take: None,
            index: 0,
            potential_markup: false,
        }
    }

    fn unescape_char(&mut self, char: char) -> Result<char, ion::Error> {
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
            _ => Err(ion::Error::Lexer(Error::UnknownEscapeCharacter)),
        }
    }

    fn parse_number<const BASE: u32>(&mut self, mut number: f64) -> Result<Token, ion::Error> {
        loop {
            let next = match self.next_char() {
                Some(next) => next,
                option => {
                    self.rollback(option);
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
                self.rollback(Some(next));
                break;
            }
        }

        Ok(Token::Number(number))
    }

    fn parse_number_tail<const BASE: u32>(&mut self, mut number: f64) -> Result<f64, ion::Error> {
        let mut multiplier = 1_f64;
        let multiplier_multiplier = 1_f64 / BASE as f64;

        loop {
            let next = match self.next_char() {
                Some(next) => next,
                option => {
                    self.rollback(option);
                    break;
                }
            };

            if let Some(to_add) = next.to_digit(BASE) {
                multiplier *= multiplier_multiplier;
                number += to_add as f64 * multiplier;
            } else if next.is_alphanumeric() {
                return Err(ion::Error::Lexer(Error::UnexpectedCharacter));
            } else if next != '_' {
                self.rollback(Some(next));
                break;
            }
        }

        Ok(number)
    }

    fn next_char(&mut self) -> Option<char> {
        self.index += 1;
        self.force_take.take().or_else(|| self.chars.next())
    }

    fn rollback(&mut self, option: Option<char>) {
        self.index -= 1;
        self.force_take = option;
    }

    fn parse_comment(&mut self) -> Result<String, ion::Error> {
        let mut comment = String::new();

        loop {
            match self.next_char() {
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
            match self.next_char() {
                Some(char) if char.is_alphanumeric() || char == '_' => identifier.push(char),
                option => {
                    self.rollback(option);
                    break;
                }
            };
        }

        identifier
    }

    fn skip_white_space(&mut self) {
        loop {
            match self.next_char() {
                Some(char) if char.is_whitespace() => {}
                option => {
                    self.rollback(option);
                    break;
                }
            }
        }
    }

    /// Assumes that `<` was already consumed.
    fn parse_markup_element(&mut self) -> Result<markup::Element, ion::Error> {
        let identifier = WithSpan {
            start: self.index,
            value: {
                let identifier = self.parse_identifier();
                if KEYWORDS.contains_key(identifier.as_str()) {
                    return Err(ion::Error::Lexer(Error::CannotUseKeywordAsTagName));
                }
                identifier
            },
            end: self.index,
        };

        let mut attributes = Vec::new();

        loop {
            self.skip_white_space();

            let key = WithSpan {
                start: self.index,
                value: match self.next_char() {
                    Some('>') => break,
                    Some('/') => {
                        return match self.next_char() {
                            None => Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                            Some('>') => Ok(markup::Element {
                                children: Vec::new(),
                                identifier,
                                attributes,
                            }),
                            Some(_) => Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
                        }
                    }
                    Some('{') => todo!("inserting"),
                    Some(char) if char.is_alphabetic() => {
                        self.rollback(Some(char));
                        let identifier = self.parse_identifier();
                        if KEYWORDS.contains_key(identifier.as_str()) {
                            return Err(ion::Error::Lexer(Error::CannotUseKeywordAsKey));
                        }
                        identifier
                    }
                    Some(_) => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
                    None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                },
                end: self.index,
            };

            self.skip_white_space();

            match self.next_char() {
                Some('=') => {}
                _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)), // TODO: Simple boolean attributes (?)
            }

            self.skip_white_space();

            let value = WithSpan {
                start: self.index,
                value: match self.next_char() {
                    Some('"') => vec![Token::String(self.parse_string()?)],
                    Some('{') => todo!("JSX style attribute"),
                    _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
                },
                end: self.index,
            };

            attributes.push((key, value));
        }

        let mut children = Vec::new();

        // Parse children
        loop {
            self.skip_white_space();

            children.push(WithSpan {
                start: self.index,
                value: match self.next_char() {
                    None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                    Some('<') => {
                        self.skip_white_space();

                        match self.next_char() {
                            None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
                            Some('/') => break,
                            option => {
                                self.rollback(option);
                                Child::Element(self.parse_markup_element()?)
                            }
                        }
                    }
                    Some(char) => {
                        let mut text = String::from(char);

                        // Collect Text
                        loop {
                            match self.next_char() {
                                None => {
                                    return Err(ion::Error::Lexer(Error::UnclosedMarkupElement))
                                }
                                Some('<') => {
                                    self.rollback(Some('<'));
                                    break;
                                }
                                Some(char) => text.push(char),
                            }
                        }

                        // Remove trailing whitespace
                        loop {
                            if let Some(last) = text.pop() {
                                if !last.is_whitespace() {
                                    text.push(last);
                                    break;
                                }
                            } else {
                                break;
                            }
                        }

                        Child::Text(text)
                    }
                },
                end: self.index,
            });
        }

        self.skip_white_space();

        let closing_identifier = self.parse_identifier();
        if identifier.value != closing_identifier {
            return Err(ion::Error::Lexer(Error::TagNamesDoNotMatch));
        }

        self.skip_white_space();

        match self.next_char() {
            None => return Err(ion::Error::Lexer(Error::UnclosedMarkupElement)),
            Some('>') => {}
            Some(_) => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
        }

        Ok(markup::Element {
            children,
            attributes,
            identifier,
        })
    }

    fn parse_string(&mut self) -> Result<String, ion::Error> {
        let mut s = String::new();

        loop {
            match self.next_char() {
                None => return Err(ion::Error::Lexer(Error::UnclosedString)),
                Some('"') => break,
                Some('\\') => s.push({
                    let next_char = self
                        .next_char()
                        .ok_or(ion::Error::Lexer(Error::UnclosedString))?;
                    self.unescape_char(next_char)?
                }),
                Some(char) => s.push(char),
            }
        }

        Ok(s)
    }

    pub fn next(&mut self) -> Result<WithSpan<Token>, ion::Error> {
        macro_rules! opt_eq {
            ($symbol: expr, $eq: expr) => {
                match self.next_char() {
                    Some('=') => $eq,
                    option => {
                        self.rollback(option);
                        $symbol
                    }
                }
            };
        }

        loop {
            let start = self.index;

            let (token, pot) = match self.next_char() {
                None => (
                    {
                        self.rollback(None);
                        Token::EndOfInput
                    },
                    false
                ),
                Some('0') => (
                    match self.next_char() {
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
                            self.rollback(option);
                            Token::Number(0.)
                        }
                    },
                    false,
                ),
                Some('1') => (self.parse_number::<10>(1.)?, false),
                Some('2') => (self.parse_number::<10>(2.)?, false),
                Some('3') => (self.parse_number::<10>(3.)?, false),
                Some('4') => (self.parse_number::<10>(4.)?, false),
                Some('5') => (self.parse_number::<10>(5.)?, false),
                Some('6') => (self.parse_number::<10>(6.)?, false),
                Some('7') => (self.parse_number::<10>(7.)?, false),
                Some('8') => (self.parse_number::<10>(8.)?, false),
                Some('9') => (self.parse_number::<10>(9.)?, false),
                Some('=') => (
                    Token::Symbol(match self.next_char() {
                        Some('=') => Symbol::EqualsEquals,
                        Some('>') => Symbol::EqualsRightAngle,
                        option => {
                            self.rollback(option);
                            Symbol::Equals
                        }
                    }),
                    true,
                ),
                Some('<') => {
                    if self.potential_markup {
                        (Token::MarkupElement(self.parse_markup_element()?), false)
                    } else {
                        (
                            Token::Symbol(match self.next_char() {
                                Some('=') => Symbol::LeftAngleEquals,
                                Some('<') => opt_eq!(
                                    Symbol::LeftAngleLeftAngle,
                                    Symbol::LeftAngleLeftAngleEquals
                                ),
                                option => {
                                    self.rollback(option);
                                    Symbol::LeftAngle
                                }
                            }),
                            true,
                        )
                    }
                }
                Some('>') => (
                    Token::Symbol(match self.next_char() {
                        Some('=') => Symbol::RightAngleEquals,
                        Some('<') => opt_eq!(
                            Symbol::RightAngleRightAngle,
                            Symbol::RightAngleRightAngleEquals
                        ),
                        option => {
                            self.rollback(option);
                            Symbol::RightAngle
                        }
                    }),
                    true,
                ),
                Some('+') => (
                    Token::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals)),
                    true,
                ),
                Some('-') => (
                    Token::Symbol(opt_eq!(Symbol::Minus, Symbol::MinusEquals)),
                    true,
                ),
                Some('*') => (
                    Token::Symbol(match self.next_char() {
                        Some('=') => Symbol::StarEquals,
                        Some('*') => opt_eq!(Symbol::StarStar, Symbol::StarStarEquals),
                        option => {
                            self.rollback(option);
                            Symbol::Star
                        }
                    }),
                    true,
                ),
                Some('/') => (
                    match self.next_char() {
                        Some('=') => Token::Symbol(Symbol::SlashEquals),
                        Some('/') => match self.next_char() {
                            Some('/') => Token::DocComment(self.parse_comment()?),
                            option => {
                                self.rollback(option);
                                Token::LineComment(self.parse_comment()?)
                            }
                        },
                        option => {
                            self.rollback(option);
                            Token::Symbol(Symbol::Slash)
                        }
                    },
                    true,
                ),
                Some('%') => (
                    Token::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals)),
                    true,
                ),
                Some('|') => (
                    Token::Symbol(match self.next_char() {
                        Some('=') => Symbol::PipeEquals,
                        Some('|') => opt_eq!(Symbol::PipePipe, Symbol::PipePipeEquals),
                        option => {
                            self.rollback(option);
                            Symbol::Pipe
                        }
                    }),
                    true,
                ),
                Some('&') => (
                    Token::Symbol(match self.next_char() {
                        Some('=') => Symbol::AmpersandEquals,
                        Some('&') => {
                            opt_eq!(Symbol::AmpersandAmpersand, Symbol::AmpersandAmpersandEquals)
                        }
                        option => {
                            self.rollback(option);
                            Symbol::Ampersand
                        }
                    }),
                    true,
                ),
                Some('^') => (
                    Token::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals)),
                    true,
                ),
                Some('(') => (Token::Symbol(Symbol::LeftParenthesis), false),
                Some(')') => (Token::Symbol(Symbol::RightParenthesis), false),
                Some('[') => (Token::Symbol(Symbol::LeftBracket), true),
                Some(']') => (Token::Symbol(Symbol::RightBracket), false),
                Some('{') => (Token::Symbol(Symbol::LeftBrace), true),
                Some('}') => (Token::Symbol(Symbol::RightBrace), false),
                Some('.') => (Token::Symbol(Symbol::Dot), false),
                Some(',') => (Token::Symbol(Symbol::Colon), true),
                Some(';') => (Token::Symbol(Symbol::Semicolon), true),
                Some(':') => (Token::Symbol(Symbol::Colon), true),
                Some('!') => match self.next_char() {
                    Some('=') => (Token::Symbol(Symbol::CaretEquals), true),
                    option => {
                        self.rollback(option);
                        (Token::Symbol(Symbol::Caret), false)
                    }
                },
                Some('?') => (
                    Token::Symbol(match self.next_char() {
                        Some('.') => Symbol::QuestionMarkDot,
                        option => {
                            self.rollback(option);
                            Symbol::QuestionMark
                        }
                    }),
                    false,
                ),
                Some('\'') => {
                    let char = match self.next_char() {
                        None => return Err(ion::Error::Lexer(Error::UnclosedChar)),
                        Some('\\') => {
                            let next_char = self
                                .next_char()
                                .ok_or(ion::Error::Lexer(Error::UnclosedString))?;
                            self.unescape_char(next_char)?
                        }
                        Some(char) => char,
                    };

                    (Token::Char(char), false)
                }
                Some('"') => (Token::String(self.parse_string()?), false),
                Some(char) if char.is_alphabetic() || char == '_' => {
                    self.rollback(Some(char));
                    let identifier = self.parse_identifier();

                    (
                        if let Ok(keyword) = KEYWORDS.get(identifier.as_str()).copied().ok_or(()) {
                            Token::Keyword(keyword)
                        } else {
                            Token::Identifier(identifier)
                        },
                        false,
                    )
                }
                Some(char) if char.is_whitespace() => continue,
                _ => return Err(ion::Error::Lexer(Error::UnexpectedCharacter)),
            };

            self.potential_markup = pot;

            return Ok(WithSpan {
                start,
                end: self.index,
                value: token,
            });
        }
    }
}
