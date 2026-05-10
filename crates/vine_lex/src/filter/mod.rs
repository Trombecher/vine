use core::iter::Peekable;

use parser_tools::{Span, Spanify};

use crate::{FilteredToken, Token};

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
        let span = match self.tokens.next()? {
            Span {
                value: Token::Whitespace(_),
                ..
            } => self.tokens.next()?,
            span => span,
        };

        Some(match span {
            Span {
                value: token,
                range,
            } if let Some(filtered_token) = FilteredToken::try_from_trivial(&token) => Span {
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
                            value: FilteredToken::EqualsEquals,
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
                            value: FilteredToken::EqualsGreaterThan,
                            range: first_range.start..second_range.end,
                        }
                    }
                    _ => Span {
                        value: FilteredToken::Equals,
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
                            value: FilteredToken::LessThanEquals,
                            range: first_range.start..second_range.end,
                        }
                    }
                    _ => Span {
                        value: FilteredToken::LessThan,
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
                        value: FilteredToken::ExclamationMarkEquals,
                        range: first_range.start..end,
                    }
                }
                _ => Span {
                    value: FilteredToken::ExclamationMark,
                    range: first_range,
                },
            },
            _ => unreachable!(),
        })
    }
}
