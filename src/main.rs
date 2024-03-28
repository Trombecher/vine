use std::fs::read_to_string;
use crate::lexer::Lexer;
use crate::token::TokenKind;

mod ion;
mod lexer;
mod token;
mod markup;

fn main() -> Result<(), ion::Error> {
    let input = read_to_string("main.ion")
        .map_err(|error| ion::Error::IO(error))?;
    
    let mut lexer = Lexer::new(input.chars());
    loop {
        let token = lexer.next()?;
        if let TokenKind::EndOfInput = token.kind {
            break;
        }
        
        println!("{:?}", token);
    }
    
    Ok(())
}