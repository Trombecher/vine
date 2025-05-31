use bumpalo::Bump;
use byte_reader::get_lines_and_columns;
use errors::Error;
use frontend::parse::parse_module;
use std::mem::forget;
use std::time::Instant;

fn main() {
    let source = include_str!("../program.vn");

    let alloc = Bump::new();

    let now = Instant::now();
    
    let module = parse_module(source.as_bytes(), &alloc);
    
    println!("dt: {:?}", now.elapsed());
    
    match module {
        Ok(module) => {
            println!("{:#?}", module);
        }
        Err((Error { data, message }, index)) => {
            let (lines, cols) = get_lines_and_columns(source, index.start as usize);

            let mut path = data.file_path.split("::");

            println!(
                "Error: {}\n  --> examples/test.vn:{}:{}\n\n\
            This error was thrown here: https://github.com/Trombecher/vine/blob/master/crates/{}/src{}/mod.rs#L{}-C{}",
                message,
                lines + 1,
                cols + 1,
                path.next().unwrap(),
                path.fold(String::new(), |mut acc, s| {
                    acc.push('/');
                    acc.push_str(s);
                    acc
                }),
                data.source_line,
                data.source_column
            );
        }
    }
}
