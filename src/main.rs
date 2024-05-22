use std::fs::read;
use vine::chars::Cursor;
use vine::Error;
use vine::lex::Lexer;
use vine::lex::token::{Token, TokenIterator};

fn print_error(error: Error) {
    println!(
        "\u{1B}[31;1;4mError E{:04} has occurred {:?}\u{1B}[0m:\n{:?}",
        error.code(),
        error
            .hint()
            .map_or("", |x| x.as_str()),
        error
    );
}

fn main() {
    let source_file = read("./libs/example/src/index.vn").expect("file should exist");
    
    let mut lexer = Lexer::new(Cursor::new(source_file.as_slice()));

    let mut result;

    loop {
        result = lexer.next_token();
        
        match result {
            Ok(token) => {
                if let Token::EndOfInput = token.value {
                    break
                }
                
                println!("{:?}", token);
            }
            Err(error) => {
                print_error(error);
                return;
            }
        }
    }
}