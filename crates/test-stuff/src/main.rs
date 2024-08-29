use frontend::bumpalo::Bump;
use frontend::parse::parse_module;

fn main() {
    let lex_alloc = Bump::new();
    let parse_alloc = Bump::new();
    println!("{:#?}", parse_module(b"fn test() {}", &lex_alloc, &parse_alloc));
}