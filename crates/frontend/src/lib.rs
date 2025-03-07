#![no_std]
#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(allocator_api)]

extern crate alloc;

mod warnings;

pub mod lex;
pub mod buffered;
pub mod parse;
// pub mod resolve;

pub use bumpalo;
#[allow(unused_imports)]
pub use warnings::*;

// Enforce specification of an allocator.
// pub(crate) type Box<'alloc, T> = boxed::Box<T, &'alloc bumpalo::Bump>;
// pub(crate) type Vec<'alloc, T> = vec::Vec<T, &'alloc bumpalo::Bump>;