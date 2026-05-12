mod bp;
mod error;

use std::iter::Peekable;

pub use error::*;

use parser_tools::Span;
use vine_lex::{FilteredToken, FilteredTokenKind};

use crate::{
    ast::{BinaryOperation, Expression},
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

impl<'source, Tokens: Iterator<Item = Span<FilteredToken<'source>>>> Parser<'source, Tokens> {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse_root_expression(&mut self) -> Result<Span<Expression<'source>>, Error<'source>> {
        let expression = self.parse_expression(BindingPrecedence::Lowest)?;

        match self.tokens.next() {
            None => Ok(expression),
            token => bail!(token, "no token"),
        }
    }

    fn parse_expression(
        &mut self,
        min_bp: BindingPrecedence,
    ) -> Result<Span<Expression<'source>>, Error<'source>> {
        let mut left = self.parse_expression_start()?;

        macro_rules! binary_operator {
            ($bp_right:expr, $operation:expr) => {{
                self.tokens.next();

                let right = self.parse_expression($bp_right)?;
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
                            kind:
                                FilteredTokenKind::Number(_)
                                | FilteredTokenKind::Function
                                | FilteredTokenKind::If
                                | FilteredTokenKind::Identifier(_)
                                | FilteredTokenKind::OpeningParenthesis
                                | FilteredTokenKind::Ampersand
                                | FilteredTokenKind::OpeningBracket
                                | FilteredTokenKind::OpeningBrace,
                            ..
                        },
                    range,
                }) => {
                    // This is the start of a new token.

                    let start = range.start;
                    let argument = self.parse_expression(BindingPrecedence::Call)?;

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

    fn parse_expression_start(&mut self) -> Result<Span<Expression<'source>>, Error<'source>> {
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
                        kind: FilteredTokenKind::Function,
                        ..
                    },
                range,
            }) => {
                let parameter_pattern = self.parse_expression(BindingPrecedence::Lowest)?;

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

                let domain = self.parse_expression(BindingPrecedence::Lowest)?;

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

                let body = self.parse_expression(BindingPrecedence::Lowest)?;

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
                ..
            }) => {
                let inner = self.parse_expression(BindingPrecedence::Lowest)?;

                match self.tokens.next() {
                    Some(Span {
                        value:
                            FilteredToken {
                                kind: FilteredTokenKind::ClosingParenthesis,
                                ..
                            },
                        ..
                    }) => {}
                    token => bail!(token, "expected ')'"),
                }

                inner
            }
            token => bail!(
                token,
                "expected '(', '!', '-', a number, a string, a character, '{', 'function', or '('"
            ),
        })
    }
}
