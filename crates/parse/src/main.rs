use std::fs;
use std::time::Instant;
use error::Error;
use lex::Lexer;
use parse::{Buffered, ParseContext};

fn main() -> Result<(), Error> {
    let file_content = fs::read_to_string("test.vn").unwrap();
    
    let now = Instant::now();
    let mut context = ParseContext::new(Buffered::new(Lexer::new(file_content.as_bytes()))?);
    let expr = context.parse_module_content()?;
    println!("time: {:?}\n{:#?}", now.elapsed(), expr);
    
    Ok(())
}