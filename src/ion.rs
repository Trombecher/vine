use std::io;
use crate::{lexer, parser};

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Lexer(lexer::Error),
    Parser(parser::Error),
}