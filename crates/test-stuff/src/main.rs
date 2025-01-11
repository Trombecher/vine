use std::mem::forget;
use bytes::get_lines_and_columns;
use errors::Error;
use frontend::bumpalo::Bump;
use frontend::parse::parse_module;

fn main() {
    let source = include_str!("../../../examples/test.vn");

    let lex_alloc = Bump::new();
    let parse_alloc = Bump::new();
    
    {
        match parse_module(source.as_bytes(), &lex_alloc, &parse_alloc) {
            Ok(module) => {
                println!("{:#?}", module);
            }
            Err((Error(err), index)) => {
                let (lines, cols) = get_lines_and_columns(source, index as usize);

                let mut path = err.file_path.split("::");

                println!(
                    "Error: {}\n  --> examples/test.vn:{}:{}\n\n\
                This error was thrown here: https://github.com/Trombecher/vine/blob/master/crates/{}/src{}/mod.rs#L{}-C{}",
                    err.message,
                    lines + 1,
                    cols + 1,
                    path.next().unwrap(),
                    path.fold(String::new(), |mut acc, s| {
                        acc.push('/');
                        acc.push_str(s);
                        acc
                    }),
                    err.source_line,
                    err.source_column
                );
            }
        }
    }
    
    forget(parse_alloc);
}
