mod tokens;

pub use tokens::*;

use core::{iter::Peekable, ops::Range};

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
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    match self.tokens.peek() {
                        Some(Span {
                            value: Token::Equals,
                            range: Range { end, .. },
                        }) => {
                            let end = *end;
                            self.tokens.next();

                            Span {
                                value: FilteredTokenKind::EqualsEqualsEquals,
                                range: start..end,
                            }
                        }
                        _ => Span {
                            value: FilteredTokenKind::EqualsEquals,
                            range: start..end,
                        },
                    }
                }
                Some(Span {
                    value: Token::GreaterThan,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::EqualsGreaterThan,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Equals,
                    range: start..end,
                },
            },
            Span {
                value: Token::LessThan,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::LessThanEquals,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::Minus,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::LessThanMinus,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::LessThan,
                    range: start..end,
                },
            },
            Span {
                value: Token::GreaterThan,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::GreaterThanEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::GreaterThan,
                    range: start..end,
                },
            },
            Span {
                value: Token::ExclamationMark,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    match self.tokens.peek() {
                        Some(Span {
                            value: Token::Equals,
                            range: Range { end, .. },
                        }) => {
                            let end = *end;
                            self.tokens.next();

                            Span {
                                value: FilteredTokenKind::ExclamationMarkEqualsEquals,
                                range: start..end,
                            }
                        }
                        _ => Span {
                            value: FilteredTokenKind::ExclamationMarkEqualsEquals,
                            range: start..end,
                        },
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::ExclamationMark,
                    range: start..end,
                },
            },
            Span {
                value: Token::Plus,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Plus,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::PlusPlus,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::PlusEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Plus,
                    range: start..end,
                },
            },
            Span {
                value: Token::Minus,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Minus,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::MinusMinus,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::MinusEquals,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::GreaterThan,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::MinusGreaterThan,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Minus,
                    range: start..end,
                },
            },
            Span {
                value: Token::Star,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Star,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::StarStar,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::StarEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Star,
                    range: start..end,
                },
            },
            Span {
                value: Token::Slash,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::SlashEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Slash,
                    range: start..end,
                },
            },
            Span {
                value: Token::Bar,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Bar,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    match self.tokens.peek() {
                        Some(Span {
                            value: Token::Equals,
                            range: Range { end, .. },
                        }) => {
                            let end = *end;
                            self.tokens.next();

                            Span {
                                value: FilteredTokenKind::BarBarEquals,
                                range: start..end,
                            }
                        }
                        _ => Span {
                            value: FilteredTokenKind::BarBar,
                            range: start..end,
                        },
                    }
                }
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::BarEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Bar,
                    range: start..end,
                },
            },
            Span {
                value: Token::Period,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Period,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    match self.tokens.peek() {
                        Some(Span {
                            value: Token::Equals,
                            range: Range { end, .. },
                        }) => {
                            let end = *end;
                            self.tokens.next();

                            Span {
                                value: FilteredTokenKind::PeriodPeriodEquals,
                                range: start..end,
                            }
                        }
                        _ => Span {
                            value: FilteredTokenKind::PeriodPeriod,
                            range: start..end,
                        },
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Period,
                    range: start..end,
                },
            },
            Span {
                value: Token::Ampersand,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Ampersand,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    match self.tokens.peek() {
                        Some(Span {
                            value: Token::Equals,
                            range: Range { end, .. },
                        }) => {
                            let end = *end;
                            self.tokens.next();

                            Span {
                                value: FilteredTokenKind::AmpersandAmpersandEquals,
                                range: start..end,
                            }
                        }
                        _ => Span {
                            value: FilteredTokenKind::AmpersandAmpersand,
                            range: start..end,
                        },
                    }
                }
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::AmpersandAmpersandEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Ampersand,
                    range: start..end,
                },
            },
            Span {
                value: Token::Colon,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::ColonEquals,
                        range: start..end,
                    }
                }
                Some(Span {
                    value: Token::Colon,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::ColonColon,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Colon,
                    range: start..end,
                },
            },
            Span {
                value: Token::Caret,
                range: Range { start, end },
            } => match self.tokens.peek() {
                Some(Span {
                    value: Token::Equals,
                    range: Range { end, .. },
                }) => {
                    let end = *end;
                    self.tokens.next();

                    Span {
                        value: FilteredTokenKind::CaretEquals,
                        range: start..end,
                    }
                }
                _ => Span {
                    value: FilteredTokenKind::Caret,
                    range: start..end,
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
