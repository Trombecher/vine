//! This file was automatically generated.

use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Error {
    /// Illegal first byte.
    E0000,

    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a two byte UTF-8 code point encoding.
    E0001,

    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a two byte UTF-8 code point encoding.
    E0002,

    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0003,

    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0004,

    /// Expected a third byte because the first byte implied it.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0005,

    /// The third byte is not a continuation byte.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0006,

    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0007,

    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0008,

    /// Expected a third byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0009,

    /// The third byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0010,

    /// Expected a fourth byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0011,

    /// The fourth byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0012,

    /// Expected escape character.
    E0013,

    /// Invalid string escape.
    ///
    /// Context: The error occurred after '\\' in a string escape.
    E0014,

    /// Cannot use a keyword as a tag name.
    ///
    /// Context: The error occurred in a markup start tag.
    E0015,

    /// Cannot use a keyword as a tag name.
    ///
    /// Context: The error occurred in a markup end tag.
    E0016,

    /// Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine.
    ///
    /// Context: The error occurred while lexing a decimal number literal.
    E0017,

    /// Expected a string escape character.
    ///
    /// Context: The error occurred in a string literal.
    E0018,

    /// Expected something.
    ///
    /// Context: The error occurred in a string literal.
    E0019,

    /// Non-ASCII characters in identifiers are currently not supported.
    ///
    /// Context: The error occurred while lexing an identifier.
    E0020,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Source {
    UTF8,
    Lexer,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Hint {
    UnexpectedEndOfInput,
    UnexpectedByte,
    UnexpectedCharacter,
}

impl Hint {
    pub fn as_str(self) -> &'static str {
        match self {
            Hint::UnexpectedEndOfInput => "unexpected end of input",
            Hint::UnexpectedByte => "unexpected byte",
            Hint::UnexpectedCharacter => "unexpected character",
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::E0000 => f.write_str("Illegal first byte."),
            Error::E0001 => f.write_str("Expected a second byte because the first byte implied it."),
            Error::E0002 => f.write_str("The second byte is not a continuation byte."),
            Error::E0003 => f.write_str("Expected a second byte because the first byte implied it."),
            Error::E0004 => f.write_str("The second byte is not a continuation byte."),
            Error::E0005 => f.write_str("Expected a third byte because the first byte implied it."),
            Error::E0006 => f.write_str("The third byte is not a continuation byte."),
            Error::E0007 => f.write_str("Expected a second byte because the first byte implied it."),
            Error::E0008 => f.write_str("The second byte is not a continuation byte."),
            Error::E0009 => f.write_str("Expected a third byte because the first byte implied it."),
            Error::E0010 => f.write_str("The third byte is not a continuation byte."),
            Error::E0011 => f.write_str("Expected a fourth byte because the first byte implied it."),
            Error::E0012 => f.write_str("The fourth byte is not a continuation byte."),
            Error::E0013 => f.write_str("Expected escape character."),
            Error::E0014 => f.write_str("Invalid string escape."),
            Error::E0015 => f.write_str("Cannot use a keyword as a tag name."),
            Error::E0016 => f.write_str("Cannot use a keyword as a tag name."),
            Error::E0017 => f.write_str("Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine."),
            Error::E0018 => f.write_str("Expected a string escape character."),
            Error::E0019 => f.write_str("Expected something."),
            Error::E0020 => f.write_str("Non-ASCII characters in identifiers are currently not supported."),
        }
    }
}

impl Error {
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn source(self) -> Source {
        match self {
            Error::E0000 => Source::UTF8,
            Error::E0001 => Source::UTF8,
            Error::E0002 => Source::UTF8,
            Error::E0003 => Source::UTF8,
            Error::E0004 => Source::UTF8,
            Error::E0005 => Source::UTF8,
            Error::E0006 => Source::UTF8,
            Error::E0007 => Source::UTF8,
            Error::E0008 => Source::UTF8,
            Error::E0009 => Source::UTF8,
            Error::E0010 => Source::UTF8,
            Error::E0011 => Source::UTF8,
            Error::E0012 => Source::UTF8,
            Error::E0013 => Source::Lexer,
            Error::E0014 => Source::Lexer,
            Error::E0015 => Source::Lexer,
            Error::E0016 => Source::Lexer,
            Error::E0017 => Source::Lexer,
            Error::E0018 => Source::Lexer,
            Error::E0019 => Source::Lexer,
            Error::E0020 => Source::Lexer,
        }
    }

    pub const fn hint(self) -> Option<Hint> {
        match self {
            Error::E0000 => Some(Hint::UnexpectedByte),
            Error::E0001 => Some(Hint::UnexpectedEndOfInput),
            Error::E0002 => Some(Hint::UnexpectedByte),
            Error::E0003 => Some(Hint::UnexpectedEndOfInput),
            Error::E0004 => Some(Hint::UnexpectedByte),
            Error::E0005 => Some(Hint::UnexpectedEndOfInput),
            Error::E0006 => Some(Hint::UnexpectedByte),
            Error::E0007 => Some(Hint::UnexpectedEndOfInput),
            Error::E0008 => Some(Hint::UnexpectedByte),
            Error::E0009 => Some(Hint::UnexpectedEndOfInput),
            Error::E0010 => Some(Hint::UnexpectedByte),
            Error::E0011 => Some(Hint::UnexpectedEndOfInput),
            Error::E0012 => None,
            Error::E0013 => Some(Hint::UnexpectedEndOfInput),
            Error::E0014 => Some(Hint::UnexpectedCharacter),
            Error::E0015 => None,
            Error::E0016 => None,
            Error::E0017 => Some(Hint::UnexpectedCharacter),
            Error::E0018 => Some(Hint::UnexpectedEndOfInput),
            Error::E0019 => Some(Hint::UnexpectedEndOfInput),
            Error::E0020 => Some(Hint::UnexpectedCharacter),
        }
    }
}