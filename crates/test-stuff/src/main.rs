use bytes::get_lines_and_columns;
use frontend::bumpalo::Bump;
use frontend::parse::parse_module;
use errors::Error;

fn main() {
    let source = include_str!("../../../examples/test.vn");
    
    let lex_alloc = Bump::new();
    let parse_alloc = Bump::new();
    
    let (module, _warnings) = parse_module(source.as_bytes(), &lex_alloc, &parse_alloc);

    match module {
        Ok(module) => {
            println!("{:#?}", module);
        }
        Err((Error(err), range)) => {
            let (lines, cols) = get_lines_and_columns(source, range.start as usize);

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