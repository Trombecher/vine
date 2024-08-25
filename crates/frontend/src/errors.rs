use crate::{lex, parse};

pub enum Error {
    Lexer(lex::Error),
    Parser(parse::Error),
}