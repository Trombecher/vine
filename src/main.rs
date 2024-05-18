use std::fs::read;
use std::time::Instant;
use vine::chars::Cursor;
use vine::lex::Lexer;
use vine::parse::Parser;

fn main() -> Result<(), vine::Error> {
    let source_file = read("../libs/example/src/list.vn").expect("file should exist");
    
    let mut parser = Parser::new(Lexer::new(Cursor::new(source_file.as_slice())))?;
    let now = Instant::now();
    let module = parser.parse_module("app");
    
    println!("{:?}", now.elapsed());
    
    match module {
        Ok(module) => println!("{:#?}", module),
        Err(error) => {
            println!("{:?}.\nLast token: {:?}", error, parser.last_token());
        }
    }
    
    Ok(())
}