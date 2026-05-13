use core::hint::unreachable_unchecked;

use parser_tools::TokenLength;

#[cfg(test)]
mod tests;

#[derive(Clone, PartialEq, Debug)]
pub enum Token<'source> {
    /// Some whitespace.
    Whitespace(WhitespaceSource<'source>),

    /// An identifer or a keyword.
    IdentifierOrKeyword(&'source str),

    /// A number
    Number(NumberSource<'source>),

    /// An invalid token
    Invalid(&'source str),

    /// A unicode character `'x'`.
    Character(CharacterSource<'source>),

    /// A line comment or a block comment.
    Comment(&'source str),

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

    /// `$`
    DollarSign,

    /// `%`
    Percent,

    /// `§`
    Paragraph,

    /// `?`
    QuestionMark,

    /// `#`
    Hashtag,

    /// `~`
    Tilde,

    /// ` ` `
    Backtick,

    /// `:`
    Colon,
}

impl TokenLength for Token<'_> {
    fn length(&self) -> u32 {
        match self {
            Self::Whitespace(source) => source.as_str().len() as u32,
            Self::IdentifierOrKeyword(source) => source.len() as u32,
            Self::Number(source) => source.as_str().len() as u32,
            Self::Invalid(source) => source.len() as u32,
            Self::Character(source) => source.as_str().len() as u32,
            Self::String(source) => source.as_str().len() as u32,
            Self::Comment(source) => source.len() as u32,
            Self::ExclamationMark
            | Self::Semicolon
            | Self::DollarSign
            | Self::Percent
            | Self::Paragraph
            | Self::Colon
            | Self::QuestionMark
            | Self::Tilde
            | Self::Hashtag
            | Self::Backtick
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
pub struct NumberSource<'source>(&'source str);

impl_source_for!(NumberSource);

impl NumberSource<'_> {
    pub fn parse(self) -> u64 {
        let mut bytes = self.as_str().bytes();

        let mut number = match bytes.next() {
            Some(start @ b'0'..=b'9') => (start - b'0') as u64,
            _ => unsafe { unreachable_unchecked() },
        };

        loop {
            match bytes.next() {
                Some(n @ b'0'..=b'9') => {
                    // TODO: refine this.

                    number = number
                        .checked_add((n - b'0') as u64)
                        .expect("number to big")
                }
                Some(b'_') => {}
                Some(b'.') => todo!("decimals not implemented"),
                _ => break,
            }
        }

        number
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CharacterSource<'source>(&'source str);

impl_source_for!(CharacterSource);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WhitespaceSource<'source>(&'source str);

impl_source_for!(WhitespaceSource);

impl WhitespaceSource<'_> {
    pub fn contains_a_line_break(self) -> bool {
        self.0.contains(|c| c == '\n' || c == '\r')
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StringSource<'source>(&'source str);

impl_source_for!(StringSource);
