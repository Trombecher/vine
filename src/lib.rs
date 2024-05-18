#![feature(maybe_uninit_uninit_array)]
#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![feature(layout_for_ptr)]
#![feature(const_alloc_layout)]
#![allow(warnings)] // TODO: disallow in the future

use std::fmt::Debug;

pub mod chars;
pub mod lex;
pub mod parse;
pub mod resolve;
pub mod vm;
pub mod transpile;

#[derive(Debug, PartialEq)]
pub enum Error {
    Chars(chars::Error),
    Lexer(lex::error::Error),
    Parser(parse::error::Error),
    Resolver(resolve::error::Error),
}

#[derive(Debug, PartialEq)]
pub struct Span<T> where T: Debug {
    pub value: T,
    pub start: u64,
    pub end: u64,
}

impl<T: Debug> Span<T> {
    #[inline]
    pub fn map<U, F: Fn(T) -> U>(self, mapper: F) -> Span<U> where U: Debug {
        Span {
            value: mapper(self.value),
            start: self.start,
            end: self.end,
        }
    }
}

const X: u8 = 2
fn x() {}