use core::fmt::Debug;
use ecow::EcoString;
use phf::phf_map;

#[cfg(test)]
use span::Index;

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Token<'a> {
    /// A unicode character (`'?'`).
    Char(char),

    /// An identifier token. Guaranteed to match against this regex `([a-zA-Z][a-zA-Z_0-9]*)|([a-zA-Z_][a-zA-Z_0-9]+)`.
    Identifier(&'a str),

    /// A number
    Number(f64),

    /// An annotation.
    Annotation(()),

    /// A [Symbol].
    Symbol(Symbol),

    /// A [Keyword].
    Keyword(Keyword),

    /// A string literal.
    String(EcoString),

    /// A string literal that indicates that an expression will follow.
    FragmentString(EcoString),

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

impl<'a> Token<'a> {
    /// To generate pseudo source offsets for testing purposes,
    /// we need to estimate the length of the token.
    #[cfg(test)]
    pub fn estimated_length(&self) -> Index {
        // TODO: correct when needed
        match self {
            Token::Char(x) => 2 + x.len_utf8() as Index,
            Token::Identifier(id) => id.len() as Index,
            Token::Number(n) => n.log10().ceil() as Index + 2,
            Token::Symbol(symbol) => symbol.estimated_length(),
            Token::Keyword(kw) => kw.str().len() as Index,
            Token::String(s) => s.len() as Index + 2,
            Token::FragmentString(f) => f.len() as Index,
            Token::MarkupStartTag(s) => s.len() as Index + 1,
            Token::MarkupKey(s) => s.len() as Index,
            Token::MarkupStartTagEnd => 1,
            Token::MarkupClose => 1,
            Token::MarkupText(t) => t.len() as Index,
            Token::MarkupEndTag(e) => e.len() as Index + 3,
            Token::LineBreak => 1,
            Token::Annotation(_) => 0,
        }
    }
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
    Fn,
    For,
    If,
    Is,
    In,
    Let,
    Mod,
    Mut,
    Match,
    Package,
    Pub,
    Return,
    Struct,
    This,
    CapitalThis,
    Trait,
    Type,
    While,
    Underscore,
    Use,
}

impl Keyword {
    pub fn str(self) -> &'static str {
        match self {
            Self::As => "as",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Else => "else",
            Self::Enum => "enum",
            Self::Extern => "extern",
            Self::Fn => "fn",
            Self::For => "for",
            Self::If => "if",
            Self::Is => "is",
            Self::In => "in",
            Self::Let => "let",
            Self::Mod => "mod",
            Self::Mut => "mut",
            Self::Match => "match",
            Self::Package => "package",
            Self::Pub => "pub",
            Self::Return => "return",
            Self::Struct => "struct",
            Self::This => "this",
            Self::CapitalThis => "This",
            Self::Trait => "trait",
            Self::Type => "type",
            Self::While => "while",
            Self::Underscore => "_",
            Self::Use => "use",
        }
    }
}

pub static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "as" => Keyword::As,
    "break" => Keyword::Break,
    "continue" => Keyword::Continue,
    "else" => Keyword::Else,
    "enum" => Keyword::Enum,
    "extern" => Keyword::Extern,
    "fn" => Keyword::Fn,
    "for" => Keyword::For,
    "if" => Keyword::If,
    "is" => Keyword::Is,
    "in" => Keyword::In,
    "let" => Keyword::Let,
    "mod" => Keyword::Mod,
    "mut" => Keyword::Mut,
    "match" => Keyword::Match,
    "package" => Keyword::Package,
    "pub" => Keyword::Pub,
    "return" => Keyword::Return,
    "struct" => Keyword::Struct,
    "this" => Keyword::This,
    "This" => Keyword::CapitalThis,
    "trait" => Keyword::Trait,
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
}

impl Symbol {
    #[cfg(test)]
    pub fn estimated_length(&self) -> Index {
        match self {
            Self::RightParenthesis
            | Self::ExclamationMark
            | Self::Equals
            | Self::LeftAngle
            | Self::Plus
            | Self::Minus
            | Self::Star
            | Self::Percent
            | Self::Slash
            | Self::RightAngle
            | Self::Pipe
            | Self::Caret
            | Self::Ampersand
            | Self::Dot
            | Self::QuestionMark
            | Self::Colon
            | Self::Comma
            | Self::At
            | Self::Semicolon
            | Self::LeftParenthesis
            | Self::LeftBracket
            | Self::RightBracket
            | Self::LeftBrace
            | Self::RightBrace => 1,

            Self::EqualsEquals
            | Self::ExclamationMarkEquals
            | Self::LeftAngleEquals
            | Self::LeftAngleLeftAngle
            | Self::RightAngleEquals
            | Self::RightAngleRightAngle
            | Self::PlusEquals
            | Self::MinusEquals
            | Self::MinusRightAngle
            | Self::StarEquals
            | Self::SlashEquals
            | Self::StarStar
            | Self::PercentEquals
            | Self::PipeEquals
            | Self::AmpersandEquals
            | Self::CaretEquals
            | Self::PipePipe
            | Self::AmpersandAmpersand
            | Self::DotDot => 2,

            Self::LeftAngleLeftAngleEquals
            | Self::RightAngleRightAngleEquals
            | Self::StarStarEquals
            | Self::PipePipeEquals
            | Self::AmpersandAmpersandEquals
            | Self::DotDotDot
            | Self::DotDotEquals => 3,
        }
    }
}
