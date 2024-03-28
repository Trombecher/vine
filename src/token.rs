use phf::phf_map;
use crate::markup;

#[derive(Debug)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

#[derive(Debug)]
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    DocComment(String),
    LineComment(String),
    Symbol(Symbol),
    Keyword(Keyword),
    String(String),
    EndOfInput,
    MarkupElement(markup::Element),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Keyword {
    As,
    Await,
    Async,
    Break,
    Class,
    Continue,
    Else,
    Enum,
    Fn,
    For,
    If,
    Mut,
    Match,
    Nil,
    Return,
    This,
    ThisCase,
    Type,
    While,
    Use,
}

static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "as" => Keyword::As,
    "await" => Keyword::Await,
    "async" => Keyword::Async,
    "break" => Keyword::Break,
    "class" => Keyword::Class,
    "continue" => Keyword::Continue,
    "else" => Keyword::Else,
    "enum" => Keyword::Enum,
    "fn" => Keyword::Fn,
    "for" => Keyword::For,
    "if" => Keyword::If,
    "mut" => Keyword::Mut,
    "match" => Keyword::Match,
    "nil" => Keyword::Nil,
    "return" => Keyword::Return,
    "this" => Keyword::This,
    "This" => Keyword::ThisCase,
    "type" => Keyword::Type,
    "while" => Keyword::While,
    "use" => Keyword::Use,
};

impl<'a> TryFrom<&'a str> for Keyword {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        KEYWORDS.get(value).copied().ok_or(())
    }
}

#[derive(Copy, Clone, Debug)]
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
    QuestionMarkDot,
    QuestionMarkColon,
    Comma,
    Colon,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}