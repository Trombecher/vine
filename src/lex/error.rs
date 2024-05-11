#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedCharacter(UnexpectedCharacterError),
    UnexpectedEndOfInput(UnexpectedEndOfInputError),
    CannotUseKeywordAsTagName,
}

#[derive(Debug, PartialEq)]
pub enum UnexpectedCharacterError {
    Number,
    NumberTail,
    EndTag,
    SelfClosingStartTag,
    StartTagKeys,
    MarkupEquals,
    MarkupValue,
    StringEscape,
    Token,
    CharLiteralEscape,
    CharLiteralQuote,
    InvalidStringEscape,
}

#[derive(Debug, PartialEq)]
pub enum UnexpectedEndOfInputError {
    EndTag,
    SelfClosingStartTag,
    StartTagKeys,
    MarkupEquals,
    MarkupValue,
    NestedMarkupStart,
    String,
    CharLiteralContent,
    CharLiteralQuote,
}