use bytes::get_lines_and_columns;
use frontend::bumpalo::Bump;
use frontend::parse::parse_module;

fn main() {
    let source = include_str!("../../../examples/test.vn");
    
    let lex_alloc = Bump::new();
    let parse_alloc = Bump::new();
    
    let (module, _warnings) = parse_module(source.as_bytes(), &lex_alloc, &parse_alloc);

    match module {
        Ok(module) => {
            println!("{:#?}", module);
        }
        Err((err, range)) => {
            let (lines, cols) = get_lines_and_columns(source, range.start as usize);
            println!("Error {:?}\n  --> examples/test.vn:{}:{}", err, lines + 1, cols + 1);
        }
    }
}