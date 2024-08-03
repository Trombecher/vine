#![feature(ptr_sub_ptr)]

use std::fs;
use error::get_lines_and_columns;
use lex::{Lexer, Span};
use lex::token::{Token, TokenIterator};

fn main() {
    let source = fs::read_to_string("../../examples/app.vn").unwrap();
    let mut lexer = Lexer::new(source.as_bytes());

    loop {
        match lexer.next_token() {
            Ok(Span { value: Token::EndOfInput, .. }) => break,
            Ok(token) => {
                let offset = unsafe { token.source.as_ptr().sub_ptr(source.as_ptr()) };
                println!(
                    "{:?} at range {}..{} ({:?})",
                    token.value,
                    offset,
                    offset + token.source.len(),
                    token.source
                );
            }
            Err(error) => {
                let (line, column) = get_lines_and_columns(&source, lexer.cursor_offset());
                eprintln!("Error {:?}: {}\n --> crates\\lex\\test.vn:{}:{}", error, error.as_str(), line + 1, column + 1);
                break
            }
        }
    }
}