use std::fs;
use error::get_lines_and_columns;
use lex::{Lexer, Span};
use lex::token::{Token, TokenIterator};

fn main() {
    let source_file = fs::read_to_string("../../docs/tour.vn").unwrap();
    let mut lexer = Lexer::new(source_file.as_bytes());

    loop {
        match lexer.next_token() {
            Ok(Span { value: Token::EndOfInput, .. }) => break,
            Ok(Span { value, source }) => {
                println!(
                    "{:?} at {:?} ({})",
                    value,
                    source,
                    &source_file[source.start as usize..source.end as usize]
                );
            }
            Err(error) => {
                let (line, column) = get_lines_and_columns(&source_file, lexer.index() as usize);
                eprintln!("Error {:?}: {}\n --> crates\\lex\\test.vn:{}:{}", error, error.as_str(), line + 1, column + 1);
                break
            }
        }
    }
}