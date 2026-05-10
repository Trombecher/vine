use crate::Token;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilteredToken<'source> {
    /// An identifer or a keyword.
    Identifier(&'source str),

    /// An invalid token
    Invalid(&'source str),

    /// A unicode character `'x'`.
    Character(char),

    // TODO: change that to fraction
    Number(u64),

    /// `;`
    Semicolon,

    /// `@`
    At,

    /// `^`
    Caret,

    /// `,`
    Comma,

    /// `=`
    Equals,

    /// `==`
    EqualsEquals,

    /// `=>`
    EqualsGreaterThan,

    /// `<`
    LessThan,

    /// `<=`
    LessThanEquals,

    /// `>`
    GreaterThan,

    /// `>=`
    GreaterThanEquals,

    /// `(`
    OpeningParenthesis,

    /// `)`
    ClosingParenthesis,

    /// `{`
    OpeningBrace,

    /// `}`
    ClosingBrace,

    /// `[`
    OpeningBracket,

    /// `]`
    ClosingBracket,

    /// `+`
    Plus,

    /// `-`
    Minus,

    /// `*`
    Star,

    /// `/`
    Slash,

    /// `/=`
    SlashEquals,

    /// `|`
    Bar,

    /// `.`
    Period,

    /// `&`
    Ampersand,

    /// `!`
    ExclamationMark,

    /// `!=`
    ExclamationMarkEquals,

    /// Keyword `function`
    Function,

    /// Keyword `match`
    Match,

    /// Keyword `if`
    If,

    /// Keyword `then`
    Then,

    /// Keyword `else`
    Else,

    /// Keyword `return`
    Return,

    /// Keyword `set`
    Set,

    /// Keyword `block`
    Block,

    /// Keyword `leave`
    Leave,

    /// Keyword `public`
    Public,
}

impl<'source> FilteredToken<'source> {
    pub fn try_from_trivial(token: &Token<'source>) -> Option<Self> {
        match token {
            Token::Ampersand => Some(Self::Ampersand),
            Token::At => Some(Self::At),
            Token::Caret => Some(Self::Caret),
            Token::Comma => Some(Self::Comma),
            Token::OpeningParenthesis => Some(Self::OpeningParenthesis),
            Token::ClosingParenthesis => Some(Self::ClosingParenthesis),
            Token::OpeningBrace => Some(Self::OpeningBrace),
            Token::ClosingBrace => Some(Self::ClosingBrace),
            Token::OpeningBracket => Some(Self::OpeningBracket),
            Token::ClosingBracket => Some(Self::ClosingBracket),
            Token::Plus => Some(Self::Plus),
            Token::Hypen => Some(Self::Minus),
            Token::Star => Some(Self::Star),
            Token::Bar => Some(Self::Bar),
            Token::Period => Some(Self::Period),
            Token::Semicolon => Some(Self::Semicolon),
            Token::IdentifierOrKeyword("function") => Some(Self::Function),
            Token::IdentifierOrKeyword("block") => Some(Self::Block),
            Token::IdentifierOrKeyword("set") => Some(Self::Set),
            Token::IdentifierOrKeyword("return") => Some(Self::Return),
            Token::IdentifierOrKeyword("leave") => Some(Self::Leave),
            Token::IdentifierOrKeyword("public") => Some(Self::Public),
            Token::IdentifierOrKeyword("if") => Some(Self::If),
            Token::IdentifierOrKeyword("then") => Some(Self::Then),
            Token::IdentifierOrKeyword("else") => Some(Self::Else),
            Token::IdentifierOrKeyword("match") => Some(Self::Match),
            Token::Number(n) => Some(Self::Number(n.parse())),
            _ => None,
        }
    }
}
