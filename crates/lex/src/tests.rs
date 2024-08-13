#![cfg(test)]

use error::Error;
use crate::{Lexer, Span, Index};
use crate::token::{Keyword, Symbol, Token, TokenIterator, UnprocessedString};

macro_rules! match_tokens {
    ($source:literal, $tokens:expr) => {
        let mut lexer = Lexer::new($source);

        for correct_token in $tokens {
            let lexer_token = lexer.next_token().unwrap();

            assert_eq!(
                correct_token,
                lexer_token
            );
        }

        assert_eq!(lexer.next_token(), Ok(Span {
            value: Token::EndOfInput,
            source: $source.len() as Index..$source.len() as Index
        }));
    };
}

#[test]
fn lex() {
    match_tokens!(
        b"pub fn test(name: str) -> str? { \"yo\" }",
        [
            Span {
                value: Token::Keyword(Keyword::Pub),
                source: 0..3,
            },
            Span {
                value: Token::Keyword(Keyword::Fn),
                source: 4..6
            },
            Span {
                value: Token::Identifier("test"),
                source: 7..11
            },
            Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source: 11..12
            },
            Span {
                value: Token::Identifier("name"),
                source: 12..16
            },
            Span {
                value: Token::Symbol(Symbol::Colon),
                source: 16..17
            },
            Span {
                value: Token::Identifier("str"),
                source: 18..21
            },
            Span {
                value: Token::Symbol(Symbol::RightParenthesis),
                source: 21..22
            },
            Span {
                value: Token::Symbol(Symbol::MinusRightAngle),
                source: 23..25
            },
            Span {
                value: Token::Identifier("str"),
                source: 26..29
            },
            Span {
                value: Token::Symbol(Symbol::QuestionMark),
                source: 29..30
            },
            Span {
                value: Token::Symbol(Symbol::LeftBrace),
                source: 31..32
            },
            Span {
                value: Token::String(unsafe { UnprocessedString::from_raw("yo") }),
                source: 33..37
            },
            Span {
                value: Token::Symbol(Symbol::RightBrace),
                source: 38..39
            },
        ]
    );
}

#[test]
fn parse_start_tag() {
    assert_eq!(
        Ok(Span {
            value: Token::MarkupStartTag("tag_name"),
            source: 0..8,
        }),
        Lexer::new(b"tag_name ").parse_start_tag(),
    );
}

#[test]
fn parse_start_tag_e0015() {
    assert_eq!(
        Err(Error::E0015),
        Lexer::new(b"fn ").parse_start_tag(),
    );
}