#[cfg(test)]
mod tests;

use core::str;

use parser_tools::PeekableChars;

use crate::tokens::{CharacterSource, NumberSource, Token, WhitespaceSource};

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier_continuation(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_token_start(c: Option<char>) -> bool {
    match c {
        Some(c) if is_identifier_start(c) => true,
        Some(c) if c.is_whitespace() => true,
        Some(
            '$'
            | '%'
            | '§'
            | '?'
            | '~'
            | '`'
            | ':'
            | '+'
            | '-'
            | '*'
            | '/'
            | '!'
            | '.'
            | ','
            | '('
            | ')'
            | '{'
            | '}'
            | '['
            | ']'
            | '<'
            | '>'
            | '|'
            | '&'
            | '^'
            | '@'
            | ';'
            | '='
            | '"'
            | '\''
            | '0'..='9',
        )
        | None => true,
        _ => false,
    }
}

pub struct Lexer<'input> {
    chars: PeekableChars<'input>,
}

impl<'input> Lexer<'input> {
    #[inline]
    #[must_use]
    pub fn new(s: &'input str) -> Self {
        Self {
            chars: PeekableChars::new(s.chars()),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.chars.as_str().as_ptr();

        macro_rules! span {
            () => {
                str::from_raw_parts(
                    start,
                    self.chars.as_str().as_ptr().offset_from_unsigned(start),
                )
            };
        }

        Some(match self.chars.next()? {
            '$' => Token::DollarSign,
            '%' => Token::Percent,
            '§' => Token::Paragraph,
            '?' => Token::QuestionMark,
            '~' => Token::Tilde,
            '`' => Token::Backtick,
            ':' => Token::Colon,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '!' => Token::ExclamationMark,
            '.' => Token::Period,
            ',' => Token::Comma,
            '(' => Token::OpeningParenthesis,
            ')' => Token::ClosingParenthesis,
            '{' => Token::OpeningBrace,
            '}' => Token::ClosingBrace,
            '[' => Token::OpeningBracket,
            ']' => Token::ClosingBracket,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '|' => Token::Bar,
            '&' => Token::Ampersand,
            '^' => Token::Caret,
            '@' => Token::At,
            ';' => Token::Semicolon,
            '=' => Token::Equals,
            '/' => {
                match self.chars.peek() {
                    Some('/') => {
                        // Line comment
                        self.chars.next();

                        // Skip until line break.
                        loop {
                            match self.chars.peek() {
                                Some('\r' | '\n') | None => {
                                    break Token::Comment(unsafe { span!() });
                                }
                                _ => {
                                    self.chars.next();
                                }
                            }
                        }
                    }
                    Some('*') => {
                        // block comment
                        self.chars.next();

                        let mut depth = 0_u64;

                        loop {
                            match self.chars.next() {
                                Some('/') if let Some('*') = self.chars.peek() => {
                                    self.chars.next();

                                    if let Some(higher_depth) = depth.checked_add(1) {
                                        depth = higher_depth;
                                    } else {
                                        unreachable!("maximum depth reached")
                                    }
                                }
                                Some('*') if let Some('/') = self.chars.peek() => {
                                    self.chars.next();

                                    if let Some(lower_depth) = depth.checked_sub(1) {
                                        depth = lower_depth;
                                    } else {
                                        break Token::Comment(unsafe { span!() });
                                    }
                                }
                                None => break Token::Invalid(unsafe { span!() }),
                                _ => {}
                            }
                        }
                    }
                    _ => Token::Slash,
                }
            }
            '#' => {
                // Python-style comment

                loop {
                    match self.chars.peek() {
                        Some('\n' | '\r') | None => break,
                        _ => {
                            self.chars.next();
                        }
                    }
                }

                Token::Comment(unsafe { span!() })
            }
            '\'' => {
                match self.chars.next() {
                    Some('\\') => todo!("escape in char literal"),
                    Some('\'') | None => return Some(Token::Invalid(unsafe { span!() })),
                    Some(_) => {}
                };

                match self.chars.next() {
                    Some('\'') => {
                        Token::Character(unsafe { CharacterSource::new_unchecked(span!()) })
                    }
                    // TODO: maybe capture everything in '...'
                    _ => return Some(Token::Invalid(unsafe { span!() })),
                }
            }
            c if is_identifier_start(c) => {
                while self.chars.peek().is_some_and(is_identifier_continuation) {
                    self.chars.next();
                }

                Token::IdentifierOrKeyword(unsafe { span!() })
            }
            c if c.is_whitespace() => {
                // Skip whitespace

                while self.chars.peek().is_some_and(char::is_whitespace) {
                    self.chars.next();
                }

                Token::Whitespace(unsafe { WhitespaceSource::new_unchecked(span!()) })
            }
            '0'..='9' => {
                while let Some('0'..='9' | '_') = self.chars.peek() {
                    self.chars.next();
                }

                if let Some('.') = self.chars.peek() {
                    self.chars.next();

                    while let Some('0'..='9' | '_') = self.chars.peek() {
                        self.chars.next();
                    }
                }

                Token::Number(unsafe { NumberSource::new_unchecked(span!()) })
            }
            _ => {
                while !is_token_start(self.chars.peek()) {
                    self.chars.next();
                }

                Token::Invalid(unsafe { span!() })
            }
        })
    }
}
