use std::fs;
use std::io::stdout;
use std::mem::transmute;
use std::ops::Range;
use std::path::Path;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use notify::{recommended_watcher, Error, RecursiveMode, Watcher};
use bytes::{get_lines_and_columns, Index};
use frontend::lex::Lexer;
use frontend::parse::{Buffered, ParseContext};
use frontend::parse::ast::ModuleContent;

fn main() -> Result<(), Error> {
    let mut watcher = recommended_watcher(move |res| {
        match res {
            Ok(_) => {
                let content = fs::read("test.vn").unwrap();
                execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge), MoveTo(0, 0)).unwrap();
                match parse_that(&content) {
                    Ok(o) => {
                        println!("{:#?}", o)
                    }
                    Err((err, range)) => unsafe {
                        let (line, col) = get_lines_and_columns(
                            transmute(content.as_slice()),
                            range.start as usize
                        );
                        
                        println!("{:?}\n  --> crates/test-file-watch/test.vn:{}:{}", err, line + 1, col + 1)
                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    watcher.watch(Path::new("test.vn"), RecursiveMode::NonRecursive)?;

    loop {}
}

fn parse_that(content: &[u8]) -> Result<ModuleContent, (frontend::parse::Error, Range<Index>)> {
    let lexer = Lexer::new(&content);
    
    let mut parser = ParseContext::new(Buffered::new(lexer).map_err(|e| (frontend::parse::Error::from(e), 0..0))?);
    parser.parse_module_content().map_err(|e| (e, parser.iter.peek().source.clone()))
}