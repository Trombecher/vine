use crate::token::{IdentifierOrKeyword, Keyword, Symbol, Token, WithSpan};
use crate::{ion, markup};
use std::str::Chars;

fn unescape_char(char: char) -> Option<char> {
    match char {
        '0' => Some('\0'),
        '\\' => Some('\\'),
        'f' => Some('\u{0c}'), // Form feed
        't' => Some('\t'),     // Horizontal tab
        'r' => Some('\r'),     // Carriage return
        'n' => Some('\n'),     // Line feed / new line
        'b' => Some('\u{07}'), // Bell
        'v' => Some('\u{0b}'), // Vertical tab
        '"' => Some('"'),
        '\'' => Some('\''),
        '[' => Some('\u{1B}'), // Escape
        _ => None,
    }
}

fn is_line_terminator(char: char) -> bool {
    match char {
        '\n' => true,
        '\r' => true,
        _ => false,
    }
}

#[derive(Debug)]
pub enum Error {
    UnclosedString,
    UnexpectedCharacter,
    ExpectedIdentifierFoundKeyword,
    UnclosedMarkupElement,
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

    fn parse_identifier_or_keyword(&mut self) -> IdentifierOrKeyword {
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

        if let Ok(keyword) = Keyword::try_from(identifier.as_str()) {
            IdentifierOrKeyword::Keyword(keyword)
        } else {
            IdentifierOrKeyword::Identifier(identifier)
        }
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
            value: match self.parse_identifier_or_keyword() {
                IdentifierOrKeyword::Identifier(id) => id,
                IdentifierOrKeyword::Keyword(_) => {
                    return Err(ion::Error::Lexer(Error::ExpectedIdentifierFoundKeyword))
                }
            },
            end: self.index,
        };

        let mut attributes = Vec::new();

        loop {
            self.skip_white_space();

            match self
                .next_char()
                .ok_or(ion::Error::Lexer(Error::UnclosedMarkupElement))?
            {
                '>' => break,
                '/' => {
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
                '{' => todo!("inserting"),
                char if char.is_alphabetic() => {
                    todo!("key")
                }
                _ => {}
            }

            break;
        }

        let mut children = Vec::new();

        Ok(markup::Element {
            children,
            attributes,
            identifier,
        })
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
                None => (Token::EndOfInput, false),
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
                Some('!') => todo!(),
                Some('?') => todo!(),
                Some('\'') => todo!(),
                Some('"') => (
                    {
                        let mut s = String::new();

                        loop {
                            match self.next_char() {
                                None => return Err(ion::Error::Lexer(Error::UnclosedString)),
                                Some('"') => break,
                                Some('\\') => {
                                    if let Some(char) = unescape_char(
                                        self.next_char()
                                            .ok_or(ion::Error::Lexer(Error::UnclosedString))?,
                                    ) {
                                        s.push(char);
                                    } else {
                                        return Err(ion::Error::Lexer(Error::UnclosedString));
                                    }
                                }
                                Some(char) => s.push(char),
                            }
                        }

                        Token::String(s)
                    },
                    false,
                ),
                Some(char) if char.is_alphabetic() || char == '_' => {
                    self.rollback(Some(char));
                    (
                        match self.parse_identifier_or_keyword() {
                            IdentifierOrKeyword::Identifier(id) => Token::Identifier(id),
                            IdentifierOrKeyword::Keyword(kw) => Token::Keyword(kw),
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
