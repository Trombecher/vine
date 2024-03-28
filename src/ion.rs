use std::io;
use crate::lexer;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Lexer(lexer::Error)
}