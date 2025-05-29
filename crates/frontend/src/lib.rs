#![no_std]
#![feature(allocator_api)]
#![feature(let_chains)]

extern crate alloc;

mod warnings;

pub mod lex;
pub mod parse;

#[allow(unused_imports)]
pub use warnings::*;
