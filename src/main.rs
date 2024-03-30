#![deny(unsafe_code)]

use std::fs::read_to_string;
use std::time::Instant;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;

mod quark;
mod lexer;
mod token;
mod parser;
mod ast;
mod bp;
mod chars;

fn main() -> Result<(), quark::Error> {
    let input = read_to_string("main.qk")
        .map_err(|error| quark::Error::IO(error)).unwrap();
    // lex(input)?;
    parse(input);
    
    Ok(())
}

fn lex(input: String) -> Result<(), quark::Error> {
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
    let now = Instant::now();
    
    let mut parser = Parser::new(Lexer::new(input.chars())).unwrap();
    
    match parser.parse_module() {
        Ok(expressions) => {
            println!("delta: {:?}", now.elapsed());
            println!("{:#?}", expressions)
        },
        Err(error) => println!("error: {:?}, last_token: {:?}", error, parser.last_token),
    }
}