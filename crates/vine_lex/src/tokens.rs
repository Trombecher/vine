use core::fmt::Debug;

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) len: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// Some whitespace.
    WhiteSpace {
        /// `true` if the whitespace contains one or more line breaks.
        line_break: bool,
    },

    /// `;`
    Semicolon,

    /// `@`
    At,

    /// An identifer or a keyword.
    IdentifierOrKeyword,

    /// `^`
    Caret,

    /// `,`
    Comma,

    /// `=`
    Equals,

    /// `<`
    LessThan,

    /// `>`
    GreaterThan,

    /// `(`
    OpenParenthesis,

    /// `)`
    CloseParenthesis,

    /// `{`
    OpenBrace,

    /// `}`
    CloseBrace,

    /// `[`
    OpenBracket,

    /// `]`
    CloseBracket,

    /// `+`
    Plus,

    /// `-`
    Hypen,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `|`
    Bar,

    /// `.`
    Period,

    /// `&`
    Ampersand,

    /// A unicode character `'x'`.
    Character,

    Number,

    Invalid,

    /// `!`
    ExclamationMark,
}
