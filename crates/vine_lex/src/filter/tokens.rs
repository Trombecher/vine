use crate::tokens::Token;

#[derive(Clone, Debug, PartialEq)]
pub struct FilteredToken<'source> {
    /// The kind.
    pub kind: FilteredTokenKind<'source>,

    /// If this token has a line break preceding it.
    pub line_break_before: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilteredTokenKind<'source> {
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

    /// `^=`
    CaretEquals,

    /// `,`
    Comma,

    /// `=`
    Equals,

    /// `==`
    EqualsEquals,

    /// `===`
    EqualsEqualsEquals,

    /// `=>`
    EqualsGreaterThan,

    /// `<`
    LessThan,

    /// `<=`
    LessThanEquals,

    /// `<-`
    LessThanMinus,

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

    /// `++`
    PlusPlus,

    /// `+=`
    PlusEquals,

    /// `-`
    Minus,

    /// `--`
    MinusMinus,

    /// `-=`
    MinusEquals,

    /// `->`
    MinusGreaterThan,

    /// `*`
    Star,

    /// `**`
    StarStar,

    /// `*=`
    StarEquals,

    /// `/`
    Slash,

    /// `/=`
    SlashEquals,

    /// `|`
    Bar,

    /// `||`
    BarBar,

    /// `|=`
    BarEquals,

    /// `||=`
    BarBarEquals,

    /// `.`
    Period,

    /// `..`
    PeriodPeriod,

    /// `..=`
    PeriodPeriodEquals,

    /// `&`
    Ampersand,

    /// `&=`
    AmpersandEquals,

    /// `&&`
    AmpersandAmpersand,

    /// `&&=`
    AmpersandAmpersandEquals,

    /// `!`
    ExclamationMark,

    /// `!=`
    ExclamationMarkEquals,

    /// `!==`
    ExclamationMarkEqualsEquals,

    /// `$`
    DollarSign,

    /// `%`
    Percent,

    /// `%=`
    PercentEquals,

    /// `§`
    Paragraph,

    /// `?`
    QuestionMark,

    /// `~`
    Tilde,

    /// ` ` `
    Backtick,

    /// `:`
    Colon,

    /// `:=`
    ColonEquals,

    /// `::`
    ColonColon,

    /// Keyword `function`
    Function,

    /// Keyword `match`
    Match,

    /// Keyword `case`
    Case,

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

    /// Keyword `is`
    Is,

    /// Keyword `in`
    In,

    /// Keyword `enum`
    Enum,

    /// Keyword `type`
    Type,

    /// Keyword `alias`
    Alias,

    /// Keyword `Self`
    BigSelf,

    /// Keyword `private`
    Private,

    /// Keyword `module`
    Module,

    /// Keyword `for`
    For,

    /// Keyword `loop`
    Loop,

    /// Keyword `while`
    While,

    /// Keyword `or`
    Or,

    /// Keyword `and`
    And,
}

impl<'source> FilteredTokenKind<'source> {
    /// Tries to convert a _trivial_ [`Token`] into a [`FilteredTokenKind`].
    ///
    /// Trivial tokens are those, that do not compose other filtered tokens
    /// with other tokens.
    pub fn try_from_trivial(token: &Token<'source>) -> Option<Self> {
        match token {
            Token::IdentifierOrKeyword("function") => Some(Self::Function),
            Token::IdentifierOrKeyword("match") => Some(Self::Match),
            Token::IdentifierOrKeyword("case") => Some(Self::Case),
            Token::IdentifierOrKeyword("if") => Some(Self::If),
            Token::IdentifierOrKeyword("then") => Some(Self::Then),
            Token::IdentifierOrKeyword("else") => Some(Self::Else),
            Token::IdentifierOrKeyword("return") => Some(Self::Return),
            Token::IdentifierOrKeyword("set") => Some(Self::Set),
            Token::IdentifierOrKeyword("block") => Some(Self::Block),
            Token::IdentifierOrKeyword("leave") => Some(Self::Leave),
            Token::IdentifierOrKeyword("public") => Some(Self::Public),
            Token::IdentifierOrKeyword("is") => Some(Self::Is),
            Token::IdentifierOrKeyword("in") => Some(Self::In),
            Token::IdentifierOrKeyword("enum") => Some(Self::Enum),
            Token::IdentifierOrKeyword("type") => Some(Self::Type),
            Token::IdentifierOrKeyword("alias") => Some(Self::Alias),
            Token::IdentifierOrKeyword("Self") => Some(Self::BigSelf),
            Token::IdentifierOrKeyword("private") => Some(Self::Private),
            Token::IdentifierOrKeyword("module") => Some(Self::Module),
            Token::IdentifierOrKeyword("for") => Some(Self::For),
            Token::IdentifierOrKeyword("loop") => Some(Self::Loop),
            Token::IdentifierOrKeyword("while") => Some(Self::While),
            Token::IdentifierOrKeyword("or") => Some(Self::Or),
            Token::IdentifierOrKeyword("and") => Some(Self::And),
            Token::IdentifierOrKeyword(identifier) => Some(Self::Identifier(identifier)),
            Token::Invalid(invalid) => Some(Self::Invalid(invalid)),
            Token::Character(source) => Some(Self::Character(source.parse())),
            Token::Number(n) => Some(Self::Number(n.parse())),
            Token::Semicolon => Some(Self::Semicolon),
            Token::At => Some(Self::At),
            Token::Comma => Some(Self::Comma),
            Token::OpeningParenthesis => Some(Self::OpeningParenthesis),
            Token::ClosingParenthesis => Some(Self::ClosingParenthesis),
            Token::OpeningBrace => Some(Self::OpeningBrace),
            Token::ClosingBrace => Some(Self::ClosingBrace),
            Token::OpeningBracket => Some(Self::OpeningBracket),
            Token::ClosingBracket => Some(Self::ClosingBracket),
            Token::Paragraph => Some(Self::Paragraph),
            Token::QuestionMark => Some(Self::QuestionMark),
            Token::Tilde => Some(Self::Tilde),
            Token::Backtick => Some(Self::Backtick),
            _ => None,
        }
    }
}
