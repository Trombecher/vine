#![no_std]
#![feature(str_from_raw_parts)]

mod filter;
mod filtered_tokens;
mod lexer;
mod tokens;

pub use filter::*;
pub use filtered_tokens::*;
pub use lexer::*;
pub use tokens::*;

pub fn lex<'source>(input: &'source str) -> TokenFilter<'source, Lexer<'source>> {
    TokenFilter::new(Lexer::new(input))
}
