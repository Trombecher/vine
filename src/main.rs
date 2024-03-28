use std::fs::read_to_string;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;

mod ion;
mod lexer;
mod token;
mod markup;
mod parser;
mod ast;
mod bp;

fn main() -> Result<(), ion::Error> {
    let input = read_to_string("main.ion")
        .map_err(|error| ion::Error::IO(error))?;
    
    let mut parser = Parser::new(Lexer::new(input.chars()))?;
    let expressions = parser.parse_block()?;
    println!("{:?}", expressions);
    
    Ok(())
}