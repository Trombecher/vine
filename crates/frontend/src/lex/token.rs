use core::fmt::Debug;
use ecow::EcoString;
use phf::phf_map;

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
    String(EcoString),

    /// A special token that indicates a string escape (`"{}"`).
    FragmentString,

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