mod unit_tests;
mod warnings;

pub mod ast;
pub mod bp;
mod labext;
mod type_bp;

use crate::lex::{Keyword, Lexer, Symbol, Token};
use crate::parse::labext::LabExt;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::ops::Range;
use ecow::EcoString;
use errors::*;
use fallible_iterator::FallibleIterator;
use labuf::LookaheadBuffer;
use span::{Index, Span};
pub use warnings::*;

pub struct ParseContext<
    'source,
    I: FallibleIterator<Item = Span<Token<'source>>, Error = Error>,
    A: Allocator + Clone,
> {
    pub iter: LookaheadBuffer<I, A>,
    pub warnings: Vec<Span<Warning>, A>,
    pub alloc: A,
}

impl<
        'source,
        I: FallibleIterator<Item = Span<Token<'source>>, Error = Error>,
        A: Allocator + Clone,
    > ParseContext<'source, I, A>
{
    #[inline]
    pub fn new(iter: LookaheadBuffer<I, A>, alloc: A) -> Self {
        Self {
            warnings: Vec::new_in(alloc.clone()),
            iter,
            alloc,
        }
    }

    /// Adds a warning with the span of the token returned by [Buffered::peek].
    #[inline]
    fn emit_single_token_warning(&mut self, warning: Warning) -> Result<(), Error> {
        let new_source = self.iter.peek()?.unwrap().source.clone();

        // If there is a last added warning that is equal to the new warning and has is extendable,
        // extend it!

        match (warning.is_extendable(), self.warnings.last_mut()) {
            (true, Some(Span { value, source })) if *value == warning => {
                source.end = new_source.end;
                return Ok(());
            }
            _ => {}
        }

        self.warnings.push(Span {
            value: warning,
            source: new_source,
        });

        Ok(())
    }

    /// Yields a warning if the delimiter is not needed due to a line break.
    ///
    /// Expects `self.iter.peek()?` to be [Some].
    ///
    /// Advances one token and then skips an optional line break. If a line break was encountered,
    /// it adds the warning with the source of [LookaheadBuffer::peek]
    /// before [LookaheadBuffer::skip_lb] was called.
    #[inline]
    fn opt_emit_unnecessary_delimiter_warning(&mut self, warning: Warning) -> Result<(), Error> {
        // We need to capture this source outside,
        // or else the borrow checker will get mad.
        let source = self.iter.peek()?.unwrap().source.clone();

        self.iter.advance()?;

        if self.iter.skip_lb()? {
            self.warnings.push(Span {
                value: warning,
                source,
            })
        }

        Ok(())
    }

    /// If `peek()` is a '<', then it parses type parameters. Ends on the token after '>'.
    #[inline]
    fn parse_opt_const_parameters(&mut self) -> Result<ast::ConstParameters<'source, A>, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::LeftAngle),
                ..
            }) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;
                self.parse_const_parameters()?
            }
            _ => Vec::new_in(self.alloc.clone()),
        })
    }

    /// Expects the next token to be the non-lb token after '<'. Ends on the token after `>`.
    fn parse_const_parameters(&mut self) -> Result<ast::ConstParameters<'source, A>, Error> {
        let mut params = Vec::new_in(self.alloc.clone());

        self.meta_parse_list(
            Token::Symbol(Symbol::RightAngle),
            Token::Symbol(Symbol::Comma),
            Warning::UnnecessaryComma,
            true,
            |s| {
                let id = s.parse_identifier()?;

                let mut trait_bounds = Vec::new_in(s.alloc.clone());

                match s.iter.peek_n_non_lb(0)?.0 {
                    Some(Span {
                        value: Token::Keyword(Keyword::Is), // TODO: maybe change to ':'
                        ..
                    }) => loop {
                        s.iter.skip_lb()?;
                        s.iter.advance()?;
                        s.iter.skip_lb()?;

                        let ty = s.parse_type(type_bp::MIN)?;

                        trait_bounds.push(ty);

                        match s.iter.peek_n_non_lb(0)?.0 {
                            Some(Span {
                                value: Token::Symbol(Symbol::Plus),
                                ..
                            }) => {}
                            _ => break,
                        }
                    },
                    Some(Span {
                        value: Token::Symbol(Symbol::Colon),
                        ..
                    }) => {
                        s.iter.skip_lb()?;
                        return error!("You gotta use 'is' instead of ':'.");
                    }
                    _ => {}
                }

                params.push(Span {
                    source: id.source.clone(),
                    value: ast::ConstParameter::Type { id, trait_bounds },
                });

                Ok(())
            },
        )?;

        self.iter.advance()?;

        Ok(params)
    }

    /// Parses a block.
    ///
    /// Expects `peek()` to be [Symbol::LeftBrace]. Ends on [Symbol::RightBrace].
    fn parse_block(
        &mut self,
    ) -> Result<Span<Vec<ast::StatementOrExpression<'source, A>, A>>, Error> {
        let start = self.iter.peek()?.unwrap().source.start;

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let mut items = Vec::new_in(self.alloc.clone());

        let end = self.meta_parse_list(
            Token::Symbol(Symbol::RightBrace),
            Token::Symbol(Symbol::Semicolon),
            Warning::UnnecessarySemicolon,
            true,
            |s| {
                if let Some(statement) = s.try_parse_statement()? {
                    items.push(ast::StatementOrExpression::Statement(statement));
                } else {
                    items.push(ast::StatementOrExpression::Expression(
                        s.parse_expression(0, false)?,
                    ));
                }

                Ok(())
            },
        )?;

        Ok(Span {
            value: items,
            source: start..end,
        })
    }

    /// Parses an identifier.
    ///
    /// Expects `peek()` to yield [Token::Identifier] and advances one token.
    fn parse_identifier(&mut self) -> Result<Span<&'source str>, Error> {
        match self.iter.peek()? {
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let id = Span {
                    value: *id,
                    source: source.clone(),
                };

                self.iter.advance()?;

                Ok(id)
            }
            _ => error!("Expected an identifier"),
        }
    }

    /// Expects the current token to be the first token of the type.
    ///
    /// Ends on past-the-end token.
    fn parse_type(&mut self, min_bp: u8) -> Result<Span<ast::Type<'source, A>>, Error> {
        match self.try_parse_type(min_bp)? {
            Some(ty) => Ok(ty),
            None => {
                error!("Expected a type starting with '!', '(', 'This', '_', or an identifier.")
            }
        }
    }

    /// Expects the current token to be the first token of the type.
    ///
    /// Ends on past-the-end token.
    fn try_parse_type(&mut self, min_bp: u8) -> Result<Option<Span<ast::Type<'source, A>>>, Error> {
        let first_term = self.try_parse_type_first_term()?;
        Ok(match first_term {
            None => None,
            Some(first_term) => Some(self.parse_type_remaining_terms(first_term, min_bp)?),
        })
    }

    fn try_parse_type_first_term(&mut self) -> Result<Option<Span<ast::Type<'source, A>>>, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Underscore),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Type::Inferred,
                    source,
                })
            }
            Some(Span {
                value: Token::Symbol(Symbol::ExclamationMark),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Type::Never,
                    source,
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::CapitalThis),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Type::This,
                    source,
                })
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                if match self.iter.peek()? {
                    Some(Span {
                        value: Token::Keyword(Keyword::Pub | Keyword::Mut),
                        ..
                    }) => true,
                    Some(Span {
                        value: Token::Identifier(_),
                        ..
                    }) => matches!(
                        self.iter.peek_n_non_lb(1)?.0,
                        Some(Span {
                            value: Token::Identifier(_) | Token::Symbol(Symbol::ExclamationMark),
                            ..
                        })
                    ),
                    _ => false,
                } {
                    let mut fields = Vec::new_in(self.alloc.clone());

                    let end = loop {
                        let visibility = match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                source,
                            }) => {
                                let end = source.end;

                                self.iter.advance()?;

                                break end;
                            }
                            _ => self.parse_visibility()?,
                        };

                        self.iter.skip_lb()?;

                        let is_mutable = match self.iter.peek()? {
                            Some(Span {
                                value: Token::Keyword(Keyword::Mut),
                                source,
                            }) => {
                                let source = source.clone();

                                self.iter.advance()?;
                                self.iter.skip_lb()?;

                                Some(Span { value: (), source })
                            }
                            _ => None,
                        };

                        let id = self.parse_identifier()?;

                        self.iter.skip_lb()?;

                        let ty = self.parse_type(type_bp::MIN)?;

                        fields.push(ast::ObjectTypeField {
                            visibility,
                            is_mutable,
                            id,
                            ty,
                        });

                        match self.iter.peek_n_non_lb(0)? {
                            (_, true) => {
                                // If the next token is a line break, we skip it.
                                self.iter.advance()?;
                            }
                            (
                                Some(Span {
                                    value: Token::Symbol(Symbol::Comma),
                                    ..
                                }),
                                _,
                            ) => {
                                self.opt_emit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            (
                                Some(Span {
                                    value: Token::Symbol(Symbol::RightParenthesis),
                                    source,
                                }),
                                _,
                            ) => {
                                let end = source.end;
                                self.iter.advance()?;
                                break end;
                            }
                            token => {
                                return error!(
                                    "Expected ',', ')' or a line break, found: {:?}",
                                    token
                                )
                            }
                        }
                    };

                    Some(Span {
                        source: start..end,
                        value: ast::Type::Object(fields),
                    })
                } else if let Some(Span {
                    value: Token::Symbol(Symbol::RightParenthesis),
                    source,
                }) = self.iter.peek()?
                {
                    let end = source.end;

                    self.iter.advance()?;

                    Some(Span {
                        value: ast::Type::Object(Vec::new_in(self.alloc.clone())),
                        source: start..end,
                    })
                } else {
                    // Grouping

                    let ty = self.parse_type(type_bp::MIN)?;

                    self.iter.skip_lb()?;

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            ..
                        }) => {}
                        _ => return error!("Expected ')'"),
                    }

                    self.iter.advance()?;

                    Some(ty)
                }
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let source = source.clone();

                let id = Span {
                    value: *id,
                    source: source.clone(),
                };

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let path = self.parse_item_path(id)?;

                let const_parameters = if let Some(Span {
                    value: Token::Symbol(Symbol::LeftAngle),
                    source,
                }) = self.iter.peek_n_non_lb(0)?.0
                {
                    let start = source.start;

                    self.iter.skip_lb()?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let mut params = Vec::new_in(self.alloc.clone());

                    let end = self.meta_parse_list(
                        Token::Symbol(Symbol::RightAngle),
                        Token::Symbol(Symbol::Comma),
                        Warning::UnnecessaryComma,
                        true, // TODO: allow empty items?
                        |s| {
                            params.push(s.parse_type(type_bp::MIN)?);

                            Ok(())
                        },
                    )?;

                    self.iter.advance()?;

                    Span {
                        value: params,
                        source: start..end,
                    }
                } else {
                    Span {
                        value: Vec::new_in(self.alloc.clone()),
                        source: source.clone(),
                    }
                };

                Some(Span {
                    source: source.start..const_parameters.source.end,
                    value: ast::Type::Item(ast::ItemRef {
                        path,
                        const_parameters,
                    }),
                })
            }
            _ => None,
        })
    }

    fn parse_type_remaining_terms(
        &mut self,
        mut left_term: Span<ast::Type<'source, A>>,
        min_bp: u8,
    ) -> Result<Span<ast::Type<'source, A>>, Error> {
        loop {
            match self.iter.peek_n_non_lb(0)?.0 {
                Some(Span {
                    value: Token::Symbol(Symbol::MinusRightAngle), // ->
                    ..
                }) => {
                    if type_bp::FUNCTION.0 < min_bp {
                        return Ok(left_term);
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance()?;

                    let right_term = self.parse_type(type_bp::FUNCTION.1)?;

                    left_term = Span {
                        source: left_term.source.start..right_term.source.end,
                        value: ast::Type::Function {
                            input: Box::new_in(left_term, self.alloc.clone()),
                            output: Box::new_in(right_term, self.alloc.clone()),
                        },
                    };
                }
                Some(Span {
                    value: Token::Symbol(Symbol::Pipe), // |
                    ..
                }) => {
                    if type_bp::UNION.0 < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance()?;

                    let right_term = self.parse_type(type_bp::UNION.1)?;

                    left_term = Span {
                        source: left_term.source.start..right_term.source.end,
                        value: ast::Type::Union {
                            left: Box::new_in(left_term, self.alloc.clone()),
                            right: Box::new_in(right_term, self.alloc.clone()),
                        },
                    };
                }
                _ => break,
            }
        }

        Ok(left_term)
    }

    /// Expects the next token to be '.' or lb before the '.'. Ends on the past-the-end token.
    fn parse_use_child(&mut self) -> Result<Span<ast::UseChild<'source, A>>, Error> {
        self.iter.skip_lb()?;
        self.iter.advance()?;
        self.iter.skip_lb()?;

        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::Star),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Span {
                    value: ast::UseChild::All,
                    source,
                }
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source,
            }) => {
                let start = source.start;
                let mut vec = Vec::new_in(self.alloc.clone());

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightParenthesis),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        let id = s.parse_identifier()?;
                        s.iter.skip_lb()?;

                        vec.push(s.parse_use(id)?);

                        Ok(())
                    },
                )?;

                Span {
                    value: ast::UseChild::Multiple(vec),
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let id = Span {
                    value: *id,
                    source: source.clone(),
                };

                self.iter.advance()?;

                let u = self.parse_use(id)?;

                Span {
                    source: u.source(),
                    value: ast::UseChild::Single(Box::new_in(u, self.alloc.clone())),
                }
            }
            _ => return error!("Expected an identifier, '*' or '('"),
        })
    }

    /// Expects the next token to be after the id. Ends on the token after the use-statement.
    fn parse_use(&mut self, id: Span<&'source str>) -> Result<ast::Use<'source, A>, Error> {
        Ok(match self.iter.peek_n_non_lb(0)?.0 {
            Some(Span {
                value: Token::Symbol(Symbol::Dot),
                ..
            }) => {
                let child = self.parse_use_child()?;

                ast::Use {
                    id,
                    child: Some(child),
                }
            }
            _ => ast::Use { id, child: None },
        })
    }

    /// Expects [Buffered::peek] to be after the first identifier.
    /// Ends on the token after the last path segment.
    fn parse_item_path(
        &mut self,
        mut first_id: Span<&'source str>,
    ) -> Result<ast::ItemPath<'source, A>, Error> {
        let parents = if let Some(Span {
            value: Token::Symbol(Symbol::Dot),
            ..
        }) = self.iter.peek_n_non_lb(0)?.0
        {
            let mut parents = Vec::new_in(self.alloc.clone());

            loop {
                parents.push(first_id);

                self.iter.advance()?;
                self.iter.skip_lb()?;

                first_id = match self.iter.peek()? {
                    Some(Span {
                        value: Token::Identifier(id),
                        source: new_source,
                    }) => Span {
                        value: *id,
                        source: new_source.clone(),
                    },
                    _ => return error!("Expected an identifier"),
                };

                self.iter.advance()?;

                match self.iter.peek_n_non_lb(0)?.0 {
                    Some(Span {
                        value: Token::Symbol(Symbol::Dot),
                        ..
                    }) => {}
                    _ => break,
                }
            }

            parents
        } else {
            Vec::new_in(self.alloc.clone())
        };

        Ok(ast::ItemPath {
            parents,
            id: first_id,
        })
    }

    /// Parses a comma-separated list of something. Every iteration `f` is called. Returns when a
    /// `bail_token` is encountered and does not advance after that.
    #[inline(always)]
    fn meta_parse_list<F: FnMut(&mut Self) -> Result<(), Error>>(
        &mut self,
        bail_token: Token<'source>,
        delimiter_token: Token<'source>,
        delimiter_warning: Warning,
        allow_empty_items: bool,
        mut f: F,
    ) -> Result<Index, Error> {
        Ok(loop {
            match self.iter.peek()? {
                Some(Span { value, source }) if value == &bail_token => break source.end,
                Some(Span { value, .. }) if value == &delimiter_token && allow_empty_items => {
                    self.emit_single_token_warning(delimiter_warning)?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                    continue;
                }
                _ => {}
            }

            f(self)?;

            match self.iter.peek_n_non_lb(0)? {
                (Some(Span { value, .. }), true) if value == &delimiter_token => {
                    // Ominous syntax

                    // Skip line break.
                    self.iter.advance()?;

                    self.emit_single_token_warning(delimiter_warning)?;
                    self.iter.advance()?;
                }
                (_, true) => {
                    self.iter.skip_lb()?;
                }
                (Some(Span { value, .. }), _) if value == &delimiter_token => {
                    self.opt_emit_unnecessary_delimiter_warning(delimiter_warning)?;
                }
                (Some(Span { value, source }), _) if value == &bail_token => break source.end,
                _ => return error!("Expected ',', '{}' or a line break", &bail_token),
            }
        })
    }

    /// Peek: first token. Ends on past-the-end.
    fn parse_pattern(&mut self) -> Result<ast::Pattern<'source, A>, Error> {
        let unit = match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Mut),
                source: mut_source,
            }) => {
                let mut_source = mut_source.clone();

                self.iter.advance()?;
                self.iter.skip_lb()?;

                // TODO: maybe custom error message
                let id = self.parse_identifier()?;

                ast::PatternUnit::Identifier {
                    is_mutable: Some(Span {
                        value: (),
                        source: mut_source,
                    }),
                    id,
                }
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let id = Span {
                    value: *id,
                    source: source.clone(),
                };

                self.iter.advance()?;

                ast::PatternUnit::Identifier {
                    is_mutable: None,
                    id,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Underscore),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                ast::PatternUnit::Any(Span { value: (), source })
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftBracket),
                source,
            }) => {
                let start = source.start;
                let mut items = Vec::new_in(self.alloc.clone());

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightBracket),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        items.push(s.parse_pattern()?);
                        Ok(())
                    },
                )?;

                self.iter.advance()?;

                ast::PatternUnit::Array(Span {
                    value: items,
                    source: start..end,
                })
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source,
            }) => {
                let start = source.start;
                let mut fields = Vec::new_in(self.alloc.clone());

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightParenthesis),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        // TODO: mut, ty

                        let id = s.parse_identifier()?;

                        let remap = match s.iter.peek_n_non_lb(0)?.0 {
                            Some(Span {
                                value: Token::Symbol(Symbol::Equals),
                                ..
                            }) => {
                                s.iter.skip_lb()?;
                                s.iter.advance()?;
                                s.iter.skip_lb()?;

                                s.parse_pattern()?
                            }
                            _ => ast::Pattern::WithType(ast::PatternUnitWithType {
                                ty: Span {
                                    value: ast::Type::Inferred,
                                    source: id.source.end..id.source.end,
                                },
                                unit: ast::PatternUnit::Identifier {
                                    is_mutable: None,
                                    id: id.clone(),
                                },
                            }),
                        };

                        fields.push(ast::ObjectPatternField { id, remap });

                        Ok(())
                    },
                )?;

                self.iter.advance()?;

                ast::PatternUnit::Object(Span {
                    value: fields,
                    source: start..end,
                })
            }
            // TODO: impl []
            _ => return error!("Expected an identifier, 'mut', '_', '(', or '['."),
        };

        self.iter.skip_lb()?;

        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::At),
                ..
            }) => {
                let (id, is_mutable) = match unit {
                    ast::PatternUnit::Identifier { id, is_mutable } => (id, is_mutable),
                    // TODO
                    _ => return error!("Cannot attach a pattern to a non-identifier"),
                };

                self.iter.advance()?;
                self.iter.skip_lb()?;

                ast::Pattern::Attach {
                    is_mutable,
                    id,
                    pattern: Box::new_in(self.parse_pattern()?, self.alloc.clone()),
                }
            }
            _ => {
                let end = unit.source().end;

                // Default to '_'.
                let ty = self.try_parse_type(type_bp::MIN)?.unwrap_or(Span {
                    value: ast::Type::Inferred,
                    source: end..end,
                });

                ast::Pattern::WithType(ast::PatternUnitWithType { unit, ty })
            }
        })
    }

    /// Expects `peek()` the token after '('. Ends on ')'. Returns `(fields, end)`.
    fn parse_object_literal(
        &mut self,
    ) -> Result<(Vec<ast::ObjectField<'source, A>, A>, Index), Error> {
        let mut fields = Vec::new_in(self.alloc.clone());

        let end = self.meta_parse_list(
            Token::Symbol(Symbol::RightParenthesis),
            Token::Symbol(Symbol::Comma),
            Warning::UnnecessaryComma,
            true,
            |s| {
                // TODO: maybe custom error message
                let id = s.parse_identifier()?;

                s.iter.advance()?;
                s.iter.skip_lb()?;

                match s.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::Equals),
                        ..
                    }) => {}
                    _ => return error!("Expected '='."),
                }

                s.iter.advance()?;
                s.iter.skip_lb()?;

                let value = s.parse_expression(0, false)?;

                fields.push(ast::ObjectField { id, value });

                Ok(())
            },
        )?;

        self.iter.advance()?;

        Ok((fields, end))
    }

    /// Function syntax ("[]" indicate optional):
    ///
    /// ```plain
    /// fn[<T, ...>] id InputType [-> OutputType] = binding => body
    /// fn[<T, ...>] id InputType [-> OutputType] { body } // when ignoring `InputType`
    /// ```
    fn parse_fn_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let const_parameters = self.parse_opt_const_parameters()?;
        self.iter.skip_lb()?;

        let id = self.parse_identifier()?;
        self.iter.skip_lb()?;

        match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::Dot),
                ..
            }) => todo!("associated fns"),
            Some(Span {
                value: Token::Symbol(Symbol::LeftAngle),
                ..
            }) => {
                return error!(
                    "Type parameter are declared after 'fn', not after the function name."
                )
            }
            _ => {}
        }

        let pattern = self.parse_pattern()?;
        self.iter.skip_lb()?;

        match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::LeftBrace),
                ..
            }) => {}
            token => return error!("Expected '{{', found: {:?}", token),
        }

        let body = self.parse_block()?;
        self.iter.advance()?;

        let end = body.source.end;

        let (pattern, output_type) = pattern.lift_function_return_type();

        Ok((
            ast::StatementKind::Function {
                const_parameters,
                output_type,
                id,
                pattern,
                body,
            },
            end,
        ))
    }

    /// Parses the visibility of a type or module, may be `None`.
    ///
    /// Expects `peek()` to be the start of the visibility declaration.
    /// Ends on the token after the visibility declaration.
    fn parse_visibility(&mut self) -> Result<ast::Visibility, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Pub),
                source,
            }) => {
                let mut source = source.clone();

                self.iter.advance()?;

                let v = match self.iter.peek_n_non_lb(0)?.0 {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftParenthesis),
                        ..
                    }) => {
                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let v = match self.iter.peek()? {
                            Some(Span {
                                value: Token::Keyword(Keyword::Package),
                                ..
                            }) => ast::Visible::Package,
                            Some(Span {
                                value: Token::Keyword(Keyword::Mod),
                                ..
                            }) => ast::Visible::Module,
                            _ => return error!("Expected 'package' or 'mod'."),
                        };

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                source: paren_source,
                            }) => {
                                source.end = paren_source.end;
                                self.iter.advance()?;
                                v
                            }
                            _ => return error!("Expected ')'"),
                        }
                    }
                    _ => ast::Visible::Public,
                };

                Some(Span { value: v, source })
            }
            _ => None,
        })
    }

    /// Expects the current token to be `type`. Ends on the token after the type statement.
    fn parse_type_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let const_parameters = self.parse_opt_const_parameters()?;
        self.iter.skip_lb()?;

        let id = self.parse_identifier()?;
        self.iter.skip_lb()?;

        let ty_visibility = self.parse_visibility()?;
        self.iter.skip_lb()?;

        let ty_is_mutable = match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Mut),
                ..
            }) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                true
            }
            _ => false,
        };

        let ty = match self.try_parse_type(type_bp::MIN)? {
            Some(ty) => ty,
            None if let None = ty_visibility
                && !ty_is_mutable =>
            {
                Span {
                    value: ast::Type::Object(Vec::new_in(self.alloc.clone())),
                    source: id.source.end..id.source.end,
                }
            }
            None => return error!("Expected a type."),
        };

        let end = ty.source.end;

        Ok((
            ast::StatementKind::Type {
                id,
                const_parameters,
                ty,
                ty_is_mutable,
                ty_visibility,
            },
            end,
        ))
    }

    fn parse_enum_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        // Skips `enum`.
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let const_parameters = self.parse_opt_const_parameters()?;
        self.iter.skip_lb()?;

        let id = self.parse_identifier()?;

        let mut variants = Vec::new_in(self.alloc.clone());

        let end = match self.iter.peek_n_non_lb(0)?.0 {
            Some(Span {
                value: Token::Symbol(Symbol::LeftBrace),
                ..
            }) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightBrace),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        // TODO: annotations

                        let id = s.parse_identifier()?;

                        let expr = match s.iter.peek_n_non_lb(0)?.0 {
                            Some(Span {
                                value: Token::Symbol(Symbol::Equals),
                                ..
                            }) => {
                                s.iter.skip_lb()?;
                                s.iter.advance()?;
                                s.iter.skip_lb()?;

                                Some(s.parse_expression(0, false)?)
                            }
                            _ => None,
                        };

                        variants.push((id, expr));

                        Ok(())
                    },
                )?;

                self.iter.advance()?;

                end
            }
            _ => id.source.end,
        };

        Ok((
            ast::StatementKind::Enum {
                const_parameters,
                id,
                variants,
            },
            end,
        ))
    }

    fn parse_mod_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let id = self.parse_identifier()?;
        let mut end = id.source.end;

        let content: Option<_> = match self.iter.peek_n_non_lb(0)? {
            // Code: mod xyz { ... }
            (
                Some(Span {
                    value: Token::Symbol(Symbol::LeftBrace),
                    ..
                }),
                _,
            ) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let content = self.parse_module_content()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::RightBrace),
                        source,
                    }) => {
                        end = source.end;
                    }
                    _ => return error!("Expected '}'"),
                }

                Some(content)
            }
            (
                Some(Span {
                    value: Token::LineBreak | Token::Symbol(Symbol::RightBrace),
                    ..
                }),
                _,
            ) => {
                self.iter.skip_lb()?;
                None
            }
            (
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }),
                _,
            ) => None, // The caller handles this
            (_, true) => None,
            _ => return error!("Expected '{', ';', or a line break."),
        };

        Ok((ast::StatementKind::Module { id, content }, end))
    }

    fn parse_let_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let pattern = self.parse_pattern()?;

        let value = match self.iter.peek_n_non_lb(0)? {
            (
                Some(Span {
                    value: Token::Symbol(Symbol::Equals),
                    ..
                }),
                _,
            ) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let expr = self.parse_expression(0, false)?;
                Some(expr)
            }
            _ => None,
        };

        let end = value
            .as_ref()
            .map(|value| value.source.end)
            .unwrap_or_else(|| pattern.source().end);

        Ok((ast::StatementKind::Let { pattern, value }, end))
    }

    fn parse_for_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let pattern = self.parse_pattern()?;
        self.iter.skip_lb()?;

        match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::In),
                ..
            }) => {}
            _ => return error!("Expected 'in'."),
        }

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let iter = self.parse_expression(0, true)?;
        self.iter.skip_lb()?;

        match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::EqualsRightAngle),
                ..
            }) => {}
            _ => return error!("Expected '=>'."),
        }

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let body = self.parse_expression(0, false)?;
        let end = body.source.end;

        Ok((
            ast::StatementKind::For {
                pattern,
                iter,
                body,
            },
            end,
        ))
    }

    /// Tries to parse a statement. If nothing matches, `None` will be returned.
    ///
    /// # Tokens
    ///
    /// - Expects `peek()` to correspond to the first non-lb token of the statement (pre-advance).
    /// - Ends on the token after the statement. The caller must validate that token.
    /// **This may be a [Token::LineBreak]!**
    fn try_parse_statement(&mut self) -> Result<Option<ast::Statement<'source, A>>, Error> {
        match self.iter.peek()? {
            None => return Ok(None),
            _ => {}
        };

        let mut annotations = Vec::new_in(self.alloc.clone());

        // Collect all available annotations associated with the statement.
        loop {
            let start = match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::At),
                    source,
                }) => source.start,
                _ => break,
            };

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let id = self.parse_identifier()?;
            self.iter.skip_lb()?;

            let path = self.parse_item_path(id)?;
            self.iter.skip_lb()?;

            annotations.push(Span {
                source: start..path.source().end,
                value: ast::Annotation {
                    path,
                    arguments: Vec::new_in(self.alloc.clone()),
                },
            });
        }

        self.iter.skip_lb()?;

        let statement_kind = match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Fn),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_fn_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Mod),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_mod_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Type),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_type_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Enum),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_enum_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Let),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_let_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::For),
                source,
            }) => {
                let start = source.start;
                let (kind, end) = self.parse_for_statement_kind()?;

                Span {
                    value: kind,
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Use),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let root_id = self.parse_identifier()?;
                self.iter.skip_lb()?;

                let u = self.parse_use(root_id)?;
                let end = u.source().end;

                Span {
                    value: ast::StatementKind::Use(u),
                    source: start..end,
                }
            }
            _ => {
                if annotations.len() > 0 {
                    return error!("Annotations are not attached to a statement");
                }

                return Ok(None);
            }
        };

        Ok(Some(ast::Statement {
            annotations,
            statement_kind,
        }))
    }

    /// Parses an expression.
    ///
    /// Expects `peek()` to be the first token of the expression. May end on anything.
    pub fn parse_expression(
        &mut self,
        min_bp: u8,
        calls_across_line_breaks: bool,
    ) -> Result<Span<ast::Expression<'source, A>>, Error> {
        match self.try_parse_expression(min_bp, calls_across_line_breaks)? {
            Some(expr) => Ok(expr),
            None => error!("Invalid start of an expression"),
        }
    }

    /// Current token is first token of the expression. If the first token does not start an expression,
    /// `None` will be returned.
    pub fn try_parse_expression(
        &mut self,
        min_bp: u8,
        calls_across_line_breaks: bool,
    ) -> Result<Option<Span<ast::Expression<'source, A>>>, Error> {
        match self.try_parse_expression_first_term()? {
            Some(first_term) => {
                let expr = self.parse_expression_remaining_terms(
                    first_term,
                    min_bp,
                    calls_across_line_breaks,
                )?;
                Ok(Some(expr))
            }
            None => Ok(None),
        }
    }

    fn parse_markup_element(
        &mut self,
        tag_name: Span<&'source str>,
    ) -> Result<(ast::Expression<'source, A>, Index), Error> {
        let params_start = tag_name.source.end;
        let mut params = Vec::new_in(self.alloc.clone());
        
        let (params_end, end) = loop {
            let id = match self.iter.peek()? {
                Some(Span {
                    value: Token::MarkupClose,
                    source,
                }) => break (source.start, source.end),
                Some(Span {
                    value: Token::MarkupKey(key),
                    source,
                }) => Span {
                    value: *key,
                    source: source.clone(),
                },
                Some(Span {
                    value: Token::MarkupStartTagEnd,
                    source
                }) => {
                    // Parse children

                    let children_start = source.end;

                    self.iter.advance()?;

                    let mut children = Vec::new_in(self.alloc.clone());
                    let mut children_end = children_start;

                    let (end_tag_name, end) = loop {
                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::MarkupStartTag(tag_name),
                                source
                            }) => {
                                let tag_name = tag_name.clone();
                                let element_start = source.start;

                                self.iter.advance()?;
                                
                                let (expr, element_end) = self.parse_markup_element(tag_name)?;
                                children_end = element_end;

                                children.push(Span {
                                    value: expr,
                                    source: element_start..element_end,
                                });
                            }
                            Some(Span {
                                value: Token::MarkupEndTag(end_tag_name),
                                source
                            }) => break (end_tag_name.clone(), source.end),
                            Some(Span {
                                value: Token::MarkupText(text),
                                source,
                            }) => {
                                let text = *text;
                                let source = source.clone();

                                self.iter.advance()?;

                                children_end = source.end;

                                children.push(Span {
                                    source,
                                    value: ast::Expression::String(EcoString::from(text)),
                                })
                            }
                            Some(Span { value: Token::Symbol(Symbol::LeftBrace), .. }) => {
                                todo!()
                            }
                            _ => unreachable!(),
                        }
                    };

                    if end_tag_name.value != tag_name.value {
                        return error!("Start and end tag do not equal.");
                    }

                    if children.len() > 0 {
                        params.push(ast::ObjectField {
                            id: Span {
                                value: "children",
                                source: children_start..children_end,
                            },
                            value: Span {
                                value: ast::Expression::Array(children),
                                source: children_start..children_end,
                            },
                        });
                    }

                    break (children_end, end);
                }
                token => unreachable!("{:?}", token),
            };

            self.iter.advance()?;

            let value = match self.iter.peek()? {
                Some(Span {
                    value: Token::String(str),
                    source,
                }) => Span {
                    value: ast::Expression::String(str.clone()),
                    source: source.clone(),
                },
                Some(Span {
                    value: Token::Symbol(Symbol::LeftBrace),
                    ..
                }) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                    self.parse_expression(0, true)?
                }
                _ => unreachable!(),
            };

            params.push(ast::ObjectField { id, value });

            self.iter.advance()?;
        };

        self.iter.advance()?;

        Ok((
            ast::Expression::Call {
                callee: Box::new_in(
                    tag_name.map(ast::Expression::Identifier),
                    self.alloc.clone(),
                ),
                argument: Box::new_in(
                    Span {
                        value: ast::Expression::Object(params),
                        source: params_start..params_end,
                    },
                    self.alloc.clone(),
                ),
            },
            end,
        ))
    }

    /// Expects a token.
    fn try_parse_expression_first_term(
        &mut self,
    ) -> Result<Option<Span<ast::Expression<'source, A>>>, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Match),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let on_expr = self.parse_expression(0, true)?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::EqualsRightAngle),
                        ..
                    }) => {}
                    _ => return error!("Expected '=>'."),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftBrace),
                        ..
                    }) => {}
                    _ => return error!("Expected '{'."),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let mut cases = Vec::new_in(self.alloc.clone());

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightBrace),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        let pattern = s.parse_pattern()?;
                        s.iter.skip_lb()?;

                        match s.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::EqualsRightAngle),
                                ..
                            }) => {}
                            _ => return error!("Expected '=>'."),
                        }

                        s.iter.advance()?;
                        s.iter.skip_lb()?;

                        let expr = s.parse_expression(0, false)?;

                        cases.push(ast::MatchCase {
                            pattern,
                            expression: expr,
                        });

                        Ok(())
                    },
                )?;

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::Match {
                        on: Box::new_in(on_expr, self.alloc.clone()),
                        cases,
                    },
                    source: start..end,
                })
            }
            Some(Span {
                value: Token::String(s),
                source,
            }) => {
                let source = source.clone();
                let s = s.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::String(s),
                    source,
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::CapitalThis),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::CapitalThis,
                    source,
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::This),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::This,
                    source,
                })
            }
            Some(Span {
                value: Token::Number(n),
                source,
            }) => {
                let source = source.clone();
                let n = *n;

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::Number(n),
                    source,
                })
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let id = *id;
                let source = source.clone();

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::Identifier(id),
                    source,
                })
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;
                self.iter.skip_lb()?;

                if let (
                    Some(Span {
                        value: Token::Identifier(_),
                        ..
                    }),
                    _,
                ) = self.iter.peek_n_non_lb(0)?
                    && let (
                        Some(Span {
                            value: Token::Symbol(Symbol::Equals),
                            ..
                        }),
                        _,
                    ) = self.iter.peek_n_non_lb(1)?
                {
                    let (fields, end) = self.parse_object_literal()?;

                    self.iter.advance()?;

                    Some(Span {
                        source: source.start..end,
                        value: ast::Expression::Object(fields),
                    })
                } else {
                    // Regular grouped expression

                    let expr = self.parse_expression(0, true)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            ..
                        }) => {}
                        token => return error!("Expected ')', found: {:?}", token),
                    }

                    self.iter.advance()?;

                    Some(expr)
                }
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftBracket),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let mut items = Vec::new_in(self.alloc.clone());

                let end = self.meta_parse_list(
                    Token::Symbol(Symbol::RightBracket),
                    Token::Symbol(Symbol::Comma),
                    Warning::UnnecessaryComma,
                    true,
                    |s| {
                        items.push(s.parse_expression(0, false)?);
                        Ok(())
                    },
                )?;

                self.iter.advance()?;

                Some(Span {
                    value: ast::Expression::Array(items),
                    source: start..end,
                })
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftBrace),
                source,
            }) => {
                let start = source.start;
                let block = self.parse_block()?;

                self.iter.advance()?;

                Some(Span {
                    source: start..block.source.end,
                    value: ast::Expression::Block(block.value),
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::Fn),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let pattern = self.parse_pattern()?;
                self.iter.skip_lb()?;

                let (pattern, return_type) = pattern.lift_function_return_type();

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::EqualsRightAngle),
                        ..
                    }) => {}
                    _ => return error!("Expected '=>'."),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let body = self.parse_expression(0, false)?;

                Some(Span {
                    source: start..body.source.end,
                    value: ast::Expression::Function {
                        pattern,
                        return_type,
                        body: Box::new_in(body, self.alloc.clone()),
                    },
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::If),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let condition = self.parse_expression(0, true)?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::EqualsRightAngle),
                        ..
                    }) => {}
                    _ => return error!("Expected '=>'"),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let then_expr = self.parse_expression(0, false)?;

                let else_expr = match self.iter.peek_n_non_lb(0)?.0 {
                    Some(Span {
                        value: Token::Keyword(Keyword::Else),
                        ..
                    }) => {
                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        Some(self.parse_expression(0, false)?)
                    }
                    _ => None,
                };

                let end = else_expr
                    .as_ref()
                    .map(|else_body| else_body.source.end)
                    .unwrap_or_else(|| then_expr.source.end);

                Some(Span {
                    source: start..end,
                    value: ast::Expression::If {
                        condition: Box::new_in(condition, self.alloc.clone()),
                        then_expr: Box::new_in(then_expr, self.alloc.clone()),
                        else_expr: else_expr.map(|e| Box::new_in(e, self.alloc.clone())),
                    },
                })
            }
            Some(Span {
                value: Token::Keyword(Keyword::While),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let condition = self.parse_expression(0, true)?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::Equals),
                        ..
                    }) => {}
                    _ => return error!("Expected '='."),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let body = self.parse_expression(0, false)?;

                Some(Span {
                    source: start..body.source.end,
                    value: ast::Expression::While {
                        condition: Box::new_in(condition, self.alloc.clone()),
                        body: Box::new_in(body, self.alloc.clone()),
                    },
                })
            }
            Some(Span {
                value: Token::MarkupStartTag(tag_name),
                source,
            }) => {
                let tag_name = tag_name.clone();
                let start = source.start;
                
                self.iter.advance()?;
                
                let (expr, end) = self.parse_markup_element(tag_name)?;

                Some(Span {
                    value: expr,
                    source: start..end,
                })
            }
            _ => None,
        })
    }

    fn parse_expression_remaining_terms(
        &mut self,
        mut first_term: Span<ast::Expression<'source, A>>,
        min_bp: u8,
        calls_across_line_breaks: bool,
    ) -> Result<Span<ast::Expression<'source, A>>, Error> {
        let start = first_term.source.start;

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let right = self.parse_expression($bp.1, calls_across_line_breaks)?;

                (
                    right.source.end,
                    ast::Expression::Operation {
                        left: Box::new_in(first_term, self.alloc.clone()),
                        operation: $op,
                        right: Box::new_in(right, self.alloc.clone()),
                    },
                )
            }};
        }

        macro_rules! as_op {
            ($op: expr) => {{
                if bp::ASSIGNMENT.0 < min_bp {
                    break;
                }

                let at = match first_term {
                    Span {
                        value: ast::Expression::Access(access),
                        source,
                    } => Span {
                        value: ast::AssignmentTarget::Access(access),
                        source,
                    },
                    Span {
                        value: ast::Expression::Identifier(id),
                        source,
                    } => Span {
                        value: ast::AssignmentTarget::Identifier(id),
                        source,
                    },
                    _ => return error!("Cannot assign to an expression"),
                };

                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let right = self.parse_expression(bp::ASSIGNMENT.1, calls_across_line_breaks)?;

                (
                    right.source.end,
                    ast::Expression::Assignment {
                        target: Box::new_in(at, self.alloc.clone()),
                        operation: $op,
                        value: Box::new_in(right, self.alloc.clone()),
                    },
                )
            }};
        }

        loop {
            let (token, skipped_line_break) = self.iter.peek_n_non_lb(0)?;

            let (end, value) = match token.map(|token| &token.value) {
                // Potential assignment operations
                Some(Token::Symbol(Symbol::Plus)) => {
                    op!(ast::Operation::PA(ast::PAOperation::Addition), bp::ADDITIVE)
                }
                Some(Token::Symbol(Symbol::Minus)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::Subtraction),
                        bp::ADDITIVE
                    )
                }
                Some(Token::Symbol(Symbol::Star)) => op!(
                    ast::Operation::PA(ast::PAOperation::Multiplication),
                    bp::MULTIPLICATIVE
                ),
                Some(Token::Symbol(Symbol::Slash)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::Division),
                        bp::MULTIPLICATIVE
                    )
                }
                Some(Token::Symbol(Symbol::Percent)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::Remainder),
                        bp::MULTIPLICATIVE
                    )
                }
                Some(Token::Symbol(Symbol::StarStar)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::Exponentiation),
                        bp::EXPONENTIAL
                    )
                }
                Some(Token::Symbol(Symbol::Pipe)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::BitwiseOr),
                        bp::BITWISE_OR
                    )
                }
                Some(Token::Symbol(Symbol::Ampersand)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::BitwiseAnd),
                        bp::BITWISE_AND
                    )
                }
                Some(Token::Symbol(Symbol::Caret)) => op!(
                    ast::Operation::PA(ast::PAOperation::BitwiseExclusiveOr),
                    bp::BITWISE_XOR
                ),
                Some(Token::Symbol(Symbol::PipePipe)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::LogicalOr),
                        bp::LOGICAL_OR
                    )
                }
                Some(Token::Symbol(Symbol::AmpersandAmpersand)) => {
                    op!(
                        ast::Operation::PA(ast::PAOperation::LogicalAnd),
                        bp::LOGICAL_AND
                    )
                }
                Some(Token::Symbol(Symbol::LeftAngleLeftAngle)) => {
                    op!(ast::Operation::PA(ast::PAOperation::ShiftLeft), bp::SHIFT)
                }
                Some(Token::Symbol(Symbol::RightAngleRightAngle)) => {
                    op!(ast::Operation::PA(ast::PAOperation::ShiftRight), bp::SHIFT)
                }

                // Assignment operations
                Some(Token::Symbol(Symbol::PlusEquals)) => as_op!(Some(ast::PAOperation::Addition)),
                Some(Token::Symbol(Symbol::MinusEquals)) => {
                    as_op!(Some(ast::PAOperation::Subtraction))
                }
                Some(Token::Symbol(Symbol::StarEquals)) => {
                    as_op!(Some(ast::PAOperation::Multiplication))
                }
                Some(Token::Symbol(Symbol::SlashEquals)) => {
                    as_op!(Some(ast::PAOperation::Division))
                }
                Some(Token::Symbol(Symbol::PercentEquals)) => {
                    as_op!(Some(ast::PAOperation::Remainder))
                }
                Some(Token::Symbol(Symbol::StarStarEquals)) => {
                    as_op!(Some(ast::PAOperation::Exponentiation))
                }
                Some(Token::Symbol(Symbol::PipeEquals)) => {
                    as_op!(Some(ast::PAOperation::BitwiseOr))
                }
                Some(Token::Symbol(Symbol::AmpersandEquals)) => {
                    as_op!(Some(ast::PAOperation::BitwiseAnd))
                }
                Some(Token::Symbol(Symbol::CaretEquals)) => {
                    as_op!(Some(ast::PAOperation::BitwiseExclusiveOr))
                }
                Some(Token::Symbol(Symbol::PipePipeEquals)) => {
                    as_op!(Some(ast::PAOperation::LogicalOr))
                }
                Some(Token::Symbol(Symbol::AmpersandAmpersandEquals)) => {
                    as_op!(Some(ast::PAOperation::LogicalAnd))
                }
                Some(Token::Symbol(Symbol::LeftAngleLeftAngleEquals)) => {
                    as_op!(Some(ast::PAOperation::ShiftLeft))
                }
                Some(Token::Symbol(Symbol::RightAngleRightAngleEquals)) => {
                    as_op!(Some(ast::PAOperation::ShiftRight))
                }

                // Comparative operations
                Some(Token::Symbol(Symbol::EqualsEquals)) => {
                    op!(
                        ast::Operation::Comp(ast::ComparativeOperation::Equals),
                        bp::EQUALITY
                    )
                }
                Some(Token::Symbol(Symbol::LeftAngle)) => op!(
                    ast::Operation::Comp(ast::ComparativeOperation::LessThan),
                    bp::EQUALITY
                ),
                Some(Token::Symbol(Symbol::RightAngle)) => op!(
                    ast::Operation::Comp(ast::ComparativeOperation::GreaterThan),
                    bp::EQUALITY
                ),
                Some(Token::Symbol(Symbol::ExclamationMarkEquals)) => op!(
                    ast::Operation::Comp(ast::ComparativeOperation::NotEquals),
                    bp::EQUALITY
                ),
                Some(Token::Symbol(Symbol::LeftAngleEquals)) => op!(
                    ast::Operation::Comp(ast::ComparativeOperation::LessThanOrEqual),
                    bp::EQUALITY
                ),
                Some(Token::Symbol(Symbol::RightAngleEquals)) => op!(
                    ast::Operation::Comp(ast::ComparativeOperation::GreaterThanOrEqual),
                    bp::EQUALITY
                ),
                Some(Token::Symbol(Symbol::Dot)) => {
                    if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    if let Some(Span {
                        value: Token::Symbol(Symbol::LeftAngle),
                        ..
                    }) = self.iter.peek()?
                    {
                        // Refine

                        // TODO: expressions in const parameters

                        self.iter.advance()?; // Skip '<'
                        self.iter.skip_lb()?;

                        let mut const_arguments = Vec::new_in(self.alloc.clone());

                        self.meta_parse_list(
                            Token::Symbol(Symbol::RightAngle),
                            Token::Symbol(Symbol::Comma),
                            Warning::UnnecessaryComma,
                            true,
                            |s| {
                                const_arguments.push(
                                    s.parse_type(type_bp::MIN)?.map(ast::ConstArgument::Type),
                                );
                                Ok(())
                            },
                        )?;

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let end = match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::LeftParenthesis),
                                source,
                            }) => source.end,
                            _ => return error!("Expected '(' (call arguments)"),
                        };

                        (
                            end,
                            ast::Expression::Refine {
                                target: Box::new_in(first_term, self.alloc.clone()),
                                const_arguments,
                            },
                        )
                    } else {
                        // Access

                        let (property, end) = match self.iter.peek()? {
                            Some(Span {
                                value: Token::Identifier(id),
                                source,
                            }) => (*id, source.end),
                            _ => return error!("Expected an identifier"),
                        };

                        self.iter.advance()?;

                        (
                            end,
                            ast::Expression::Access(ast::Access {
                                target: Box::new_in(first_term, self.alloc.clone()),
                                property,
                            }),
                        )
                    }
                }
                _ => {
                    // Function call.

                    if bp::CALL < min_bp || (skipped_line_break && !calls_across_line_breaks) {
                        break;
                    }

                    self.iter.skip_lb()?;

                    if let Some(expr) =
                        self.try_parse_expression(bp::CALL, calls_across_line_breaks)?
                    {
                        let end = expr.source.end;

                        (
                            end,
                            ast::Expression::Call {
                                callee: Box::new_in(first_term, self.alloc.clone()),
                                argument: Box::new_in(expr, self.alloc.clone()),
                            },
                        )
                    } else {
                        break;
                    }
                }
            };

            first_term = Span {
                value,
                source: start..end,
            };
        }

        Ok(first_term)
    }

    /// Expects that the first non-lb token after `{` was already consumed.
    ///
    /// Ends on `}` or [Token::EndOfInput].
    fn parse_module_content(&mut self) -> Result<ast::ModuleContent<'source, A>, Error> {
        let mut items = Vec::new_in(self.alloc.clone());

        loop {
            let visibility = match self.iter.peek()? {
                // Ignore semicolons
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }) => {
                    self.emit_single_token_warning(Warning::UnnecessarySemicolon)?;

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    continue;
                }
                None
                | Some(Span {
                    value: Token::Symbol(Symbol::RightBrace),
                    ..
                }) => break,
                _ => self.parse_visibility()?,
            };

            if let Some(Span {
                value: ast::Visible::Module,
                source,
            }) = &visibility
            {
                self.warnings.push(Span {
                    value: Warning::PubModInModule,
                    source: source.clone(),
                });
            }

            self.iter.skip_lb()?;

            items.push(ast::TopLevelItem {
                visibility,
                statement: if let Some(statement) = self.try_parse_statement()? {
                    statement
                } else {
                    return error!("Expected a statement");
                },
            });

            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }) => {
                    self.opt_emit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Some(Span {
                    value: Token::LineBreak,
                    ..
                }) => self.iter.advance()?,
                Some(Span {
                    value: Token::Symbol(Symbol::RightBrace),
                    ..
                })
                | None => break,
                _ => return error!("Expected ';', '}' or a line break"),
            }
        }

        Ok(ast::ModuleContent(items))
    }

    fn parse_module(&mut self) -> Result<ast::ModuleContent<'source, A>, Error> {
        let content = self.parse_module_content()?;
        match self.iter.peek()? {
            None => Ok(content),
            _ => error!("This '}' does not close anything; consider removing it"),
        }
    }
}

/// Parses a source module commonly obtained from file content.
pub fn parse_module<A: Allocator + Clone>(
    source: &[u8],
    alloc: A,
) -> Result<(ast::ModuleContent<A>, Vec<Span<Warning>, A>), (Error, Range<Index>)> {
    let buf = LookaheadBuffer::new_in(Lexer::new(source, alloc.clone()), alloc.clone());

    let mut context = ParseContext::new(buf, alloc);
    let maybe_module = context.parse_module();
    let ParseContext { iter, warnings, .. } = context;

    maybe_module
        .map(|module| (module, warnings))
        .map_err(|err| (err, iter.iter().cursor().bytes_consumed() as Index..0))
    // TODO
}
