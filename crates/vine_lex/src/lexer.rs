use crate::{Token, TokenKind, chars::Chars, error::Error};

/*

Keywords:

* fn
* let
* if
* else
* match
* for
* pub
* use
* return

* while
* continue
* break
* loop

*/

pub struct TokenSplitter<'input> {
    chars: Chars<'input>,
}

impl<'input> TokenSplitter<'input> {
    #[inline]
    #[must_use]
    pub fn new(s: &'input str) -> Self {
        Self {
            chars: Chars::new(s),
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        match self.chars.next() {
            None => None,
            Some('=') => Some(Token {
                kind: TokenKind::Equals,
                len: 1,
            }),
            Some('!') => Some(Token {
                kind: TokenKind::ExclamationMark,
                len: 1,
            }),
            Some('+') => Some(Token {
                kind: TokenKind::Plus,
                len: 1,
            }),
            Some('-') => Some(Token {
                kind: TokenKind::Hypen,
                len: 1,
            }),
            Some('*') => Some(Token {
                kind: TokenKind::Star,
                len: 1,
            }),
            Some('/') => Some(Token {
                kind: TokenKind::Slash,
                len: 1,
            }),
            Some('.') => Some(Token {
                kind: TokenKind::Period,
                len: 1,
            }),
            Some(',') => Some(Token {
                kind: TokenKind::Comma,
                len: 1,
            }),
            Some('(') => Some(Token {
                kind: TokenKind::OpenParenthesis,
                len: 1,
            }),
            Some(')') => Some(Token {
                kind: TokenKind::CloseParenthesis,
                len: 1,
            }),
            Some('{') => Some(Token {
                kind: TokenKind::OpenBrace,
                len: 1,
            }),
            Some('}') => Some(Token {
                kind: TokenKind::CloseBrace,
                len: 1,
            }),
            Some('[') => Some(Token {
                kind: TokenKind::CloseBracket,
                len: 1,
            }),
            Some(']') => Some(Token {
                kind: TokenKind::CloseBracket,
                len: 1,
            }),
            Some('<') => Some(Token {
                kind: TokenKind::LessThan,
                len: 1,
            }),
            Some('>') => Some(Token {
                kind: TokenKind::GreaterThan,
                len: 1,
            }),
            Some('|') => Some(Token {
                kind: TokenKind::Bar,
                len: 1,
            }),
            Some('&') => Some(Token {
                kind: TokenKind::Ampersand,
                len: 1,
            }),
            Some('^') => Some(Token {
                kind: TokenKind::Caret,
                len: 1,
            }),
            Some('@') => Some(Token {
                kind: TokenKind::At,
                len: 1,
            }),
            Some(';') => Some(Token {
                kind: TokenKind::Semicolon,
                len: 1,
            }),
            _ => {
                let len = 0;

                Some(Token {
                    len,
                    kind: TokenKind::Invalid,
                })
            }
        }
    }
}
