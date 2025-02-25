use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use byte_reader::Cursor;
use span::Span;
use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use derive_where::derive_where;
use phf::phf_map;

use crate::lex::{unescape_char, Error};

/// This type solely exists because [Clone] is not implemented for `Box<str, A>` (which is ridiculous).
#[derive_where(PartialEq, Clone)]
pub struct BoxStr<A: Allocator + Clone>(Box<[u8], A>);

impl<A: Allocator + Clone> BoxStr<A> {
    #[inline]
    pub fn unbox(self) -> Box<str, A> {
        let (raw, alloc) = Box::into_raw_with_allocator(self.0);
        unsafe { Box::from_raw_in(raw as *mut str, alloc) }
    }
}

impl<A: Allocator + Clone> From<Box<str, A>> for BoxStr<A> {
    #[inline]
    fn from(value: Box<str, A>) -> Self {
        Self(value.into())
    }
}

impl<A: Allocator + Clone> Debug for BoxStr<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Steal the Debug impl from Box<str, A>
        unsafe { transmute::<_, &Box<str, A>>(self) }.fmt(f)
    }
}

/// Boxes an `&str` which has some characteristics it must obey.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnprocessedString<'source>(&'source str);

impl<'source> UnprocessedString<'source> {
    pub const unsafe fn from_raw(raw: &'source str) -> Self {
        Self(raw)
    }

    pub fn process<A: Allocator>(&self, alloc: A) -> Result<Box<str, A>, Error> {
        let mut string = Vec::with_capacity_in(self.0.len(), alloc);
        let mut cursor = Cursor::new(self.0.as_bytes());

        loop {
            match cursor.next() {
                None => break,
                Some(b'\\') => string.extend_from_slice(
                    unescape_char(&mut cursor)?
                        .encode_utf8(&mut [0; 4])
                        .as_bytes(),
                ),
                Some(byte) => string.push(byte),
            }
        }

        let (raw, alloc) = Box::into_raw_with_allocator(string.into_boxed_slice());
        
        // SAFETY: this is a string
        Ok(unsafe { Box::from_raw_in(raw as *mut str, alloc) })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Token<'a> {
    /// A unicode character (`'?'`).
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

    /// A special token that indicates a string escape (`"{}"`).
    StringEscape,
    
    /// A special token that indicates that the string continues after an escape.
    StringReturn(UnprocessedString<'a>),

    /// `<tag`
    MarkupStartTag(&'a str),
    
    /// `key`
    MarkupKey(&'a str),
    
    /// `>`
    MarkupStartTagEnd,
    
    /// `/>`
    MarkupClose,

    /// The contained text needs processing.
    ///
    /// Although the leading whitespace is removed, the trailing whitespace needs to be removed
    /// and internal whitespace needs to be collapsed.
    MarkupText(&'a str),

    /// `</tag>`
    MarkupEndTag(&'a str),
    
    /// A line break (yes, line breaks have semantic meaning).
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
    Is,
    In,
    Let,
    Mod,
    Mut,
    Match,
    Pub,
    Return,
    Struct,
    This,
    CapitalThis,
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
    DotDot,
    DotDotDot,
    DotDotEquals,
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
}
