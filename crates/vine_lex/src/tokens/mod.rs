#[cfg(test)]
mod tests;

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'source> {
    /// Some whitespace.
    Whitespace(WhitespaceSource<'source>),

    /// An identifer or a keyword.
    IdentifierOrKeyword(IdentifierOrKeywordSource<'source>),

    /// A number
    Number(NumberSource<'source>),

    /// An invalid token
    Invalid(&'source str),

    /// A unicode character `'x'`.
    Character(CharacterSource<'source>),

    String(StringSource<'source>),

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

    /// `<`
    LessThan,

    /// `>`
    GreaterThan,

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

    /// `!`
    ExclamationMark,
}

impl<'source> Token<'source> {
    /// Returns the length in bytes of this token.
    pub fn length(&self) -> usize {
        match self {
            Self::Whitespace(source) => source.as_str().len(),
            Self::IdentifierOrKeyword(source) => source.as_str().len(),
            Self::Number(source) => source.as_str().len(),
            Self::Invalid(source) => source.len(),
            Self::Character(source) => source.as_str().len(),
            Self::String(source) => source.as_str().len(),
            Self::ExclamationMark
            | Self::Semicolon
            | Self::OpeningBracket
            | Self::ClosingBracket
            | Self::OpeningBrace
            | Self::ClosingBrace
            | Self::Equals
            | Self::LessThan
            | Self::Comma
            | Self::Caret
            | Self::At
            | Self::GreaterThan
            | Self::OpeningParenthesis
            | Self::ClosingParenthesis
            | Self::Ampersand
            | Self::Period
            | Self::Bar
            | Self::Slash
            | Self::Star
            | Self::Plus
            | Self::Hypen => 1,
        }
    }
}

macro_rules! impl_source_for {
    ($T:ident) => {
        impl<'source> $T<'source> {
            pub const fn as_str(self) -> &'source str {
                self.0
            }

            pub const unsafe fn new_unchecked(input: &'source str) -> Self {
                Self(input)
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IdentifierOrKeywordSource<'source>(&'source str);

impl_source_for!(IdentifierOrKeywordSource);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NumberSource<'source>(&'source str);

impl_source_for!(NumberSource);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CharacterSource<'source>(&'source str);

impl_source_for!(CharacterSource);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WhitespaceSource<'source>(&'source str);

impl_source_for!(WhitespaceSource);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StringSource<'source>(&'source str);

impl_source_for!(StringSource);
