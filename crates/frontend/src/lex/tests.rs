#![cfg(test)]

use crate::lex::{Keyword, Lexer, Symbol, Token};
use ecow::EcoString;
use fallible_iterator::{FallibleIterator, IteratorExt};
use span::Span;

#[test]
fn lex_keywords() {
    let input = b"as break continue else enum extern fn for if in is let mod mut match package pub return struct this This trait type while _ use";

    let lexer = Lexer::new(input);
    assert_eq!(
        [
            Keyword::As,
            Keyword::Break,
            Keyword::Continue,
            Keyword::Else,
            Keyword::Enum,
            Keyword::Extern,
            Keyword::Fn,
            Keyword::For,
            Keyword::If,
            Keyword::In,
            Keyword::Is,
            Keyword::Let,
            Keyword::Mod,
            Keyword::Mut,
            Keyword::Match,
            Keyword::Package,
            Keyword::Pub,
            Keyword::Return,
            Keyword::Struct,
            Keyword::This,
            Keyword::CapitalThis,
            Keyword::Trait,
            Keyword::Type,
            Keyword::While,
            Keyword::Underscore,
            Keyword::Use,
        ]
        .iter()
        .into_fallible()
        .partial_cmp(lexer),
        Ok(None)
    );
}

#[test]
fn lex() {
    let lexer = Lexer::new(b"pub fn test(name: str) -> str? { \"yo\" }");
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

    assert_eq!(tokens.iter().into_fallible().partial_cmp(lexer), Ok(None))
}   