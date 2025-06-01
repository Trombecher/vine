use core::fmt::{Debug, Display, Formatter};
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

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Token::Char(c) => write!(f, "'{}'", *c),
            Token::Identifier(id) => write!(f, "{}", id),
            Token::Number(n) => write!(f, "{}", *n),
            Token::Annotation(_) => write!(f, "<todo>"),
            Token::Symbol(s) => write!(f, "{}", *s),
            Token::Keyword(k) => write!(f, "{}", k),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::FragmentString(_) => write!(f, "<todo>"),
            Token::MarkupStartTag(_) => write!(f, "<todo>"),
            Token::MarkupKey(_) => write!(f, "<todo>"),
            Token::MarkupStartTagEnd => write!(f, "<todo>"),
            Token::MarkupClose => write!(f, "<todo>"),
            Token::MarkupText(_) => write!(f, "<todo>"),
            Token::MarkupEndTag(_) => write!(f, "<todo>"),
            Token::LineBreak => write!(f, "a line break"),
        }
    }
}

impl<'a> Token<'a> {
    /// To generate pseudo source offsets for testing purposes,
    /// we need to estimate the length of the token.
    #[cfg(test)]
    pub fn estimated_length(&self) -> Index {
        // TODO: correct when needed
        match self {
            Self::Char(x) => 2 + x.len_utf8() as Index,
            Self::Identifier(id) => id.len() as Index,
            Self::Number(n) => n.log10().ceil() as Index + 2,
            Self::Symbol(symbol) => symbol.estimated_length(),
            Self::Keyword(kw) => kw.str().len() as Index,
            Self::String(s) => s.len() as Index + 2,
            Self::FragmentString(f) => f.len() as Index,
            Self::MarkupStartTag(s) => s.len() as Index + 1,
            Self::MarkupKey(s) => s.len() as Index,
            Self::MarkupStartTagEnd => 1,
            Self::MarkupClose => 1,
            Self::MarkupText(t) => t.len() as Index,
            Self::MarkupEndTag(e) => e.len() as Index + 3,
            Self::LineBreak => 1,
            Self::Annotation(_) => 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Keyword {
    Alias,
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
            Self::Alias => "alias",
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

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.str())
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
    "alias" => Keyword::Alias,
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
    EqualsRightAngle,
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

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.str())
    }
}

impl Symbol {
    pub fn str(self) -> &'static str {
        match self {
            Self::Equals => "=",
            Self::EqualsEquals => "==",
            Self::EqualsRightAngle => "=>",
            Self::ExclamationMark => "!",
            Self::ExclamationMarkEquals => "!=",
            Self::LeftAngle => ">",
            Self::LeftAngleEquals => ">=",
            Self::LeftAngleLeftAngle => ">>",
            Self::LeftAngleLeftAngleEquals => ">>=",
            Self::RightAngle => "<",
            Self::RightAngleEquals => "<=",
            Self::RightAngleRightAngle => "<<",
            Self::RightAngleRightAngleEquals => "<<=",
            Self::Plus => "+",
            Self::PlusEquals => "+=",
            Self::Minus => "-",
            Self::MinusEquals => "-=",
            Self::MinusRightAngle => "->",
            Self::Star => "*",
            Self::StarEquals => "*=",
            Self::Percent => "%",
            Self::PercentEquals => "%=",
            Self::Slash => "/",
            Self::SlashEquals => "/=",
            Self::StarStar => "**",
            Self::StarStarEquals => "**=",
            Self::Pipe => "|",
            Self::PipeEquals => "|=",
            Self::Ampersand => "&",
            Self::AmpersandEquals => "&=",
            Self::Caret => "^",
            Self::CaretEquals => "^=",
            Self::PipePipe => "||",
            Self::PipePipeEquals => "||=",
            Self::AmpersandAmpersand => "&&",
            Self::AmpersandAmpersandEquals => "&&=",
            Self::Dot => ".",
            Self::DotDot => "..",
            Self::DotDotDot => "...",
            Self::DotDotEquals => "..=",
            Self::QuestionMark => "?",
            Self::Comma => ".",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::LeftParenthesis => "(",
            Self::RightParenthesis => ")",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::At => "@",
        }
    }
    
    #[cfg(test)]
    pub fn estimated_length(&self) -> Index {
        self.str().len() as Index
    }
}
