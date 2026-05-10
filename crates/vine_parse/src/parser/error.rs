use parser_tools::Span;
use vine_lex::FilteredToken;

pub type Error<'source> = Box<ErrorInfo<'source>>;

#[derive(Debug, Clone)]
pub struct ErrorInfo<'source> {
    pub found: Option<Span<FilteredToken<'source>>>,
    pub message: &'static str,
}
