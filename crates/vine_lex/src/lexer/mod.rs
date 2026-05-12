#[cfg(test)]
mod tests;

use core::str;

use parser_tools::PeekableChars;

use crate::{NumberSource, Token, WhitespaceSource};

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
            '+'
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
            '+' => Token::Plus,
            '-' => Token::Hypen,
            '*' => Token::Star,
            '/' => Token::Slash,
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
