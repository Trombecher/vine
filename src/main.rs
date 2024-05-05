use std::fs::read_to_string;
use std::time::Instant;
use vine::lex::Lexer;
use vine::parse::Parser;

fn main() -> Result<(), vine::Error> {
    let source_file = read_to_string("libs/example/src/app.vn")
        .map_err(|err| vine::Error::IO(err))?;
    
    let mut lexer = Parser::new(Lexer::new(source_file.chars()))?;
    let now = Instant::now();
    let module = lexer.parse_module("app");
    
    println!("{:?}", now.elapsed());
    
    match module {
        Ok(module) => println!("{:#?}", module),
        Err(error) => {
            println!("{:#?}; last token: {:#?}", error, lexer.last_token())
        }
    }
    
    Ok(())
}