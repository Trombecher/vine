use crate::ion;
use crate::token::{Keyword, Symbol, Token, TokenKind};
use std::str::Chars;

fn unescape_char(char: char) -> Option<char> {
    match char {
        '0' => Some('\0'),
        '\\' => Some('\\'),
        'f' => Some('\u{0c}'), // Form feed
        't' => Some('\t'), // Horizontal tab
        'r' => Some('\r'), // Carriage return
        'n' => Some('\n'), // Line feed / new line
        'b' => Some('\u{07}'), // Bell
        'v' => Some('\u{0b}'), // Vertical tab
        _ => None
    }
}

#[derive(Debug)]
pub enum Error {
    UnclosedString
}

pub struct Lexer<'a> {
    chars: Chars<'a>,
    force_take: Option<char>,
    index: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            force_take: None,
            index: 0,
        }
    }

    fn parse_number<const BASE: u32>(&mut self, number: f64) -> Result<TokenKind, ion::Error> {
        Ok(TokenKind::Number(0.))
    }

    fn next_char(&mut self) -> Option<char> {
        self.index += 1;
        self.force_take.take().or_else(|| self.chars.next())
    }

    fn rollback(&mut self, option: Option<char>) {
        self.index -= 1;
        self.force_take = option;
    }
    
    pub fn next(&mut self) -> Result<Token, ion::Error> {
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

            let kind = match self.next_char() {
                None => TokenKind::EndOfInput,
                Some('0') => match self.next_char() {
                    Some('x') => self.parse_number::<16>(0.)?,
                    Some('o') => self.parse_number::<8>(0.)?,
                    Some('b') => self.parse_number::<2>(0.)?,
                    Some('_') => self.parse_number::<10>(0.)?,
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
                        TokenKind::Number(0.)
                    },
                },
                Some('1') => self.parse_number::<10>(0.)?,
                Some('2') => self.parse_number::<10>(1.)?,
                Some('3') => self.parse_number::<10>(2.)?,
                Some('4') => self.parse_number::<10>(3.)?,
                Some('5') => self.parse_number::<10>(4.)?,
                Some('6') => self.parse_number::<10>(5.)?,
                Some('7') => self.parse_number::<10>(6.)?,
                Some('8') => self.parse_number::<10>(7.)?,
                Some('9') => self.parse_number::<10>(8.)?,
                Some('=') => TokenKind::Symbol(match self.next_char() {
                    Some('=') => Symbol::EqualsEquals,
                    Some('>') => Symbol::EqualsRightAngle,
                    option => {
                        self.rollback(option);
                        Symbol::Equals
                    }
                }),
                Some('<') => TokenKind::Symbol(match self.next_char() {
                    Some('=') => Symbol::LeftAngleEquals,
                    Some('<') => opt_eq!(Symbol::LeftAngleLeftAngle, Symbol::LeftAngleLeftAngleEquals),
                    option => {
                        self.rollback(option);
                        Symbol::LeftAngle
                    }
                }),
                Some('>') => TokenKind::Symbol(match self.next_char() {
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
                Some('+') => TokenKind::Symbol(opt_eq!(Symbol::Plus, Symbol::PlusEquals)),
                Some('-') => TokenKind::Symbol(opt_eq!(Symbol::Minus, Symbol::MinusEquals)),
                Some('*') => TokenKind::Symbol(match self.next_char() {
                    Some('=') => Symbol::StarEquals,
                    Some('*') => opt_eq!(Symbol::StarStar, Symbol::StarStarEquals),
                    option => {
                        self.rollback(option);
                        Symbol::Star
                    }
                }),
                Some('/') => TokenKind::Symbol(opt_eq!(Symbol::Slash, Symbol::SlashEquals)),
                Some('%') => TokenKind::Symbol(opt_eq!(Symbol::Percent, Symbol::PercentEquals)),
                Some('|') => TokenKind::Symbol(match self.next_char() {
                    Some('=') => Symbol::PipeEquals,
                    Some('|') => opt_eq!(Symbol::PipePipe, Symbol::PipePipeEquals),
                    option => {
                        self.rollback(option);
                        Symbol::Pipe
                    }
                }),
                Some('&') => TokenKind::Symbol(match self.next_char() {
                    Some('=') => Symbol::AmpersandEquals,
                    Some('&') => opt_eq!(Symbol::AmpersandAmpersand, Symbol::AmpersandAmpersandEquals),
                    option => {
                        self.rollback(option);
                        Symbol::Ampersand
                    }
                }),
                Some('^') => TokenKind::Symbol(opt_eq!(Symbol::Caret, Symbol::CaretEquals)),
                Some('(') => TokenKind::Symbol(Symbol::LeftParenthesis),
                Some(')') => TokenKind::Symbol(Symbol::RightParenthesis),
                Some('[') => TokenKind::Symbol(Symbol::LeftBracket),
                Some(']') => TokenKind::Symbol(Symbol::RightBracket),
                Some('{') => TokenKind::Symbol(Symbol::LeftBrace),
                Some('}') => TokenKind::Symbol(Symbol::RightBrace),
                Some('.') => TokenKind::Symbol(Symbol::Dot),
                Some(',') => TokenKind::Symbol(Symbol::Colon),
                Some(';') => TokenKind::Symbol(Symbol::Semicolon),
                Some(':') => TokenKind::Symbol(Symbol::Colon),
                Some('!') => todo!(),
                Some('?') => todo!(),
                Some('\'') => todo!(),
                Some('"') => {
                    let mut s = String::new();

                    loop {
                        match self.next_char() {
                            None => return Err(ion::Error::Lexer(Error::UnclosedString)),
                            Some('"') => break,
                            Some('\\') => {
                                if let Some(char) = unescape_char(self
                                    .next_char()
                                    .ok_or(ion::Error::Lexer(Error::UnclosedString))?) {
                                    s.push(char);
                                } else {
                                    return Err(ion::Error::Lexer(Error::UnclosedString))
                                }
                            }
                            Some(char) => s.push(char),
                        }
                    }
                    
                    TokenKind::String(s)
                }
                Some(char) if char.is_alphabetic() || char == '_' => {
                    let mut identifier = String::from(char);
                    loop {
                        match self.next_char() {
                            Some(char) if char.is_alphanumeric() || char == '_' => {
                                identifier.push(char)
                            }
                            option => {
                                self.rollback(option);
                                break
                            },
                        };
                    }

                    if let Ok(keyword) = Keyword::try_from(identifier.as_str()) {
                        TokenKind::Keyword(keyword)
                    } else {
                        TokenKind::Identifier(identifier)
                    }
                }
                Some(char) if char.is_whitespace() => continue,
                char => return Err(ion::Error::Lexer(todo!("{:?}", char))),
            };
            
            return Ok(Token {
                start,
                end: self.index,
                kind,
            });
        }
    }
}
