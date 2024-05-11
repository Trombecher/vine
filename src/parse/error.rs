#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedToken(UnexpectedTokenError),
    TagNamesDoNotMatch,
    InvalidAssignmentTarget,
}

#[derive(Debug, PartialEq)]
pub enum UnexpectedTokenError {
    ExpectedSemicolonOrRightBraceWhileParsingEndOfBlock,
    ExpectedKeywordFn,
    ExpectedIdentifierOrLeftParenthesis,
    ExpectedLeftParenthesis,
    NamedFunctionBodiesMustBeSurroundedByBraces,
}