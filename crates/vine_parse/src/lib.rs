pub mod ast;
mod parser;
mod peekable;

pub use parser::*;
use parser_tools::Span;
use vine_lex::lex;

use crate::ast::Expression;

pub fn parse_expression<'source>(
    input: &'source str,
) -> Result<Span<Expression<'source>>, Error<'source>> {
    Parser::new(lex(input)).parse_root_expression()
}
