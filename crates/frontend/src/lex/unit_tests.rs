#![cfg(test)]

use alloc::alloc::Global;
use crate::lex::{Lexer, Token, UnprocessedString};
use bytes::Span;

#[test]
fn parse_number_dec() {
    assert_eq!(
        Lexer::new(b"000__0123456789", Global)
            .parse_number_dec(0.),
        Ok(Token::Number(123456789.))
    );
}

#[test]
fn parse_string() {
    assert_eq!(
        Lexer::new(br#"abcdefg01239(=)($%\\\1""#, Global)
            .parse_string(),
        Ok(unsafe {
            UnprocessedString::from_raw(r#"abcdefg01239(=)($%\\\1"#)
        })
    )
}

#[test]
fn parse_id() {
    assert_eq!(
        Lexer::new(b"abc_2340598+", Global).parse_id(),
        Ok("abc_2340598")
    );

    assert!(Lexer::new("a😃".as_bytes(), Global).parse_id().is_err());
}

#[test]
fn parse_start_tag() {
    assert_eq!(
        Ok(Span {
            value: Token::MarkupStartTag("tag_name"),
            source: 0..8,
        }),
        Lexer::new(b"tag_name ", Global).parse_start_tag(),
    );
}

#[test]
fn parse_start_tag_e0015() {
    assert!(Lexer::new(b"fn ", Global).parse_start_tag().is_err());
}