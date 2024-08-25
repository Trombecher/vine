#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(allocator_api)]

mod errors;
mod warnings;

pub mod parse;
pub mod lex;
pub mod resolve;
pub mod buffered;

pub use bumpalo;

pub use errors::*;
pub use warnings::*;