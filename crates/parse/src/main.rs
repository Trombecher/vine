use std::time::Instant;
use error::Error;
use lex::Lexer;
use parse::{Buffered, ParseContext};

fn main() -> Result<(), Error> {
    let now = Instant::now();
    let mut context = ParseContext::new(Buffered::new(Lexer::new("fn two(a: num, b: num) -> num { a \n a + b }\n fn main() {}".as_bytes()))?);
    let expr = context.parse_module_content()?;
    println!("time: {:?}\n{:#?}", now.elapsed(), expr);
    Ok(())
}