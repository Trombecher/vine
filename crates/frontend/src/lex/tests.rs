#![cfg(test)]

use crate::lex::{Keyword, Lexer, Symbol, Token};
use alloc::alloc::Global;
use ecow::EcoString;
use errors::Error;
use fallible_iterator::{FallibleIterator, IteratorExt};
use span::Span;

fn assert_iter(
    mut lexer: Lexer<Global>,
    mut expected: impl FallibleIterator<Item = Span<Token<'static>>, Error = Error>,
) {
    loop {
        let a = lexer.next();
        let b = expected.next();

        assert_eq!(a, b);

        match a {
            Ok(None) | Err(_) => break,
            _ => {}
        }
    }
}

#[test]
fn lex() {
    let lexer = Lexer::new(b"pub fn test(name: str) -> str? { \"yo\" }", Global);

    let tokens = [
        Span {
            value: Token::Keyword(Keyword::Pub),
            source: 0..3,
        },
        Span {
            value: Token::Keyword(Keyword::Fn),
            source: 4..6,
        },
        Span {
            value: Token::Identifier("test"),
            source: 7..11,
        },
        Span {
            value: Token::Symbol(Symbol::LeftParenthesis),
            source: 11..12,
        },
        Span {
            value: Token::Identifier("name"),
            source: 12..16,
        },
        Span {
            value: Token::Symbol(Symbol::Colon),
            source: 16..17,
        },
        Span {
            value: Token::Identifier("str"),
            source: 18..21,
        },
        Span {
            value: Token::Symbol(Symbol::RightParenthesis),
            source: 21..22,
        },
        Span {
            value: Token::Symbol(Symbol::MinusRightAngle),
            source: 23..25,
        },
        Span {
            value: Token::Identifier("str"),
            source: 26..29,
        },
        Span {
            value: Token::Symbol(Symbol::QuestionMark),
            source: 29..30,
        },
        Span {
            value: Token::Symbol(Symbol::LeftBrace),
            source: 31..32,
        },
        Span {
            value: Token::String(EcoString::from("yo")),
            source: 33..37,
        },
        Span {
            value: Token::Symbol(Symbol::RightBrace),
            source: 38..39,
        },
    ];

    assert_iter(lexer, tokens.into_iter().map(Ok).transpose_into_fallible());
}
