use std::fmt::Debug;
use phf::phf_map;
use error::Error;
use crate::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Char(char),
    Identifier(&'a str),
    Number(f64),
    DocComment(&'a str),
    LineComment(&'a str),
    Symbol(Symbol),
    Keyword(Keyword),

    /// The contained text needs processing:
    ///
    /// - Validate escape sequences
    /// - Normalize line breaks
    String(&'a str),

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
    
    // Optional primitive types:
    
    // Num,
    // Str,
    // Bool,
    // Char,
    // Object,
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
    fn next_token(&mut self) -> Result<Span<'a, Token<'a>>, Error>;
}