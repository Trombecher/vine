#![feature(ptr_sub_ptr)]

use std::fs;
use std::mem::transmute;
use error::{Error, get_lines_and_columns};
use lex::{Lexer, Span};
use parse::{Buffered, ParseContext};

fn main() -> Result<(), Error> {
    let file_content = fs::read("test.vn").unwrap();

    let mut context = ParseContext::new(Buffered::new(Lexer::new(&file_content))?);
    context.iter.skip_lb()?; // Skip initial line break
    let module_content = context.parse_module_content()?;
    
    println!("Warnings:");
    
    for Span { source, value } in context.iter.warnings() {
        let (line, col) = unsafe {
            get_lines_and_columns(
                transmute::<&[u8], &str>(&file_content),
                source.as_ptr().sub_ptr(file_content.as_ptr())
            )
        };
        println!("warning: {:?}\n    --> crates/parse/test.vn:{}:{}", value, line + 1, col + 1);
    }
    
    println!("{:#?}", module_content);
    
    Ok(())
}