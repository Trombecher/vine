use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use bumpalo::Bump;
use bytes::{Cursor, Span};
use phf::phf_map;

use crate::lex::{unescape_char, Error};
use crate::{Box, Vec};

/// This type solely exists because [Clone] is not implemented for `Box<str, A>` (which is ridiculous).
#[derive(Clone, PartialEq)]
pub struct BoxStr<'alloc>(Box<'alloc, [u8]>);

impl<'alloc> From<Box<'alloc, str>> for BoxStr<'alloc> {
    #[inline]
    fn from(value: Box<'alloc, str>) -> Self {
        Self(unsafe { transmute(value) })
    }
}

impl<'alloc> Into<Box<'alloc, str>> for BoxStr<'alloc> {
    fn into(self) -> Box<'alloc, str> {
        unsafe { transmute(self) }
    }
}

impl<'alloc> Debug for BoxStr<'alloc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Steal the Debug impl from Box<str, A>
        unsafe { transmute::<_, &Box<'alloc, str>>(self) }.fmt(f)
    }
}

/// Boxes an `&str` which has some characteristics it must obey.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UnprocessedString<'source>(&'source str);

impl<'source> UnprocessedString<'source> {
    pub const unsafe fn from_raw(slice: &'source str) -> UnprocessedString<'source> {
        UnprocessedString(slice)
    }

    pub fn process<'alloc>(&self, alloc: &'alloc Bump) -> Result<Box<'alloc, str>, Error> {
        let mut string = Vec::with_capacity_in(self.0.len(), alloc);
        let mut cursor = Cursor::new(self.0.as_bytes());

        loop {
            match cursor.next() {
                None => break,
                Some(b'\\') => string.extend_from_slice(unescape_char(&mut cursor)?.encode_utf8(&mut [0; 4]).as_bytes()),
                Some(byte) => string.push(byte)
            }
        }
        
        // SAFETY: this is a string
        Ok(unsafe { transmute::<_, Box<'alloc, str>>(string.into_boxed_slice()) })
    }
}

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum Token<'a> {
    Char(char),

    /// An identifier token. Guaranteed to match against this regex `([a-zA-Z][a-zA-Z_0-9]*)|([a-zA-Z_][a-zA-Z_0-9]+)`.
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