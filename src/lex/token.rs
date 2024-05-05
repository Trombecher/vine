use std::fmt::Debug;
use phf::phf_map;

#[derive(Debug)]
pub struct Span<T> where T: Debug {
    pub value: T,
    pub start: usize,
    pub end: usize,
}

impl<T: Debug> Span<T> {
    #[inline]
    pub fn map<U, F: Fn(T) -> U>(self, mapper: F) -> Span<U> where U: Debug {
        Span {
            value: mapper(self.value),
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Char(char),
    Identifier(&'a str),
    Number(f64),
    DocComment(&'a str),
    LineComment(&'a str),
    Symbol(Symbol),
    Keyword(Keyword),
    String(&'a str),
    EndOfInput,
    MarkupStartTag(&'a str),
    MarkupKey(&'a str),
    MarkupStartTagEnd,
    MarkupClose,
    MarkupText(&'a str),
    MarkupEndTag(&'a str),
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Keyword {
    As,
    Await,
    Async,
    Break,
    Class,
    Continue,
    Else,
    Enum,
    False,
    Fn,
    For,
    If,
    In,
    Let,
    Mod,
    Mut,
    Match,
    Nil,
    Pub,
    Return,
    This,
    ThisCase,
    True,
    Type,
    While,
    Underscore,
    Use,
}

pub static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "as" => Keyword::As,
    "await" => Keyword::Await,
    "async" => Keyword::Async,
    "break" => Keyword::Break,
    "class" => Keyword::Class,
    "continue" => Keyword::Continue,
    "else" => Keyword::Else,
    "enum" => Keyword::Enum,
    "false" => Keyword::False,
    "fn" => Keyword::Fn,
    "for" => Keyword::For,
    "if" => Keyword::If,
    "in" => Keyword::In,
    "let" => Keyword::Let,
    "mod" => Keyword::Mod,
    "mut" => Keyword::Mut,
    "match" => Keyword::Match,
    "nil" => Keyword::Nil,
    "pub" => Keyword::Pub,
    "return" => Keyword::Return,
    "this" => Keyword::This,
    "This" => Keyword::ThisCase,
    "true" => Keyword::True,
    "type" => Keyword::Type,
    "while" => Keyword::While,
    "use" => Keyword::Use,
    "_" => Keyword::Underscore
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
    At
}