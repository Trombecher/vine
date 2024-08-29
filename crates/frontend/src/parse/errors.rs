use crate::lex;

#[derive(Debug)]
pub enum Error {
    Lexer(lex::Error),
    ExpectedIdentifierOfModule,
    UnboundAnnotations,
    UnboundDocComment,
    DocCommentOnUse,
    ExpectedTypeParameter,
    ExpectedTypeParameterDelimiter,
    UnimplementedError,
    Unimplemented,
    TrailingClosingBraceInTopLevelModule
}

impl From<lex::Error> for Error {
    fn from(value: lex::Error) -> Self {
        Self::Lexer(value)
    }
}