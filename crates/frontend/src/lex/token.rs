use std::fmt::Debug;

use bytes::{Cursor, Span};
use phf::phf_map;

use crate::lex::{unescape_char, Error};

/// Boxes an `&str` which has some characteristics it must obey.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UnprocessedString<'a>(&'a str);

impl<'a> UnprocessedString<'a> {
    pub const unsafe fn from_raw(slice: &'a str) -> UnprocessedString<'a> {
        UnprocessedString(slice)
    }

    pub fn process(&self) -> Result<String, Error> {
        // By allocating more, no reallocations will take place
        let mut string = String::with_capacity(self.0.len());

        // Get first and past-the-end pointers
        let mut cursor = Cursor::new(self.0.as_bytes());

        loop {
            match cursor.next() {
                None => break,
                Some(b'\\') => string.push(unescape_char(&mut cursor)?),
                // SAFETY: self.0 is a string.
                Some(byte) => unsafe { string.as_mut_vec() }.push(byte)
            }
        }

        Ok(string)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum Token<'a> {
    Char(char),

    /// An identifier token. Guaranteed to match against the regex `([a-zA-Z][a-zA-Z_0-9]*)|([a-zA-Z_][a-zA-Z_0-9]+)`.
    Identifier(&'a str),

    /// A number
    Number(f64),

    /// A documentation comment.
    DocComment(&'a str),

    /// A [Symbol].
    Symbol(Symbol),

    /// A [Keyword].
    Keyword(Keyword),

    /// The contained text needs processing:
    ///
    /// - Validate escape sequences
    /// - Normalize line breaks
    String(UnprocessedString<'a>),

    MarkupStartTag(&'a str),
    MarkupKey(&'a str),
    MarkupStartTagEnd,
    MarkupClose,

    /// The contained text needs processing.
    ///
    /// Although the leading whitespace is removed, the trailing whitespace needs to be removed
    /// and internal whitespace needs to be collapsed.
    MarkupText(&'a str),

    MarkupEndTag(&'a str),
    LineBreak,

    /// This token means that the lexer has finished lexing.
    EndOfInput,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Keyword {
    As,
    Break,
    Continue,
    Else,
    Enum,
    Extern,
    False,
    Fn,
    For,
    If,
    In,
    Let,
    Mod,
    Mut,
    Match,
    Pub,
    Return,
    Struct,
    This,
    Trait,
    True,
    Type,
    While,
    Underscore,
    Use,
}

pub static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "as" => Keyword::As,
    "break" => Keyword::Break,
    "continue" => Keyword::Continue,
    "else" => Keyword::Else,
    "enum" => Keyword::Enum,
    "extern" => Keyword::Extern,
    "false" => Keyword::False,
    "fn" => Keyword::Fn,
    "for" => Keyword::For,
    "if" => Keyword::If,
    "in" => Keyword::In,
    "let" => Keyword::Let,
    "mod" => Keyword::Mod,
    "mut" => Keyword::Mut,
    "match" => Keyword::Match,
    "pub" => Keyword::Pub,
    "return" => Keyword::Return,
    "struct" => Keyword::Struct,
    "this" => Keyword::This,
    "trait" => Keyword::Trait,
    "true" => Keyword::True,
    "type" => Keyword::Type,
    "while" => Keyword::While,
    "_" => Keyword::Underscore,
    "use" => Keyword::Use,
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Symbol {
    Equals,
    EqualsEquals,
    ExclamationMark,
    ExclamationMarkEquals,
    LeftAngle,
    LeftAngleEquals,
    LeftAngleLeftAngle,
    LeftAngleLeftAngleEquals,
    RightAngle,
    RightAngleEquals,
    RightAngleRightAngle,
    RightAngleRightAngleEquals,
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    MinusRightAngle,
    Star,
    StarEquals,
    Percent,
    PercentEquals,
    Slash,
    SlashEquals,
    StarStar,
    StarStarEquals,
    Pipe,
    PipeEquals,
    Ampersand,
    AmpersandEquals,
    Caret,
    CaretEquals,
    PipePipe,
    PipePipeEquals,
    AmpersandAmpersand,
    AmpersandAmpersandEquals,
    Dot,
    QuestionMark,
    QuestionMarkDot,
    Comma,
    Colon,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    At,
    AtExclamationMark,
}

pub trait TokenIterator<'a> {
    fn next_token(&mut self) -> Result<Span<Token<'a>>, Error>;
    
    // /// Returns a view of all warnings gathered so far.
    // fn warnings(&self) -> &[Span<Warning>];
    
    // /// Returns a mutable reference to the warnings.
    // fn warnings_mut(&mut self) -> &mut Vec<Span<Warning>>;
    
    // /// Consumes the iterator.
    // fn consume_warnings(self) -> Vec<Span<Warning>>;
}