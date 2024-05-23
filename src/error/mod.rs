//! This file was automatically generated.

use std::fmt::{Display, Formatter};

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Error {
    /// **unexpected byte**
    /// 
    /// Illegal first byte.
    E0000,

    /// **unexpected end of input**
    /// 
    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a two byte UTF-8 code point encoding.
    E0001,

    /// **unexpected byte**
    /// 
    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a two byte UTF-8 code point encoding.
    E0002,

    /// **unexpected end of input**
    /// 
    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0003,

    /// **unexpected byte**
    /// 
    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0004,

    /// **unexpected end of input**
    /// 
    /// Expected a third byte because the first byte implied it.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0005,

    /// **unexpected byte**
    /// 
    /// The third byte is not a continuation byte.
    ///
    /// Context: The error occurred in a three byte UTF-8 code point encoding.
    E0006,

    /// **unexpected end of input**
    /// 
    /// Expected a second byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0007,

    /// **unexpected byte**
    /// 
    /// The second byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0008,

    /// **unexpected end of input**
    /// 
    /// Expected a third byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0009,

    /// **unexpected byte**
    /// 
    /// The third byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0010,

    /// **unexpected end of input**
    /// 
    /// Expected a fourth byte because the first byte implied it.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0011,

    /// The fourth byte is not a continuation byte.
    ///
    /// Context: The error occurred in a four byte UTF-8 code point encoding.
    E0012,

    /// **unexpected end of input**
    /// 
    /// Expected escape character.
    E0013,

    /// **unexpected character**
    /// 
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

    /// **unexpected character**
    /// 
    /// Invalid (alphabetic) digit for decimal number literal. In decimal, you can only use numbers from zero to nine.
    ///
    /// Context: The error occurred while lexing a decimal number literal.
    E0017,

    /// **unexpected end of input**
    /// 
    /// Expected a string escape character.
    ///
    /// Context: The error occurred in a string literal.
    E0018,

    /// **unexpected end of input**
    /// 
    /// Expected something.
    ///
    /// Context: The error occurred in a string literal.
    E0019,

    /// **unexpected character**
    /// 
    /// Non-ASCII characters in identifiers are currently not supported.
    ///
    /// Context: The error occurred while lexing an identifier.
    E0020,

    /// **unexpected end of input**
    /// 
    /// Expected first hex digit.
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0021,

    /// **unexpected character**
    /// 
    /// Invalid hex digit (0-9A-F).
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0022,

    /// **unexpected character**
    /// 
    /// Valid hex digit, but out of range for an ascii character (`0..=0x7F`).
    ///
    /// Context: The error occurred in the first digit of a string literal hex escape code.
    E0023,

    /// **unexpected end of input**
    /// 
    /// Expected second hex digit.
    ///
    /// Context: The error occurred in the second digit of a string literal hex escape code.
    E0024,

    /// **unexpected character**
    /// 
    /// Invalid hex digit (0-9A-F).
    ///
    /// Context: The error occurred in the second digit of a string literal hex escape code.
    E0025,

    /// **unexpected character**
    /// 
    /// Expected '>'.
    ///
    /// Context: The error occurred in a markup end tag.
    E0026,

    /// Vine does not have the Rust and C++ like :: path separator. Just use a dot.
    E0027,

    /// **unexpected character**
    /// 
    /// .
    ///
    /// Context: The error occurred while trying to match a new token.
    E0028,

    /// **unexpected character**
    /// 
    /// Expected a number, found an alphabetic character.
    ///
    /// Context: The error occurred while lexing the tail of a floating point decimal number.
    E0029,

    /// **unexpected end of input**
    /// 
    /// Expected a character.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0030,

    /// **unexpected end of input**
    /// 
    /// Expected a single quote to close the character literal.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0031,

    /// **unexpected character**
    /// 
    /// Expected a single quote to close the character literal.
    ///
    /// Context: The error occurred while lexing a character literal.
    E0032,

    /// Expected '>'.
    ///
    /// Context: The error occurred in a self-closing markup tag.
    E0033,

    /// Expected '>', '/' or an identifier.
    ///
    /// Context: The error occurred while collecting attributes in a markup start tag.
    E0034,

    /// Expected '='.
    ///
    /// Context: The error occurred while lexing an attribute in a markup start tag.
    E0035,

    /// Expected '"' or '{'.
    ///
    /// Context: The error occurred while lexing a value in a markup start tag.
    E0036,

    /// **unexpected end of input**
    /// 
    /// Cannot terminate without closing the markup element.
    ///
    /// Context: The error occurred in a markup element (while lexing children).
    E0037,

    /// **unexpected end of input**
    /// 
    /// Expected '/' or a start tag identifier.
    ///
    /// Context: The error occurred in a markup element (while lexing children).
    E0038,
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Source {
    UTF8,
    Lexer,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::UTF8 => "utf-8",
            Source::Lexer => "lexer",
        }
    }
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
            Hint::UnexpectedEndOfInput => "unexpected end of input. ",
            Hint::UnexpectedByte => "unexpected byte. ",
            Hint::UnexpectedCharacter => "unexpected character. ",
        }
    }
}

impl Display for Error {
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
            Error::E0021 => f.write_str("Expected first hex digit."),
            Error::E0022 => f.write_str("Invalid hex digit (0-9A-F)."),
            Error::E0023 => f.write_str("Valid hex digit, but out of range for an ascii character (`0..=0x7F`)."),
            Error::E0024 => f.write_str("Expected second hex digit."),
            Error::E0025 => f.write_str("Invalid hex digit (0-9A-F)."),
            Error::E0026 => f.write_str("Expected '>'."),
            Error::E0027 => f.write_str("Vine does not have the Rust and C++ like :: path separator. Just use a dot."),
            Error::E0028 => f.write_str("."),
            Error::E0029 => f.write_str("Expected a number, found an alphabetic character."),
            Error::E0030 => f.write_str("Expected a character."),
            Error::E0031 => f.write_str("Expected a single quote to close the character literal."),
            Error::E0032 => f.write_str("Expected a single quote to close the character literal."),
            Error::E0033 => f.write_str("Expected '>'."),
            Error::E0034 => f.write_str("Expected '>', '/' or an identifier."),
            Error::E0035 => f.write_str("Expected '='."),
            Error::E0036 => f.write_str("Expected '\"' or '{'."),
            Error::E0037 => f.write_str("Cannot terminate without closing the markup element."),
            Error::E0038 => f.write_str("Expected '/' or a start tag identifier."),
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
            Error::E0021 => Source::Lexer,
            Error::E0022 => Source::Lexer,
            Error::E0023 => Source::Lexer,
            Error::E0024 => Source::Lexer,
            Error::E0025 => Source::Lexer,
            Error::E0026 => Source::Lexer,
            Error::E0027 => Source::Lexer,
            Error::E0028 => Source::Lexer,
            Error::E0029 => Source::Lexer,
            Error::E0030 => Source::Lexer,
            Error::E0031 => Source::Lexer,
            Error::E0032 => Source::Lexer,
            Error::E0033 => Source::Lexer,
            Error::E0034 => Source::Lexer,
            Error::E0035 => Source::Lexer,
            Error::E0036 => Source::Lexer,
            Error::E0037 => Source::Lexer,
            Error::E0038 => Source::Lexer,
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
            Error::E0021 => Some(Hint::UnexpectedEndOfInput),
            Error::E0022 => Some(Hint::UnexpectedCharacter),
            Error::E0023 => Some(Hint::UnexpectedCharacter),
            Error::E0024 => Some(Hint::UnexpectedEndOfInput),
            Error::E0025 => Some(Hint::UnexpectedCharacter),
            Error::E0026 => Some(Hint::UnexpectedCharacter),
            Error::E0027 => None,
            Error::E0028 => Some(Hint::UnexpectedCharacter),
            Error::E0029 => Some(Hint::UnexpectedCharacter),
            Error::E0030 => Some(Hint::UnexpectedEndOfInput),
            Error::E0031 => Some(Hint::UnexpectedEndOfInput),
            Error::E0032 => Some(Hint::UnexpectedCharacter),
            Error::E0033 => None,
            Error::E0034 => None,
            Error::E0035 => None,
            Error::E0036 => None,
            Error::E0037 => Some(Hint::UnexpectedEndOfInput),
            Error::E0038 => Some(Hint::UnexpectedEndOfInput),
        }
    }
}