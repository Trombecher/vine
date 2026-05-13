#![no_std]
#![feature(str_from_raw_parts)]

pub mod filter;
mod lexer;
pub mod tokens;

pub use lexer::*;

use crate::filter::TokenFilter;

pub fn lex<'source>(input: &'source str) -> TokenFilter<'source, Lexer<'source>> {
    TokenFilter::new(Lexer::new(input))
}
