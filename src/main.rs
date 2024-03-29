#![deny(unsafe_code)]

use std::fs::read_to_string;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;

mod ion;
mod lexer;
mod token;
mod parser;
mod ast;
mod bp;
mod chars;

fn main() -> Result<(), ion::Error> {
    let input = read_to_string("main.ion")
        .map_err(|error| ion::Error::IO(error)).unwrap();
    // lex(input)?;
    parse(input);
    
    Ok(())
}

fn lex(input: String) -> Result<(), ion::Error> {
    let mut lexer = Lexer::new(input.chars());

    loop {
        let next = lexer.next()?;
        if let Token::EndOfInput = next.value {
            break
        }
        
        println!("{:?}", next);
    }
    
    Ok(())
}

fn parse(input: String) {
    let mut parser = Parser::new(Lexer::new(input.chars())).unwrap();
    
    match parser.parse_block() {
        Ok(expressions) => println!("{:?}", expressions),
        Err(error) => println!("error: {:?}, last_token: {:?}", error, parser.last_token),
    }
}