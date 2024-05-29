use std::fs::read;
use std::io::stdout;
use std::time::Instant;
use parse_tools::bytes::Cursor;
use vine::error::format::format;
use vine::lex::Lexer;
use vine::parse::ParseContext;

fn main() {
    let source_file = read("./libs/example/src/index.vn").expect("file should exist");
    let now = Instant::now();

    let mut parser = ParseContext::new(Lexer::new(Cursor::new(source_file.as_slice()))).unwrap();
    
    match parser.parse_module() {
        Ok(module) => {
            println!("{:?}\n{:#?}", now.elapsed(), module);
        }
        Err(error) => {
            format(error, &mut stdout()).unwrap();
        }
    }
}