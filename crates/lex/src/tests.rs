#![cfg(test)]

use parse_tools::bytes::Cursor;
use error::Error;
use crate::{Lexer, Span};
use crate::token::{Keyword, Symbol, Token};

macro_rules! match_tokens {
    ($source:literal, $tokens:expr) => {
        use crate::Lexer;
        use parse_tools::bytes::Cursor;
        
        static TEXT: &'static str = $source;
        
        let tokens = $tokens;
        let mut lexer = Lexer::new(Cursor::new(TEXT.as_bytes()));
        
        for correct_token in tokens {
            let lexer_token = lexer.next_token().unwrap();
            
            assert_eq!(
                (
                    correct_token.value,
                    correct_token.source,
                    correct_token.source.as_ptr(),
                    correct_token.source.len()
                ),
                (
                    lexer_token.value,
                    lexer_token.source,
                    lexer_token.source.as_ptr(),
                    lexer_token.source.len()
                )
            );
        }
    
        assert_eq!(lexer.next_token(), Ok(Span {
            value: Token::EndOfInput,
            source: &TEXT[TEXT.len()..TEXT.len()]
        }));
    };
}

#[test]
fn lex() {
    match_tokens!(
        "pub fn test(name: str) -> str? { \"yo\" }",
        [
            Span {
                value: Token::Keyword(Keyword::Pub),
                source: &TEXT[0..3]
            },
            Span {
                value: Token::Keyword(Keyword::Fn),
                source: &TEXT[4..6]
            },
            Span {
                value: Token::Identifier("test"),
                source: &TEXT[7..11]
            },
            Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source: &TEXT[11..12]
            },
            Span {
                value: Token::Identifier("name"),
                source: &TEXT[12..16]
            },
            Span {
                value: Token::Symbol(Symbol::Colon),
                source: &TEXT[16..17]
            },
            Span {
                value: Token::Identifier("str"),
                source: &TEXT[18..21]
            },
            Span {
                value: Token::Symbol(Symbol::RightParenthesis),
                source: &TEXT[21..22]
            },
            Span {
                value: Token::Symbol(Symbol::MinusRightAngle),
                source: &TEXT[23..25]
            },
            Span {
                value: Token::Identifier("str"),
                source: &TEXT[26..29]
            },
            Span {
                value: Token::Symbol(Symbol::QuestionMark),
                source: &TEXT[29..30]
            },
            Span {
                value: Token::Symbol(Symbol::LeftBrace),
                source: &TEXT[31..32]
            },
            Span {
                value: Token::String("yo".to_string()),
                source: &TEXT[33..37]
            },
            Span {
                value: Token::Symbol(Symbol::RightBrace),
                source: &TEXT[38..39]
            },
        ]
    );
}

#[test]
fn parse_start_tag() {
    static TEXT: &'static str = "tag_name ";

    assert_eq!(
        Ok(Span {
            value: Token::MarkupStartTag("tag_name"),
            source: &TEXT[0..8],
        }),
        Lexer::new(Cursor::new(TEXT.as_bytes())).parse_start_tag(),
    );
}

#[test]
fn parse_start_tag_e0015() {
    static TEXT: &'static str = "fn ";

    assert_eq!(
        Err(Error::E0015),
        Lexer::new(Cursor::new(TEXT.as_bytes())).parse_start_tag(),
    );
}

#[test]
fn output() {
    let mut lexer = Lexer::new(Cursor::new(" fn return_any() -> any { 20 }".as_bytes()));
    
    loop {
        match lexer.next_token() {
            Ok(Span { value: Token::EndOfInput, .. }) => break,
            Ok(token) => {
                println!("{:?}", token);
            }
            Err(error) => {
                println!("{:?}", error);
                break
            }
        }
    }
}