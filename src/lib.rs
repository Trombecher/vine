#![feature(maybe_uninit_uninit_array)]
#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![allow(warnings)] // TODO: turn back off

pub mod lex;
pub mod parse;
pub mod resolve;
pub mod vm;
pub mod transpile;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Lexer(lex::Error),
    Parser(parse::Error),
}