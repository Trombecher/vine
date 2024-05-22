#![cfg(test)]

use crate::chars::Cursor;
use crate::lex::Lexer;
use crate::lex::token::{Keyword, Symbol, Token, TokenIterator};
use crate::Span;

#[test]
fn lex() {
    let mut lexer = Lexer::new(Cursor::new("x += 200".as_bytes()));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Identifier("x"),
        start: 0,
        end: 1,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Symbol(Symbol::PlusEquals),
        start: 2,
        end: 4,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Number(200.),
        start: 5,
        end: 8,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::EndOfInput,
        start: 8,
        end: 8,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::EndOfInput,
        start: 8,
        end: 8,
    }));
}

#[test]
fn lex_markup() {
    let mut lexer = Lexer::new(Cursor::new("let x = <div>{10}</div>".as_bytes()));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Keyword(Keyword::Let),
        start: 0,
        end: 3,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Identifier("x"),
        start: 4,
        end: 5,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Symbol(Symbol::Equals),
        start: 6,
        end: 7,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::MarkupStartTag("div"),
        start: 8,
        end: 12,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::MarkupStartTagEnd,
        start: 12,
        end: 13,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Symbol(Symbol::LeftBrace),
        start: 13,
        end: 14,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Number(10.),
        start: 14,
        end: 16,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::Symbol(Symbol::RightBrace),
        start: 16,
        end: 17,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::MarkupEndTag("div"),
        start: 19,
        end: 23,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::EndOfInput,
        start: 23,
        end: 23,
    }));
    assert_eq!(lexer.next_token(), Ok(Span {
        value: Token::EndOfInput,
        start: 23,
        end: 23,
    }));
}