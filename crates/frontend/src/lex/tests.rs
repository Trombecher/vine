#![cfg(test)]

use alloc::alloc::Global;
use crate::lex::{Keyword, Lexer, Symbol, Token, TokenIterator, UnprocessedString};
use bytes::{Span, Index};

macro_rules! match_tokens {
    ($source:literal, $tokens:expr) => {
        let mut lexer = Lexer::new($source, Global);

        for correct_token in $tokens {
            let lexer_token = lexer.next_token().unwrap();

            assert_eq!(
                correct_token,
                lexer_token
            );
        }

        assert_eq!(lexer.next_token().unwrap(), Span {
            value: Token::EndOfInput,
            source: $source.len() as Index..$source.len() as Index
        });
    };
}

#[test]
fn lex_keywords() {
    let input = b"as break continue else enum extern false fn for if in let mod mut match pub return struct this trait true type while _ use";

    let mut lexer = Lexer::new(input, Global);
    let keywords = [
        Keyword::As,
        Keyword::Break,
        Keyword::Continue,
        Keyword::Else,
        Keyword::Enum,
        Keyword::Extern,
        Keyword::False,
        Keyword::Fn,
        Keyword::For,
        Keyword::If,
        Keyword::In,
        Keyword::Let,
        Keyword::Mod,
        Keyword::Mut,
        Keyword::Match,
        Keyword::Pub,
        Keyword::Return,
        Keyword::Struct,
        Keyword::This,
        Keyword::Trait,
        Keyword::True,
        Keyword::Type,
        Keyword::While,
        Keyword::Underscore,
        Keyword::Use,
    ];

    for kw in keywords {
        assert_eq!(
            lexer.next_token().map(|token| token.value).unwrap(),
            Token::Keyword(kw)
        )
    }

    assert_eq!(
        lexer.next_token().unwrap(),
        Span {
            value: Token::EndOfInput,
            source: input.len() as Index..input.len() as Index
        }
    )
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