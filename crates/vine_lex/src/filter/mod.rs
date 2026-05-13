mod tokens;

pub use tokens::*;

use core::iter::Peekable;

use parser_tools::{Span, Spanify};

use crate::tokens::Token;

pub struct TokenFilter<'source, Tokens: Iterator<Item = Token<'source>>> {
    tokens: Peekable<Spanify<Token<'source>, Tokens>>,
}

impl<'source, Tokens: Iterator<Item = Token<'source>>> TokenFilter<'source, Tokens> {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens: Spanify::new(tokens).peekable(),
        }
    }
}

impl<'source, Tokens: Iterator<Item = Token<'source>>> Iterator for TokenFilter<'source, Tokens> {
    type Item = Span<FilteredToken<'source>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line_break = false;

        // Skip irrelevant tokens.
        let spanned_token = loop {
            match self.tokens.next()? {
                Span {
                    value: Token::Comment(_),
                    ..
                } => {}
                Span {
                    value: Token::Whitespace(whitespace),
                    ..
                } => {
                    if whitespace.contains_a_line_break() {
                        line_break = true;
                    }
                }
                span => break span,
            }
        };

        let spanned_filtered_token_kind = match spanned_token {
            Span {
                value: token,
                range,
            } if let Some(filtered_token) = FilteredTokenKind::try_from_trivial(&token) => Span {
                value: filtered_token,
                range,
            },
            Span {
                value: Token::Equals,
                range: first_range,
            } => {
                // `=`, `=>`, or `==`.

                match self.tokens.peek() {
                    Some(Span {
                        value: Token::Equals,
                        range: second_range,
                    }) => {
                        let second_range = second_range.clone();
                        self.tokens.next();

                        Span {
                            value: FilteredTokenKind::EqualsEquals,
                            range: first_range.start..second_range.end,
                        }
                    }
                    Some(Span {
                        value: Token::GreaterThan,
                        range: second_range,
                    }) => {
                        let second_range = second_range.clone();
                        self.tokens.next();

                        Span {
                            value: FilteredTokenKind::EqualsGreaterThan,
                            range: first_range.start..second_range.end,
                        }
                    }
                    _ => Span {
                        value: FilteredTokenKind::Equals,
                        range: first_range,
                    },
                }
            }
            Span {
                value: Token::LessThan,
                range: first_range,
            } => {
                // `<` or `<=`.

                match self.tokens.peek() {
                    Some(Span {
                        value: Token::Equals,
                        range: second_range,
                    }) => {
                        let second_range = second_range.clone();
                        self.tokens.next();

                        Span {
                            value: FilteredTokenKind::LessThanEquals,
                            range: first_range.start..second_range.end,
                        }
                    }
                    _ => Span {
                        value: FilteredTokenKind::LessThan,
                        range: first_range,
                    },
                }
            }
            Span {
                value: Token::ExclamationMark,
                range: first_range,
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: second_range,
                }) => {
                    let end = second_range.end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::ExclamationMarkEquals,
                        range: first_range.start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::ExclamationMark,
                    range: first_range,
                },
            },
            Span {
                value: Token::Slash,
                range: first_range,
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: second_range,
                }) => {
                    let end = second_range.end;

                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::SlashEquals,
                        range: first_range.start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Slash,
                    range: first_range,
                },
            },
            token => unreachable!("{token:?} is not filterable"),
        };

        Some(Span {
            value: FilteredToken {
                kind: spanned_filtered_token_kind.value,
                line_break_before: line_break,
            },
            range: spanned_filtered_token_kind.range,
        })
    }
}
