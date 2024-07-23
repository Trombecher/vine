use parse_tools::bytes::Cursor;
use lex::Lexer;
use parse::{Buffered, ParseContext};

fn main() {
    let mut context = ParseContext::new(Buffered::new(Lexer::new(Cursor::new("\"Hello, World!\"".as_bytes()))));
    println!("{:?}", context.parse_expression(0))
}