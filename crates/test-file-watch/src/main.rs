use std::fs;
use std::io::stdout;
use std::mem::transmute;
use std::ops::Range;
use std::path::Path;
use std::time::Instant;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use notify::{recommended_watcher, Error, RecursiveMode, Watcher};
use bytes::{get_lines_and_columns, Index};
use frontend::bumpalo::Bump;
use frontend::lex::Lexer;
use frontend::parse::{Buffered, ParseContext};
use frontend::parse::ast::ModuleContent;

fn x() {
    // let x = 2 < 2 / < 4;
}

fn main() -> Result<(), Error> {
    let mut watcher = recommended_watcher(move |res| {
        match res {
            Ok(_) => {
                let content = fs::read("test.vn").unwrap();
                let arena = Bump::new();
                
                // Clear screen
                execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge), MoveTo(0, 0)).unwrap();
                
                let now = Instant::now();
                let module_content = parse_that(&content, &arena);
                let elapsed = now.elapsed();
                
                match module_content {
                    Ok(o) => {
                        println!("{:?}\n{:#?}", elapsed, o)
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
            Err(e) => println!("watch errors: {:?}", e),
        }
    })?;

    watcher.watch(Path::new("test.vn"), RecursiveMode::NonRecursive)?;

    loop {}
}

fn parse_that<'sf: 'arena, 'arena>(content: &'sf [u8], parse_arena: &'arena Bump) -> Result<ModuleContent<'sf, 'arena>, (frontend::parse::Error, Range<Index>)> {
    let lexer = Lexer::new(&content);
    let buffered_iter = Buffered::new(lexer).map_err(|e| (frontend::parse::Error::from(e), 0..0))?;
    let mut parser = ParseContext::new(buffered_iter, parse_arena);
    
    parser.parse_module_content().map_err(move |e| (e, parser.iter.peek().source.clone()))
}