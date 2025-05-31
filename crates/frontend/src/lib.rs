#![no_std]
#![feature(allocator_api)]
#![feature(let_chains)]
#![feature(if_let_guard)]
extern crate alloc;

mod warnings;

pub mod lex;
pub mod parse;

#[allow(unused_imports)]
pub use warnings::*;
