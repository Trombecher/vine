mod bp;
mod error;

use std::{iter::Peekable, ops::Range};

pub use error::*;

use parser_tools::Span;
use vine_lex::{FilteredToken, FilteredTokenKind};

use crate::{
    ast::{BinaryOperation, Expression, MatchCase},
    parser::bp::BindingPrecedence,
};

pub struct Parser<'source, Tokens: Iterator<Item = Span<FilteredToken<'source>>>> {
    tokens: Peekable<Tokens>,
}

macro_rules! bail {
    ($found:expr, $message:literal) => {
        return Err(Box::new(ErrorInfo {
            found: $found,
            message: $message,
        }))
    };
}

/// Determines whether a token can start an expression.
fn token_kind_can_start_expression(token: &FilteredTokenKind) -> bool {
    matches!(
        token,
        FilteredTokenKind::Number(_)
            | FilteredTokenKind::Function
            | FilteredTokenKind::If
            | FilteredTokenKind::Identifier(_)
            | FilteredTokenKind::OpeningBracket
            | FilteredTokenKind::OpeningBrace
            | FilteredTokenKind::OpeningParenthesis
            | FilteredTokenKind::Ampersand
            | FilteredTokenKind::Match
    )
}

impl<'source, Tokens: Iterator<Item = Span<FilteredToken<'source>>>> Parser<'source, Tokens> {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse_root_expression(&mut self) -> Result<Span<Expression<'source>>, Error<'source>> {
        let expression = self.parse_expression(BindingPrecedence::Lowest, false)?;

        match self.tokens.next() {
            None => Ok(expression),
            token => bail!(token, "no token"),
        }
    }

    fn parse_expression(
        &mut self,
        min_bp: BindingPrecedence,
        line_break_as_delimiter: bool,
    ) -> Result<Span<Expression<'source>>, Error<'source>> {
        let mut left = self.parse_expression_start(line_break_as_delimiter)?;

        macro_rules! binary_operator {
            ($bp_right:expr, $operation:expr) => {{
                self.tokens.next();

                let right = self.parse_expression($bp_right, line_break_as_delimiter)?;
                let range = left.range.start..right.range.end;

                Span {
                    value: Expression::Binary {
                        left: Box::new(left),
                        operation: $operation,
                        right: Box::new(right),
                    },
                    range,
                }
            }};
        }

        loop {
            left = match self.tokens.peek() {
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Equals,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::AssignmentLeft => {
                    binary_operator!(
                        BindingPrecedence::AssignmentRight,
                        BinaryOperation::Definition
                    )
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Period,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::AccessLeft => {
                    binary_operator!(BindingPrecedence::AccessRight, BinaryOperation::Access)
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Plus,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::AdditiveLeft => {
                    binary_operator!(BindingPrecedence::AdditiveRight, BinaryOperation::Add)
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Minus,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::AdditiveLeft => {
                    binary_operator!(BindingPrecedence::AdditiveRight, BinaryOperation::Subtract)
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Star,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::MultiplicativeLeft => {
                    binary_operator!(
                        BindingPrecedence::MultiplicativeRight,
                        BinaryOperation::Multiply
                    )
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind: FilteredTokenKind::Slash,
                            ..
                        },
                    ..
                }) if min_bp <= BindingPrecedence::MultiplicativeLeft => {
                    binary_operator!(
                        BindingPrecedence::MultiplicativeRight,
                        BinaryOperation::Divide
                    )
                }
                Some(Span {
                    value:
                        FilteredToken {
                            kind,
                            line_break_before,
                        },
                    range,
                }) if (!*line_break_before || !line_break_as_delimiter)
                    && token_kind_can_start_expression(kind) =>
                {
                    let start = range.start;
                    let argument = self.parse_expression(BindingPrecedence::Call, false)?;

                    Span {
                        range: start..argument.range.end,
                        value: Expression::Call {
                            function: Box::new(left),
                            argument: Box::new(argument),
                        },
                    }
                }
                _ => break,
            };
        }

        Ok(left)
    }

    /// Parses a [`MatchCase`]. Expects `peek` to yield the first token of the pattern.
    fn parse_match_case(
        &mut self,
        start: u32,
        line_break_as_delimiter: bool,
    ) -> Result<Span<MatchCase<'source>>, Error<'source>> {
        let pattern = self.parse_expression(BindingPrecedence::Lowest, false)?;

        let domain = match self.tokens.peek() {
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::Is | FilteredTokenKind::In,
                        ..
                    },
                ..
            }) => {
                self.tokens.next();

                Some(self.parse_expression(BindingPrecedence::Lowest, false)?)
            }
            _ => None,
        };

        match self.tokens.next() {
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::EqualsGreaterThan,
                        ..
                    },
                ..
            }) => {}
            token => bail!(token, "expected 'is', 'in', or '=>'"),
        }

        let case_to_expression =
            self.parse_expression(BindingPrecedence::Lowest, line_break_as_delimiter)?;

        Ok(Span {
            range: start..case_to_expression.range.end,
            value: MatchCase {
                pattern: Box::new(pattern),
                domain: domain.map(Box::new),
                maps_to: Box::new(case_to_expression),
            },
        })
    }

    fn parse_expression_start(
        &mut self,
        line_break_as_delimiter: bool,
    ) -> Result<Span<Expression<'source>>, Error<'source>> {
        Ok(match self.tokens.next() {
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::Number(n),
                        ..
                    },
                range,
            }) => Span {
                value: Expression::Number(n),
                range,
            },
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::Identifier(identifier),
                        ..
                    },
                range,
            }) => Span {
                value: Expression::Identifier(identifier),
                range,
            },
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::Match,
                        ..
                    },
                range: Range { start, .. },
            }) => {
                let expression_to_match_on =
                    self.parse_expression(BindingPrecedence::Lowest, false)?;

                let case_start_index = match self.tokens.next() {
                    Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::Case,
                                ..
                            },
                        range,
                    }) => range.start,
                    token => bail!(
                        token,
                        "expected 'case'; match expressions must always have at least one case"
                    ),
                };

                let first_case =
                    self.parse_match_case(case_start_index, line_break_as_delimiter)?;

                let mut other_cases = Vec::new();

                loop {
                    let case_start = match self.tokens.peek() {
                        Some(Span {
                            value:
                                FilteredToken {
                                    kind: FilteredTokenKind::Case,
                                    ..
                                },
                            range,
                        }) => range.start,
                        _ => break,
                    };

                    self.tokens.next();

                    other_cases.push(self.parse_match_case(case_start, line_break_as_delimiter)?);
                }

                Span {
                    range: start..other_cases.last().unwrap_or(&first_case).range.end,
                    value: Expression::Match {
                        on: Box::new(expression_to_match_on),
                        first_case,
                        other_cases,
                    },
                }
            }
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::Function,
                        ..
                    },
                range,
            }) => {
                let parameter_pattern = self.parse_expression(BindingPrecedence::Lowest, false)?;

                match self.tokens.next() {
                    Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::Is | FilteredTokenKind::In,
                                ..
                            },
                        ..
                    }) => {}
                    token => bail!(token, "expected 'is' or 'in'"),
                }

                let domain = self.parse_expression(BindingPrecedence::Lowest, false)?;

                match self.tokens.next() {
                    Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::EqualsGreaterThan,
                                ..
                            },
                        ..
                    }) => {}
                    token @ Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::Then,
                                ..
                            },
                        ..
                    }) => bail!(
                        token,
                        "expected '=>'. functions don't use 'then' they use '=>'"
                    ),
                    token => bail!(token, "expected '=>'"),
                }

                let body = self.parse_expression(BindingPrecedence::Lowest, false)?;

                Span {
                    range: range.start..body.range.end,
                    value: Expression::Function {
                        parameter_pattern: Box::new(parameter_pattern),
                        parameter_domain: Box::new(domain),
                        body: Box::new(body),
                    },
                }
            }
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::OpeningParenthesis,
                        ..
                    },
                range: Range { start, .. },
            }) => self.parse_structure(start)?,
            token => bail!(
                token,
                "expected '(', '!', '-', a number, a string, a character, '{', 'function', or '('"
            ),
        })
    }

    fn parse_structure(&mut self, start: u32) -> Result<Span<Expression<'source>>, Error<'source>> {
        Ok(match self.tokens.peek() {
            Some(Span {
                value:
                    FilteredToken {
                        kind: FilteredTokenKind::ClosingParenthesis,
                        ..
                    },
                range,
            }) => {
                // ()

                let end = range.end;
                self.tokens.next(); // Skip ')'

                Span {
                    value: Expression::Structure { fields: Vec::new() },
                    range: start..end,
                }
            }
            _ => {
                let first = self.parse_expression(BindingPrecedence::Lowest, true)?;

                match self.tokens.peek() {
                    Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::ClosingParenthesis,
                                ..
                            },
                        range: Range { end, .. },
                    }) => {
                        // (expression)
                        let end = *end;

                        self.tokens.next(); // Skip ')'

                        match first {
                            Span {
                                value:
                                    Expression::Binary {
                                        ref left,
                                        operation: BinaryOperation::Definition,
                                        ..
                                    },
                                ..
                            } if let Span {
                                value: Expression::Identifier(_),
                                ..
                            } = left.as_ref() =>
                            {
                                // (identifier = expression)

                                Span {
                                    value: Expression::Structure {
                                        fields: vec![first],
                                    },
                                    range: start..end,
                                }
                            }
                            first => Span {
                                value: Expression::Parenthesized(Box::new(first)),
                                range: start..end,
                            },
                        }
                    }
                    Some(Span {
                        value:
                            FilteredToken {
                                line_break_before: true,
                                ..
                            }
                            | FilteredToken {
                                kind: FilteredTokenKind::Comma,
                                ..
                            },
                        ..
                    }) => {
                        // We're expecting more fields.

                        if let Some(Span {
                            value:
                                FilteredToken {
                                    kind: FilteredTokenKind::Comma,
                                    ..
                                },
                            ..
                        }) = self.tokens.peek()
                        {
                            // Skip ','
                            self.tokens.next();
                        }

                        let mut fields = vec![first];

                        loop {
                            match self.tokens.peek() {
                                Some(Span {
                                    value:
                                        FilteredToken {
                                            kind: FilteredTokenKind::ClosingParenthesis,
                                            ..
                                        },
                                    ..
                                }) => break,
                                _ => {}
                            }

                            fields.push(self.parse_expression(BindingPrecedence::Lowest, true)?);

                            match self.tokens.peek() {
                                Some(Span {
                                    value:
                                        FilteredToken {
                                            kind: FilteredTokenKind::Comma,
                                            ..
                                        },
                                    ..
                                }) => {
                                    self.tokens.next();
                                }
                                Some(Span {
                                    value:
                                        FilteredToken {
                                            line_break_before: true,
                                            ..
                                        }
                                        | FilteredToken {
                                            kind: FilteredTokenKind::ClosingParenthesis,
                                            ..
                                        },
                                    ..
                                }) => {
                                    // Expression terminated because of line break.
                                    // A ')' will be handled in the next iteration.
                                }
                                token => {
                                    bail!(token.cloned(), "expected a line break, ',', or ')'")
                                }
                            }
                        }

                        // Skip ')'
                        self.tokens.next();

                        Span {
                            range: start..fields.last().unwrap().range.end,
                            value: Expression::Structure { fields: fields },
                        }
                    }
                    token => bail!(
                        token.cloned(),
                        "expected ')', ',', a line break, or an expression"
                    ),
                }
            }
        })
    }
}
