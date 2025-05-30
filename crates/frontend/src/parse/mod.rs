mod unit_tests;
mod warnings;

pub mod ast;
pub mod bp;
mod labext;

use crate::lex::{Keyword, Symbol, Token};
use crate::parse::labext::LabExt;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
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
    fn omit_single_token_warning(&mut self, warning: Warning) -> Result<(), Error> {
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
    /// it adds the warning with the source of the token of [LookaheadBuffer::peek]
    /// before [LookaheadBuffer::skip_lb] was called.
    #[inline]
    fn opt_omit_unnecessary_delimiter_warning(&mut self, warning: Warning) -> Result<(), Error> {
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

    /// If `peek()` is a '<', then it parses type parameters.
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

    /// Parses type declarations like this:
    ///
    /// ```text
    /// fn<A, B>
    ///    ^
    /// ```
    ///
    /// Expects the next token to be marked. Ends on the non-lb token after `>`.
    fn parse_const_parameters(&mut self) -> Result<ast::ConstParameters<'source, A>, Error> {
        let mut params = Vec::new_in(self.alloc.clone());

        loop {
            match self.iter.peek()? {
                Some(Span {
                    value: Token::Identifier(id),
                    source,
                }) => {
                    params.push(Span {
                        value: ast::ConstParameter::Type {
                            id,
                            trait_bounds: Vec::new_in(self.alloc.clone()),
                        },
                        source: source.clone(),
                    }); // TODO: Add traits
                }
                Some(Span {
                    value: Token::Symbol(Symbol::RightAngle),
                    ..
                }) => break,
                _ => return error!("Expected type parameter"),
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            // TODO: Add lf for tp separation
            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::RightAngle),
                    ..
                }) => break,
                Some(Span {
                    value: Token::Symbol(Symbol::Comma),
                    ..
                }) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                }
                _ => return error!("Expected ',' or '>'"),
            }
        }

        self.iter.advance()?;
        self.iter.skip_lb()?;

        Ok(params)
    }

    /// Parses a block.
    ///
    /// Expects `peek()` to be [Symbol::LeftBrace]. Ends on [Symbol::RightBrace].
    fn parse_block(
        &mut self,
    ) -> Result<Span<Vec<Span<ast::StatementOrExpression<'source, A>>, A>>, Error> {
        let start = self.iter.peek()?.unwrap().source.start;

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let mut items = Vec::new_in(self.alloc.clone());

        let end = loop {
            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::RightBrace),
                    source,
                }) => break source.end,
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                    continue;
                }
                _ => {}
            }

            if let Some(statement) = self.try_parse_statement()? {
                items.push(statement.map(|s| ast::StatementOrExpression::Statement(s)));
            } else {
                items.push(
                    self.parse_expression(0)?
                        .map(|e| ast::StatementOrExpression::Expression(e)),
                );
            }

            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Some(Span {
                    value: Token::LineBreak,
                    ..
                }) => self.iter.advance()?,
                Some(Span {
                    value: Token::Symbol(Symbol::RightBrace),
                    source,
                }) => break source.end,
                _ => return error!("Expected ';', '}' or a line break"),
            }
        };

        Ok(Span {
            value: items,
            source: start..end,
        })
    }

    fn parse_type(&mut self) -> Result<Span<ast::Type<'source, A>>, Error> {
        let first_term = self.parse_type_first_term()?;
        Ok(self.parse_type_remaining_terms(first_term, 0)?)
    }

    /// Expects that the first token of the type is accessible via `peek()`.
    ///
    /// Ends on past-the-end token.
    fn parse_type_first_term(&mut self) -> Result<Span<ast::Type<'source, A>>, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::ExclamationMark),
                source,
            }) => {
                let source = source.clone();

                self.iter.advance()?;

                Span {
                    value: ast::Type::Never,
                    source,
                }
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let source = source.clone();
                let id = *id;

                let path = self.parse_item_path(id)?;

                let const_parameters = if let Some(Span {
                    value: Token::Symbol(Symbol::LeftAngle),
                    ..
                }) = self.iter.peek_n_non_lb(0)?.0
                {
                    self.iter.skip_lb()?;

                    let mut tps = Span {
                        value: Vec::new_in(self.alloc.clone()),
                        source: self.iter.peek()?.unwrap().source.start..0,
                    };

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    loop {
                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::RightAngle),
                                ..
                            }) => break,
                            Some(Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            }) => {
                                self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;
                            }
                            _ => {}
                        }

                        tps.value.push(self.parse_type()?);

                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::RightAngle),
                                ..
                            }) => break,
                            Some(Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            }) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            Some(Span {
                                value: Token::LineBreak,
                                ..
                            }) => self.iter.advance()?,
                            _ => return error!("Expected ',', '}' or a line break"),
                        }
                    }

                    self.iter.advance()?;

                    tps
                } else {
                    Span {
                        value: Vec::new_in(self.alloc.clone()),
                        source: source.clone(),
                    }
                };

                Span {
                    source: source.start..const_parameters.source.end,
                    value: ast::Type::Item(ast::ItemRef {
                        path,
                        const_parameters,
                    }),
                }
            }
            _ => return error!("Expected an identifier or '!' (the never type)"),
        })
    }

    fn parse_type_remaining_terms(
        &mut self,
        left_term: Span<ast::Type<'source, A>>,
        bp: u8,
    ) -> Result<Span<ast::Type<'source, A>>, Error> {
        todo!()
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
                let mut source = source.clone();
                let mut vec = Vec::new_in(self.alloc.clone());

                self.iter.advance()?;
                self.iter.skip_lb()?;
                
                loop {
                    let value = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Identifier(id),
                            source,
                        }) => {
                            let id = Span {
                                value: *id,
                                source: source.clone(),
                            };

                            self.iter.advance()?;
                            
                            self.parse_use(id)?
                        }
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            ..
                        }) => {
                            self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                            continue;
                        }
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            source: new_source,
                        }) => {
                            source.end = new_source.end;
                            self.iter.advance()?;
                            break;
                        }
                        _ => return error!("Expected an identifier or ')'"),
                    };

                    vec.push(value);

                    match self.iter.peek_n_non_lb(0)? {
                        (
                            Some(Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            }),
                            _,
                        ) => {
                            if self.iter.skip_lb()? {
                                self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                            }
                        }
                        (
                            Some(Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                source: new_source,
                            }),
                            _,
                        ) => {
                            let new_source = new_source.clone();

                            self.iter.skip_lb()?;
                            source.end = new_source.end;
                            self.iter.advance()?;

                            break;
                        }
                        (_, true) => {},
                        _ => return error!("Expected ',', ')' or a line break"),
                    }

                    self.iter.advance()?
                }

                Span {
                    value: ast::UseChild::Multiple(vec),
                    source,
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

    /// Expects [Buffered::peek] to yield [Token::Identifier].
    /// Ends on the token after the last path segment (greedy).
    fn parse_item_path(
        &mut self,
        mut first_id: &'source str,
    ) -> Result<Span<ast::ItemPath<'source, A>>, Error> {
        let mut source = self.iter.peek()?.unwrap().source.clone();

        self.iter.advance()?;

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
                    }) => {
                        source.end = new_source.end;
                        *id
                    }
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

        Ok(Span {
            value: ast::ItemPath {
                parents,
                id: first_id,
            },
            source,
        })
    }

    /// Parses the parameters of a function expression.
    ///
    /// Expects `peek()` to be the non-lb token after `(`. Ends on `)`.
    fn parse_fn_parameters(
        &mut self,
        parameters: &mut Vec<ast::Parameter<'source, A>, A>,
    ) -> Result<(), Error> {
        loop {
            let is_mutable = match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::RightParenthesis),
                    ..
                }) => break,
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

            let id = match self.iter.peek()? {
                Some(Span {
                    value: Token::Identifier(id),
                    ..
                }) => *id,
                _ => return error!("Expected an identifier"),
            };

            self.iter.advance()?;
            self.iter.skip_lb()?;

            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::Colon),
                    ..
                }) => {}
                _ => return error!("Expected ':'"),
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let ty = self.parse_type()?;

            parameters.push(ast::Parameter { id, is_mutable, ty });

            let lb = self.iter.skip_lb()?;

            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::RightParenthesis),
                    ..
                }) => break,
                Some(Span {
                    value: Token::Symbol(Symbol::Comma),
                    ..
                }) => self.iter.advance()?,
                _ if lb => {}
                _ => return error!("Expected ',', ')' or a line break"),
            }
        }

        Ok(())
    }

    fn parse_fn_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let const_parameters = self.parse_opt_const_parameters()?;

        let id = match self.iter.peek()? {
            Some(Span {
                value: Token::Identifier(id),
                ..
            }) => *id,
            _ => return error!("Expected an identifier"),
        };

        self.iter.advance()?;
        self.iter.skip_lb()?;

        match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                ..
            }) => {}
            Some(Span {
                value: Token::Symbol(Symbol::LeftAngle),
                ..
            }) => {
                return error!(
                    "Type parameter are declared after 'fn', not after the function name."
                )
            }
            _ => return error!("Expected '('"),
        }

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let mut parameters = Vec::new_in(self.alloc.clone());

        let this_parameter = match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::This),
                ..
            }) => {
                self.iter.advance()?;
                let lb = self.iter.skip_lb()?;

                match self.iter.peek()? {
                    _ if lb => {}
                    Some(Span {
                        value: Token::Symbol(Symbol::Comma),
                        ..
                    }) => {
                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                    }
                    _ => return error!("Expected ',' or a line break"),
                }

                Some(ast::ThisParameter::This)
            }
            Some(Span {
                value: Token::Keyword(Keyword::Mut),
                ..
            }) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    // Case: `mut this`
                    Some(Span {
                        value: Token::Keyword(Keyword::This),
                        ..
                    }) => {
                        self.iter.advance()?;
                        let lb = self.iter.skip_lb()?;

                        match self.iter.peek()? {
                            _ if lb => {}
                            Some(Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                ..
                            }) => {}
                            Some(Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            }) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            _ => return error!("Expected ',', ')' or a line break"),
                        }

                        Some(ast::ThisParameter::ThisMut)
                    }

                    // In this case we parse the first parameter ourselves.
                    Some(Span {
                        value: Token::Identifier(id),
                        ..
                    }) => {
                        let id = *id;

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::Colon),
                                ..
                            }) => {}
                            _ => return error!("Expected ':'"),
                        }

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let ty = self.parse_type()?;

                        parameters.push(ast::Parameter {
                            id,
                            is_mutable: true,
                            ty,
                        });

                        let lb = self.iter.skip_lb()?;

                        match self.iter.peek()? {
                            _ if lb => {}
                            Some(Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                ..
                            }) => {}
                            Some(Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            }) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            _ => return error!("Expected ',', ')' or a line break"),
                        }

                        None
                    }
                    _ => return error!("Expected an identifier or 'this'"),
                }
            }
            _ => None,
        };

        self.parse_fn_parameters(&mut parameters)?;

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let return_type = match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::MinusRightAngle),
                ..
            }) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let ty = self.parse_type()?;
                self.iter.skip_lb()?;

                ty
            }
            Some(Span { source, .. }) => Span {
                value: ast::Type::Inferred,
                source: source.start..source.start,
            },
            _ => return error!("Expected something"), // TODO
        };

        // Validate that the block starts with `{`
        match self.iter.peek()? {
            Some(Span {
                value: Token::Symbol(Symbol::LeftBrace),
                ..
            }) => {}
            _ => return error!("Expected '{'"),
        }

        let body = self.parse_block()?;
        let end = body.source.end;

        self.iter.advance()?;

        Ok((
            ast::StatementKind::Function {
                const_parameters: Vec::new_in(self.alloc.clone()),
                input_type: Span {
                    // TODO
                    value: ast::Type::Inferred,
                    source: Default::default(),
                },
                output_type: return_type,
                id,
                pattern: (),
                this_parameter,
                body: Box::new_in(body.map(ast::Expression::Block), self.alloc.clone()),
            },
            end,
        ))
    }

    fn parse_struct_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let const_parameters = self.parse_opt_const_parameters()?;

        let id = match self.iter.peek()? {
            Some(Span {
                value: Token::Identifier(id),
                ..
            }) => *id,
            _ => return error!("Expected an identifier"),
        };

        let mut fields = Vec::new_in(self.alloc.clone());

        self.iter.advance()?;

        match self.iter.peek_n_non_lb(0)? {
            (
                Some(Span {
                    value: Token::Symbol(Symbol::LeftParenthesis),
                    ..
                }),
                _,
            ) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let end: Index = loop {
                    let start = match self.iter.peek()? {
                        Some(Span { source, .. }) => source.start,
                        _ => return error!("Expected something"), // TODO
                    };

                    let is_public = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Keyword(Keyword::Pub),
                            ..
                        }) => {
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
                            true
                        }
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            source,
                        }) => break source.end,
                        _ => false,
                    };

                    let is_mutable = match self.iter.peek()? {
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

                    let id = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Identifier(id),
                            ..
                        }) => *id,
                        _ => return error!("Expected an identifier"),
                    };

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let ty = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::Colon),
                            ..
                        }) => {
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
                            self.parse_type()?
                        }
                        _ => return error!("Expected ':'"),
                    };

                    fields.push(Span {
                        source: start..ty.source.end,
                        value: ast::ObjectTypeField {
                            is_public,
                            is_mutable,
                            id,
                            ty,
                        },
                    });

                    let lb = self.iter.skip_lb()?;

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            source,
                        }) => break source.end,
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            ..
                        }) => {
                            self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                        }
                        _ if lb => {}
                        _ => return error!("Expected ',', ')' or a line break"),
                    }
                };

                self.iter.advance()?;
            }
            (_, true) => {}
            _ => {
                return error!(
                "Expected struct content (starting with '(') or a delimiter (';' or a line break)"
            )
            }
        }

        Ok((
            ast::StatementKind::Type {
                id,
                const_parameters,
                ty: Span {
                    // TODO
                    value: ast::Type::Inferred,
                    source: Default::default(),
                },
            },
            0, // TODO
        ))
    }

    fn parse_enum_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        todo!()
    }

    fn parse_mod_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let (id, mut end) = match self.iter.peek()? {
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => (*id, source.end),
            _ => return error!("Expected an identifier"),
        };

        self.iter.advance()?;

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
            _ => {
                return error!(
                "Expected module content (starting with '{') or a delimiter (';' or a line break)"
            )
            }
        };

        Ok((ast::StatementKind::Module { id, content }, end))
    }

    fn parse_let_statement_kind(
        &mut self,
    ) -> Result<(ast::StatementKind<'source, A>, Index), Error> {
        self.iter.advance()?;
        self.iter.skip_lb()?;

        let is_mutable = match self.iter.peek()? {
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

        let (id, mut end) = match self.iter.peek()? {
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => (*id, source.end),
            _ => return error!("Expected an identifier"),
        };

        self.iter.advance()?;

        let ty = match self.iter.peek_n_non_lb(0)? {
            (
                Some(Span {
                    value: Token::Symbol(Symbol::Colon),
                    ..
                }),
                _,
            ) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;
                let ty = self.parse_type()?;

                end = ty.source.end; // Adjust end of statement
                ty
            }
            _ => Span {
                // TODO
                value: ast::Type::Inferred,
                source: Default::default(),
            },
        };

        let value = match self.iter.peek_n_non_lb(0)? {
            // If the next non-line-break token is '=', then an expression is parsed.
            (Some(Span { value: Token::Symbol(Symbol::Equals), .. }), _) => {
                self.iter.skip_lb()?;
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let expr = self.parse_expression(0)?;
                end = expr.source.end; // Adjust end of statement
                Some(expr)
            }

            // If it is not a '=' and there is a line break between this token and the previous,
            // then this token does not belong to this statement and there is no value.
            (_, true) => None,

            // Else (there is a token that is not separated by a line break), short-circuit.
            _ => return error!("Expected an initialization (starting with '=') or a delimiter (';' or a line break)"),
        };

        Ok((
            ast::StatementKind::Let {
                is_mutable,
                ty,
                id,
                value: value.map(|v| Box::new_in(v, self.alloc.clone())),
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
    fn try_parse_statement(&mut self) -> Result<Option<Span<ast::Statement<'source, A>>>, Error> {
        let mut source = match self.iter.peek()? {
            Some(Span { source, .. }) => source.clone(),
            None => return Ok(None), // TODO: look
        };

        let mut doc_comments = Vec::new_in(self.alloc.clone());

        // Collect all doc comments.
        loop {
            match self.iter.peek()? {
                Some(Span {
                    value: Token::DocComment(doc_comment),
                    ..
                }) => doc_comments.push(*doc_comment),
                _ => break,
            }
            self.iter.advance()?;
        }

        let mut annotations = Vec::new_in(self.alloc.clone());

        // Collect all available annotations associated with the statement.
        loop {
            match self.iter.peek()? {
                Some(Span {
                    value: Token::Symbol(Symbol::At),
                    ..
                }) => {}
                _ => {
                    self.iter.skip_lb()?;
                    break;
                }
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let id = match self.iter.peek()? {
                Some(Span {
                    value: Token::Identifier(id),
                    ..
                }) => *id,
                _ => return error!("Expected an identifier"),
            };

            let path = self.parse_item_path(id)?;

            annotations.push(ast::Annotation {
                path,
                arguments: Vec::new_in(self.alloc.clone()),
            });
        }

        let statement_kind: Option<ast::StatementKind<'source, A>> = match self.iter.peek()? {
            Some(Span {
                value: Token::Keyword(Keyword::Fn),
                ..
            }) => {
                let (kind, end) = self.parse_fn_statement_kind()?;
                source.end = end;
                Some(kind)
            }
            Some(Span {
                value: Token::Keyword(Keyword::Mod),
                ..
            }) => {
                let (kind, end) = self.parse_mod_statement_kind()?;
                source.end = end;
                Some(kind)
            }
            Some(Span {
                value: Token::Keyword(Keyword::Struct),
                ..
            }) => {
                let (kind, end) = self.parse_struct_statement_kind()?;
                source.end = end;
                Some(kind)
            }

            // Schema (brackets denote optionals, angles denote other constructs):
            //
            // let [mut] <variable_name>[: <type>] [= <expr>]
            Some(Span {
                value: Token::Keyword(Keyword::Let),
                ..
            }) => {
                let (kind, end) = self.parse_let_statement_kind()?;
                source.end = end;
                Some(kind)
            }

            // use a
            // use b.c
            // use d.*
            // use d.(x, y, z.*)
            Some(Span {
                value: Token::Keyword(Keyword::Use),
                ..
            }) => {
                if doc_comments.len() > 0 {
                    // TODO: Maybe change this from being an error (?)
                    return error!("Doc comments cannot be attached to use statements");
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let root_id = match self.iter.peek()? {
                    Some(Span {
                        value: Token::Identifier(id),
                        source,
                    }) => Span {
                        value: *id,
                        source: source.clone(),
                    },
                    _ => return error!("Expected an identifier"),
                };

                self.iter.advance()?;
                
                let u = self.parse_use(root_id)?;
                source = u.source();
                
                Some(ast::StatementKind::Use(u))
            }
            Some(Span {
                value: Token::Keyword(Keyword::Break),
                ..
            }) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;
                Some(ast::StatementKind::Break)
            }
            _ => {
                if doc_comments.len() > 0 {
                    return error!("Doc comments are not attached to a statement");
                }

                if annotations.len() > 0 {
                    return error!("Annotations are not attached to a statement");
                }

                None
            }
        };

        Ok(statement_kind.map(|statement_kind| Span {
            value: ast::Statement {
                annotations,
                statement_kind,
            },
            source,
        }))
    }

    /// Parses an expression.
    ///
    /// Expects `peek()` to be the first token of the expression. May end on anything.
    pub fn parse_expression(
        &mut self,
        min_bp: u8,
    ) -> Result<Span<ast::Expression<'source, A>>, Error> {
        let first_term = self.parse_expression_first_term()?;
        self.parse_expression_remaining_terms(first_term, min_bp)
    }

    /// Expects a token.
    fn parse_expression_first_term(&mut self) -> Result<Span<ast::Expression<'source, A>>, Error> {
        Ok(match self.iter.peek()? {
            Some(Span {
                value: Token::String(s),
                source,
            }) => {
                let source = source.clone();
                let s = s.clone();

                self.iter.advance()?;

                Span {
                    value: ast::Expression::String(s),
                    source,
                }
            }
            Some(Span {
                value: Token::Number(n),
                source,
            }) => {
                let source = source.clone();
                let n = *n;

                self.iter.advance()?;

                Span {
                    value: ast::Expression::Number(n),
                    source,
                }
            }
            Some(Span {
                value: Token::Identifier(id),
                source,
            }) => {
                let id = *id;
                let source = source.clone();

                self.iter.advance()?;

                Span {
                    value: ast::Expression::Identifier(id),
                    source,
                }
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftParenthesis),
                source,
            }) => {
                // Object expression

                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let mut fields = Vec::new_in(self.alloc.clone());

                loop {
                    let is_mutable = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::RightParenthesis),
                            ..
                        }) => break,
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            ..
                        }) => {
                            self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
                            continue;
                        }
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

                    let (id, id_source) = match self.iter.peek()? {
                        Some(Span {
                            value: Token::Identifier(id),
                            source,
                        }) => (*id, source.clone()),
                        _ => return error!("Expected an identifier"),
                    };

                    self.iter.advance()?;

                    /*
                    let ty = match self.iter.peek_n_non_lb(0)? {
                        (
                            Span {
                                value: Token::Symbol(Symbol::Colon),
                                ..
                            },
                            _,
                        ) => {
                            self.iter.skip_lb()?;
                            self.iter.advance()?;
                            self.iter.skip_lb()?;

                            Some(self.parse_type()?)
                        }
                        _ => None,
                    };
                     */

                    let init = match self.iter.peek_n_non_lb(0)? {
                        (Some(Span { value: Token::Symbol(Symbol::Equals), .. }), _) => {
                            self.iter.skip_lb()?;
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
                            self.parse_expression(0)?
                        }
                        (Some(Span { value: Token::Symbol(Symbol::RightParenthesis | Symbol::Comma), .. }), _) | (_, true) => Span {
                            value: ast::Expression::Identifier(id),
                            source: id_source,
                        },
                        _ => return error!("Expected an initialization (starting with '='), a type (starting with ':'), ')' or a delimiter (';' or a line break)"),
                    };

                    // TODO
                    // fields.push(ast::ObjectField {
                    //     field: "",
                    //     value: Span {},
                    // });

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            source,
                        }) => {
                            let source = source.clone();
                            self.iter.advance()?;

                            // Capture ",\n" and ",)" groups
                            if self.iter.skip_lb()?
                                || match self.iter.peek()? {
                                    Some(Span {
                                        value: Token::Symbol(Symbol::RightParenthesis),
                                        ..
                                    }) => true,
                                    _ => false,
                                }
                            {
                                self.warnings.push(Span {
                                    value: Warning::UnnecessaryComma,
                                    source,
                                })
                            }
                        }
                        Some(Span {
                            value: Token::LineBreak,
                            ..
                        }) => self.iter.advance()?,
                        _ => {}
                    }
                }

                // peek() == ')'

                // TODO
                // first_source.end = self.iter.peek()?.unwrap().source.end;
                self.iter.advance()?;

                Span {
                    source: start..1000000, // TODO
                    value: ast::Expression::Object(fields),
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

                let end = loop {
                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::RightBracket),
                            source,
                        }) => break source.end,
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            ..
                        }) => {
                            self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
                            continue;
                        }
                        _ => {}
                    }

                    items.push(self.parse_expression(0)?);

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::LineBreak,
                            ..
                        }) => self.iter.advance()?,
                        Some(Span {
                            value: Token::Symbol(Symbol::Comma),
                            ..
                        }) => {
                            self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                        }
                        _ => {}
                    }
                };

                self.iter.advance()?;

                Span {
                    value: ast::Expression::Array(items),
                    source: start..end,
                }
            }
            Some(Span {
                value: Token::Symbol(Symbol::LeftBrace),
                source,
            }) => {
                let start = source.start;
                let block = self.parse_block()?;

                self.iter.advance()?;

                Span {
                    source: start..block.source.end,
                    value: ast::Expression::Block(block.value),
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::Fn),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let const_parameters = self.parse_opt_const_parameters()?;

                let mut parameters = Vec::new_in(self.alloc.clone());

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftParenthesis),
                        ..
                    }) => {}
                    _ => return error!("Expected '('."),
                };

                self.iter.advance()?;
                self.iter.skip_lb()?;

                self.parse_fn_parameters(&mut parameters)?;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let (return_type, body) = match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::MinusRightAngle),
                        ..
                    }) => return error!("Closures cannot have return type annotations."),
                    _ => ((), self.parse_expression(0)?),
                };

                Span {
                    source: start..body.source.end,
                    value: ast::Expression::Function {
                        // TODO
                        // signature: ast::FunctionSignature {
                        //     const_parameters,
                        //     return_type,
                        //     parameters,
                        // },
                        body: Box::new_in(body, self.alloc.clone()),
                    },
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::If),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let condition = self.parse_expression(0)?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftBrace),
                        ..
                    }) => {}
                    _ => return error!("Expected '{'"),
                }

                let body = self.parse_block()?;

                let mut else_ifs = Vec::new_in(self.alloc.clone());

                let else_body = loop {
                    self.iter.advance()?;

                    match self.iter.peek_n_non_lb(0)?.0 {
                        Some(Span {
                            value: Token::Keyword(Keyword::Else),
                            ..
                        }) => {}
                        _ => break None,
                    }

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Keyword(Keyword::If),
                            ..
                        }) => {}
                        Some(Span {
                            value: Token::Symbol(Symbol::LeftBrace),
                            ..
                        }) => {
                            let block = self.parse_block()?;
                            self.iter.advance()?;
                            break Some(block);
                        }
                        _ => return error!("Expected `if` or '{'"),
                    }

                    // else-if-branch

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek()? {
                        Some(Span {
                            value: Token::Symbol(Symbol::LeftBrace),
                            ..
                        }) => {}
                        _ => return error!("Expected '{' after 'else if' condition"),
                    }

                    let body = self.parse_block()?;

                    else_ifs.push(ast::If {
                        condition: Box::new_in(condition, self.alloc.clone()),
                        body,
                    })
                };

                let end = else_body
                    .as_ref()
                    .map(|else_body| else_body.source.end)
                    .or_else(|| else_ifs.last().map(|else_if| else_if.body.source.end))
                    .unwrap_or_else(|| body.source.end);

                Span {
                    source: start..end,
                    value: ast::Expression::If {
                        base: ast::If {
                            condition: Box::new_in(condition, self.alloc.clone()),
                            body,
                        },
                        else_ifs,
                        else_body,
                    },
                }
            }
            Some(Span {
                value: Token::Keyword(Keyword::While),
                source,
            }) => {
                let start = source.start;

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let condition = self.parse_expression(0)?;
                self.iter.skip_lb()?;

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::Symbol(Symbol::LeftBrace),
                        ..
                    }) => {}
                    _ => return error!("Expected '{'"),
                }

                let body = self.parse_block()?;

                self.iter.advance()?;

                Span {
                    source: start..body.source.end,
                    value: ast::Expression::While {
                        condition: Box::new_in(condition, self.alloc.clone()),
                        body,
                    },
                }
            }
            Some(Span {
                value: Token::MarkupStartTag(tag_name),
                source,
            }) => {
                let tag_name = *tag_name;
                let source = source.clone();

                let mut params = Vec::new_in(self.alloc.clone());

                loop {
                    self.iter.advance()?;

                    let key = match self.iter.peek()? {
                        Some(Span {
                            value: Token::MarkupClose,
                            ..
                        }) => break,
                        Some(Span {
                            value: Token::MarkupKey(key),
                            source,
                        }) => Span {
                            value: *key,
                            source: source.clone(),
                        },
                        _ => todo!("markup children"),
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
                            self.parse_expression(0)?
                        }
                        _ => unreachable!(),
                    };

                    params.push((key, value));
                }

                self.iter.advance()?;

                Span {
                    source: source.start..0,
                    value: ast::Expression::Call {
                        target: Box::new_in(
                            Span {
                                value: ast::Expression::Identifier(tag_name),
                                source,
                            },
                            self.alloc.clone(),
                        ),
                        arguments: ast::CallArguments::Named(params),
                    },
                }
            }
            _ => return error!("Invalid start of expression"),
        })
    }

    /*
    /// Parses call arguments. Expects `peek()` to be '(' or a line break followed by a '('.
    /// Ends on the token after ')'.
    fn parse_call_arguments(&mut self) -> Result<ast::CallArguments<'source, A>, Error> {
        self.iter.skip_lb()?;
        self.iter.advance()?;
        self.iter.skip_lb()?;

        // Skip initial commas
        loop {
            match self.iter.peek()?.value {
                Token::Symbol(Symbol::Comma) => {
                    self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                }
                _ => break,
            }
        }

        let arguments = if let Span {
            value: Token::Identifier(first_property),
            source: first_source,
        } = self.iter.peek()?.clone()
            && let Token::Symbol(Symbol::Equals) = self.iter.peek_n_non_lb(1)?.0.value
        {
            let mut arguments = Vec::new_in(self.alloc.clone());

            self.iter.advance()?; // Skip property
            self.iter.skip_lb()?;
            self.iter.advance()?; // Skip '='
            self.iter.skip_lb()?;

            arguments.push((
                Span {
                    value: first_property,
                    source: first_source.clone(),
                },
                self.parse_expression(bp::COMMA_AND_SEMICOLON)?,
            ));

            match self.iter.peek()?.value {
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::Comma) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                }
                Token::Symbol(Symbol::RightParenthesis) => {}
                _ => return error!("Expected ')', ',' or a line break"),
            }

            loop {
                match self.iter.peek()?.value {
                    Token::Symbol(Symbol::Comma) => {
                        self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;
                        continue;
                    }
                    Token::LineBreak => {
                        self.iter.advance()?;
                        continue;
                    }
                    Token::Symbol(Symbol::RightParenthesis) => break,
                    _ => {}
                }

                let arg = match self.iter.peek()? {
                    Span {
                        value: Token::Identifier(id),
                        source,
                    } => Span {
                        value: *id,
                        source: source.clone(),
                    },
                    _ => return error!("Expected identifier (property)."),
                };

                self.iter.advance()?; // Skip property
                self.iter.skip_lb()?;

                match self.iter.peek()?.value {
                    Token::Symbol(Symbol::Equals) => {}
                    _ => return error!("Expected '='"),
                }

                self.iter.advance()?; // Skip '='
                self.iter.skip_lb()?;

                arguments.push((arg, self.parse_expression(bp::COMMA_AND_SEMICOLON)?));

                match self.iter.peek()? {
                    Some(Span {
                        value: Token::LineBreak,
                        ..
                    }) => self.iter.advance()?,
                    Some(Span {
                        value: Token::Symbol(Symbol::Comma),
                        ..
                    }) => {
                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                    }
                    Some(Span {
                        value: Token::Symbol(Symbol::RightParenthesis),
                        ..
                    }) => {}
                    _ => return error!("Expected ')', ',' or a line break"),
                }
            }

            ast::CallArguments::Named(arguments)
        } else if let Token::Symbol(Symbol::RightParenthesis) = self.iter.peek()?.value {
            // There are no arguments.
            ast::CallArguments::Named(Vec::new_in(self.alloc.clone()))
        } else {
            // Single, unnamed argument.

            let expr = self.parse_expression(bp::COMMA_AND_SEMICOLON)?;

            self.iter.skip_lb()?; // A line break here has no semantic meaning.

            match self.iter.peek()? {
                Some(Span { value: Token::Symbol(Symbol::RightParenthesis), .. }) => {}
                _ => return error!("Expected ')'. If you want to call with multiple parameters, you have to name them like `a = ?, b = ?`...")
            };

            ast::CallArguments::Single(Box::new_in(expr, self.alloc.clone()))
        };

        self.iter.advance()?; // Skip ')'

        Ok(arguments)
    }
    */

    fn parse_expression_remaining_terms(
        &mut self,
        mut first_term: Span<ast::Expression<'source, A>>,
        min_bp: u8,
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

                let right = self.parse_expression($bp.1)?;

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

                let right = self.parse_expression(bp::ASSIGNMENT.1)?;

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
            let (token, _line_break) = self.iter.peek_n_non_lb(1)?; // TODO: marker

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

                // Other
                Some(Token::Symbol(Symbol::LeftParenthesis)) => {
                    if bp::CALL < min_bp {
                        break;
                    }

                    // TODO
                    // let args = self.parse_call_arguments()?;
                    let end = self.iter.peek()?.unwrap().source.end;

                    self.iter.advance()?;

                    (
                        end,
                        ast::Expression::Call {
                            target: Box::new_in(first_term, self.alloc.clone()),
                            // TODO
                            arguments: ast::CallArguments::Named(Vec::new_in(self.alloc.clone())),
                        },
                    )
                }
                Some(Token::Symbol(Symbol::Dot)) => {
                    if let Some(Span {
                        value: Token::Symbol(Symbol::LeftAngle),
                        ..
                    }) = self.iter.peek_n_non_lb(1)?.0
                    {
                        let target = match first_term {
                            Span {
                                value: ast::Expression::Access(a),
                                source,
                            } => Span {
                                value: ast::ConstParametersCallTarget::Access(a),
                                source,
                            },
                            Span {
                                value: ast::Expression::OptionalAccess(a),
                                source,
                            } => Span {
                                value: ast::ConstParametersCallTarget::OptionalAccess(a),
                                source,
                            },
                            Span {
                                value: ast::Expression::Identifier(i),
                                source,
                            } => Span {
                                value: ast::ConstParametersCallTarget::Identifier(i),
                                source,
                            },
                            _ => return error!("Cannot call an expression with const parameters."),
                        };

                        self.iter.skip_lb()?;
                        self.iter.advance()?; // Skip '.'
                        self.iter.skip_lb()?;
                        self.iter.advance()?; // Skip '<'
                        self.iter.skip_lb()?;

                        let const_arguments = Vec::new_in(self.alloc.clone());

                        // TODO: implement this

                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::RightAngle),
                                ..
                            }) => {}
                            _ => todo!(),
                        }

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        match self.iter.peek()? {
                            Some(Span {
                                value: Token::Symbol(Symbol::LeftParenthesis),
                                ..
                            }) => {}
                            _ => return error!("Expected '(' (call arguments)"),
                        }

                        // TODO
                        // let arguments = self.parse_call_arguments()?;
                        let end = self.iter.peek()?.unwrap().source.end;

                        (
                            end,
                            ast::Expression::CallWithConstParameters {
                                target: Box::new_in(target, self.alloc.clone()),
                                // TODO
                                arguments: ast::CallArguments::Named(Vec::new_in(
                                    self.alloc.clone(),
                                )),
                                const_arguments,
                            },
                        )
                    } else {
                        if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                            break;
                        }

                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

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
                /*
                Token::Symbol(Symbol::QuestionMarkDot) => {
                    if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let property = match self.iter.peek()?.value {
                        Token::Identifier(id) => id,
                        _ => return error!("Expected an identifier"),
                    };

                    let end = self.iter.peek()?.source.end;

                    self.iter.advance()?;

                    (
                        end,
                        ast::Expression::OptionalAccess(ast::Access {
                            target: Box::new_in(first_term, self.alloc.clone()),
                            property,
                        }),
                    )
                }
                 */
                _ => break,
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
            let is_public = match self.iter.peek()? {
                // Ignore semicolons
                Some(Span {
                    value: Token::Symbol(Symbol::Semicolon),
                    ..
                }) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    continue;
                }
                None
                | Some(Span {
                    value: Token::Symbol(Symbol::RightBrace),
                    ..
                }) => break,
                Some(Span {
                    value: Token::Keyword(Keyword::Pub),
                    ..
                }) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    true
                }
                _ => false,
            };

            items.push(ast::TopLevelItem {
                is_public,
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
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
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

/*
/// Parses a source module commonly obtained from file content.
pub fn parse_module<LexA: Allocator + Clone, ParseA: Allocator + Clone>(
    source: &[u8],
    lex_alloc: LexA,
    parse_alloc: ParseA,
) -> Result<(ast::ModuleContent<ParseA>, Vec<Span<Warning>, ParseA>), (Error, Index)> {
    let buf = LookaheadBuffer::new(Lexer::new(source, lex_alloc));

    let mut context = ParseContext::new(buf, parse_alloc);
    let maybe_module = context.parse_module();
    let ParseContext { iter, warnings, .. } = context;

    maybe_module
        .map(|module| (module, warnings))
        .map_err(|err| (err, iter.iter().cursor().index()))
}
 */
