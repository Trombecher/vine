#![cfg(test)]

use crate::lex::{Error, Lexer, Token, UnprocessedString};
use bytes::Span;

#[test]
fn parse_number_dec() {
    assert_eq!(
        Lexer::new(b"000__0123456789")
            .parse_number_dec(0.),
        Ok(Token::Number(123456789.))
    );
}

#[test]
fn parse_string() {
    assert_eq!(
        Lexer::new(br#"abcdefg01239(=)($%\\\1""#)
            .parse_string(),
        Ok(unsafe {
            UnprocessedString::from_raw(r#"abcdefg01239(=)($%\\\1"#)
        })
    )
}

#[test]
fn parse_id() {
    assert_eq!(
        Lexer::new(b"abc_2340598+").parse_id(),
        Ok("abc_2340598")
    );

    assert_eq!(
        Lexer::new("a😃".as_bytes()).parse_id(),
        Err(Error::InvalidCharacterInIdentifier)
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
        Err(Error::KeywordAsTagName),
        Lexer::new(b"fn ").parse_start_tag(),
    );
}