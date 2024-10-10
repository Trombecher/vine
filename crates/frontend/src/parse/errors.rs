use crate::lex;

#[derive(Debug)]
pub enum Error {
    Lexer(lex::Error),
    UnimplementedError,
    Unimplemented,
    ExpectedIdentifierOfModule,
    UnboundAnnotations,
    UnboundDocComment,
    DocCommentOnUse,
    ExpectedTypeParameter,
    ExpectedTypeParameterDelimiter,
    TrailingClosingBraceInTopLevelModule,
    InvalidStartOfExpression,
    InvalidContinuationOfExpression,
    ExpectedDelimiterInBlock,
    
    /// Expected a line break, ',' or '>'.
    ExpectedDelimiterAfterItemInTPUsage,

    InvalidTypeStart,
    ExpectedField,
    ExpectedDelimiterInObject,
    InvalidStartOfUseChild,
    InvalidPositioningOfTypeParametersInFunction,
    ExpectedDelimiterAfterThisParameter,
    ExpectedThisOrIdInFunctionParameters,
    ExpectedDelimiterAfterParameter
}

impl From<lex::Error> for Error {
    fn from(value: lex::Error) -> Self {
        Self::Lexer(value)
    }
}