#![cfg(test)]

use std::time::{Duration, Instant};
use crate::chars::Cursor;
use crate::lex::Lexer;
use crate::lex::token::Token;

#[test]
fn tokens() {
    let mut lexer = Lexer::new(Cursor::new("let x = 10;\nfn test(this) -> This { self.current = \"yo\"; }".as_bytes()));

    let now = Instant::now();
    
    loop {
        if let Token::EndOfInput = lexer.next().unwrap().value {
            break
        }
    }
    
    assert_eq!(now.elapsed(), Duration::default())
}