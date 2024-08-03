use std::time::Instant;
use error::Error;
use lex::Lexer;
use parse::{Buffered, ParseContext};

fn main() -> Result<(), Error> {
    let now = Instant::now();
    let mut context = ParseContext::new(Buffered::new(Lexer::new("1 + 2 * 3 + 4 % 6".as_bytes()))?);
    let expr = context.parse_expression(0);
    println!("time: {:?}\n{:#?}", now.elapsed(), expr);
    Ok(())
}