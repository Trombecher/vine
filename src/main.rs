use std::fs::read;
use std::time::Instant;
use colored::Colorize;
use vine::chars::Cursor;
use vine::Error;
use vine::lex::Lexer;
use vine::parse::ast::Module;
use vine::parse::Parser;

fn print_error(error: Error) {
    println!(
        "{}: {}",
        format!("E{:04} ({} error)", error.code(), error.source().as_str()).bright_red().bold(),
        format!("{}{}", error.hint().map_or("", |x| x.as_str()), error).bright_white(),
    );
}

fn do_something<'s>(source_file: &'s Vec<u8>, module_id: &'static str) -> Result<Module<'s>, Error> {
    let mut parser = Parser::new(Lexer::new(Cursor::new(source_file.as_slice())))?;
    parser.parse_module(module_id)
}

fn main() {
    let source_file = read("./libs/example/src/index.vn").expect("file should exist");
    let now = Instant::now();
    
    match do_something(&source_file, "index") {
        Ok(module) => {
            println!("{:?}\n{:#?}", now.elapsed(), module);
        }
        Err(error) => {
            print_error(error);
        }
    }
}