#![no_std]

extern crate alloc;

mod warnings;

pub mod lex;
pub mod parse;

#[allow(unused_imports)]
pub use warnings::*;