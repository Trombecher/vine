#![cfg(test)]

use crate::lex::{Keyword, Lexer, Symbol, Token};
use alloc::alloc::Global;
use ecow::EcoString;
use errors::Error;
use fallible_iterator::{FallibleIterator, IteratorExt};
use span::Span;

fn assert_iter(
    mut lexer: Lexer<Global>,
    mut expected: impl FallibleIterator<Item=Span<Token<'static>>, Error=Error>,
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
fn lex_keywords() {
    let input = b"as break continue else enum extern fn for if in is let mod mut match package pub return struct this This trait type while _ use";

    assert_iter(
        Lexer::new(input, Global),
        [
            Span {
                value: Keyword::As,
                source: 0..2,
            },
            Span {
                value: Keyword::Break,
                source: 3..8,
            },
            Span {
                value: Keyword::Continue,
                source: 9..17,
            },
            Span {
                value: Keyword::Else,
                source: 18..22,
            },
            Span {
                value: Keyword::Enum,
                source: 23..27,
            },
            Span {
                value: Keyword::Extern,
                source: 28..34,
            },
            Span {
                value: Keyword::Fn,
                source: 35..37,
            },
            Span {
                value: Keyword::For,
                source: 38..41,
            },
            Span {
                value: Keyword::If,
                source: 42..44,
            },
            Span {
                value: Keyword::In,
                source: 45..47,
            },
            Span {
                value: Keyword::Is,
                source: 48..50,
            },
            Span {
                value: Keyword::Let,
                source: 51..54,
            },
            Span {
                value: Keyword::Mod,
                source: 55..58,
            },
            Span {
                value: Keyword::Mut,
                source: 59..62,
            },
            Span {
                value: Keyword::Match,
                source: 63..68,
            },
            Span {
                value: Keyword::Package,
                source: 69..76,
            },
            Span {
                value: Keyword::Pub,
                source: 77..80,
            },
            Span {
                value: Keyword::Return,
                source: 81..87,
            },
            Span {
                value: Keyword::Struct,
                source: 88..94,
            },
            Span {
                value: Keyword::This,
                source: 95..99,
            },
            Span {
                value: Keyword::CapitalThis,
                source: 100..104,
            },
            Span {
                value: Keyword::Trait,
                source: 105..110,
            },
            Span {
                value: Keyword::Type,
                source: 111..115,
            },
            Span {
                value: Keyword::While,
                source: 116..121,
            },
            Span {
                value: Keyword::Underscore,
                source: 122..123,
            },
            Span {
                value: Keyword::Use,
                source: 124..127,
            },
        ]
            .into_iter()
            .map(|x| Ok(x.map(|kw| Token::Keyword(kw))))
            .transpose_into_fallible(),
    );
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
