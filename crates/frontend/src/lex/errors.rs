#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Bytes(bytes::Error),
    
    InvalidDigitInDecimalNumber,
    InvalidDigitInDecimalNumberTail,
    UnterminatedString,
    InvalidCharacterInIdentifier,
    ExpectedBinaryDigit,
    InvalidDigitInBinaryNumber,
    EncounteredPathSeparator,
    UnterminatedCharacter,
    IllegalCharacter,
    KeywordAsTagName,
    UnterminatedEndTag,
    UnterminatedSelfClosingTag,
    IllegalCharacterInProps,
    ExpectedEquals,
    ExpectedValue,
    UnterminatedMarkupElement,
    UnterminatedTagStart,
    InvalidEscapeSequence,
    InvalidHexEscapeFirst,
    InvalidHexEscapeSecond,
    UnimplementedError,
}

impl From<bytes::Error> for Error {
    fn from(value: bytes::Error) -> Self {
        Self::Bytes(value)
    }
}