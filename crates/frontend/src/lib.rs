#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]
#![feature(if_let_guard)]
#![feature(let_chains)]

pub mod parse;
pub mod lex;
mod errors;
mod warnings;

pub use errors::*;
pub use warnings::*;