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
use errors::*;
use fallible_iterator::FallibleIterator;
use labuf::LookaheadBuffer;
use span::{Index, Span};
pub use warnings::*;

pub struct ParseContext<
    'source,
    I: FallibleIterator<Item=Span<Token<'source>>, Error=Error>,
    A: Allocator + Clone,
> {
    pub iter: LookaheadBuffer<I, A>,
    pub warnings: Vec<Span<Warning>, A>,
    pub alloc: A,
}

impl<
    'source,
    I: FallibleIterator<Item=Span<Token<'source>>, Error=Error>,
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
                    self.parse_expression(0, false)?
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

    /// Expects the current token to be the first token of the type.
    ///
    /// Ends on past-the-end token.
    fn parse_type(&mut self, min_bp: u8) -> Result<Span<ast::Type<'source, A>>, Error> {
        let first_term = self.parse_type_first_term()?;
        Ok(self.parse_type_remaining_terms(first_term, min_bp)?)
    }

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
                        let is_public = match self.iter.peek()? {
                            Some(Span {
                                     value: Token::Symbol(Symbol::RightParenthesis),
                                     source,
                                 }) => {
                                let end = source.end;

                                self.iter.advance()?;

                                break end;
                            }
                            Some(Span {
                                     value: Token::Keyword(Keyword::Pub),
                                     source,
                                 }) => {
                                let source = source.clone();

                                self.iter.advance()?;
                                self.iter.skip_lb()?;

                                Some(Span { value: (), source })
                            }
                            _ => None,
                        };

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

                        let id = match self.iter.peek()? {
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
                        self.iter.skip_lb()?;

                        let ty = self.parse_type(type_bp::MIN)?;

                        fields.push(ast::ObjectTypeField {
                            is_public,
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
                                self.opt_omit_unnecessary_delimiter_warning(
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

                    Span {
                        source: start..end,
                        value: ast::Type::Object(fields),
                    }
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

                    ty
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

                        tps.value.push(self.parse_type(type_bp::MIN)?);

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
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
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
                        (_, true) => {}
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

    /// Expects `peek()` the token after '('. Ends on ')'. Returns `(fields, end)`.
    fn parse_object_literal(
        &mut self,
    ) -> Result<(Vec<ast::ObjectField<'source, A>, A>, Index), Error> {
        let mut fields = Vec::new_in(self.alloc.clone());

        let end = loop {
            let id = match self.iter.peek()? {
                Some(Span {
                         value: Token::Identifier(id),
                         source,
                     }) => Span {
                    value: *id,
                    source: source.clone(),
                },
                Some(Span {
                         value: Token::Symbol(Symbol::RightParenthesis),
                         source,
                     }) => break source.end,
                token => return error!("Expected an identifier or ')', {:?}", token),
            };

            self.iter.advance()?;
            self.iter.skip_lb()?;

            match self.iter.peek()? {
                Some(Span {
                         value: Token::Symbol(Symbol::Equals),
                         ..
                     }) => {}
                _ => return error!("Expected '='"),
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let value = self.parse_expression(0, false)?;

            fields.push(ast::ObjectField { id, value });

            match self.iter.peek_n_non_lb(0)? {
                (
                    Some(Span {
                             value: Token::Symbol(Symbol::RightParenthesis),
                             source,
                         }),
                    _,
                ) => {
                    let end = source.end;
                    self.iter.skip_lb()?;
                    break end;
                }
                (
                    Some(Span {
                             value: Token::Symbol(Symbol::Comma),
                             ..
                         }),
                    _,
                ) => {
                    self.iter.skip_lb()?;
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                }
                (_, true) => {
                    // Skip the line break.
                    self.iter.advance()?;
                }
                _ => return error!("Expected ',', ')' or a line break"),
            }
        };

        Ok((fields, end))
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

            let ty = self.parse_type(type_bp::MIN)?;

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

                        let ty = self.parse_type(type_bp::MIN)?;

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

                let ty = self.parse_type(type_bp::MIN)?;
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
                // TODO
                pattern: ast::Pattern::WithType(ast::PatternUnitWithType {
                    unit: Span {
                        value: ast::PatternUnit::Any,
                        source: 0..0,
                    },
                    ty: Span {
                        value: ast::Type::Inferred,
                        source: 0..0,
                    },
                }),
                this_parameter,
                body: Box::new_in(body.map(ast::Expression::Block), self.alloc.clone()),
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

        let id = match self.iter.peek()? {
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

        let ty = self.parse_type(type_bp::MIN)?;
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
                let ty = self.parse_type(type_bp::MIN)?;

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

                let expr = self.parse_expression(0, false)?;
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
                     value: Token::Keyword(Keyword::Type),
                     ..
                 }) => {
                let (kind, end) = self.parse_type_statement_kind()?;
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
        match self.try_parse_expression_first_term(calls_across_line_breaks)? {
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

    /// Expects a token.
    fn try_parse_expression_first_term(
        &mut self,
        calls_across_line_breaks: bool,
    ) -> Result<Option<Span<ast::Expression<'source, A>>>, Error> {
        Ok(match self.iter.peek()? {
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
                        _ => return error!("Expected ')'"),
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

                    items.push(self.parse_expression(0, false)?);

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
                    _ => ((), self.parse_expression(0, calls_across_line_breaks)?),
                };

                Some(Span {
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

                    let condition = self.parse_expression(0, true)?;
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

                Some(Span {
                    source: start..end,
                    value: ast::Expression::If {
                        base: ast::If {
                            condition: Box::new_in(condition, self.alloc.clone()),
                            body,
                        },
                        else_ifs,
                        else_body,
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
                             value: Token::Symbol(Symbol::LeftBrace),
                             ..
                         }) => {}
                    _ => return error!("Expected '{'"),
                }

                let body = self.parse_block()?;

                self.iter.advance()?;

                Some(Span {
                    source: start..body.source.end,
                    value: ast::Expression::While {
                        condition: Box::new_in(condition, self.alloc.clone()),
                        body,
                    },
                })
            }
            Some(Span {
                     value: Token::MarkupStartTag(tag_name),
                     source,
                 }) => {
                let tag_name = *tag_name;
                let source = source.clone();

                let mut params = Vec::new_in(self.alloc.clone());

                let end = loop {
                    self.iter.advance()?;

                    let id = match self.iter.peek()? {
                        Some(Span {
                                 value: Token::MarkupClose,
                                 source,
                             }) => break source.end,
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
                            self.parse_expression(0, true)?
                        }
                        _ => unreachable!(),
                    };

                    // TODO: revisit this part

                    params.push(ast::ObjectField { id, value });
                };

                self.iter.advance()?;

                Some(Span {
                    source: source.start..end,
                    value: ast::Expression::Call {
                        callee: Box::new_in(
                            Span {
                                value: ast::Expression::Identifier(tag_name),
                                source: source.clone(),
                            },
                            self.alloc.clone(),
                        ),
                        argument: Box::new_in(
                            Span {
                                value: ast::Expression::Object(params),
                                source: source.end..end, // TODO: range error
                            },
                            self.alloc.clone(),
                        ),
                    },
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
            let (token, skipped_line_break) = self.iter.peek_n_non_lb(0)?; // TODO: marker

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
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    if let Some(Span {
                                    value: Token::Symbol(Symbol::LeftAngle),
                                    ..
                                }) = self.iter.peek_n_non_lb(0)?.0
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

                        // let const_arguments = Vec::new_in(self.alloc.clone());

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
                            ast::Expression::Break,
                            /*
                            CallWithConstParameters {
                                target: Box::new_in(target, self.alloc.clone()),
                                // TODO
                                arguments: ast::CallArguments::Named(Vec::new_in(
                                    self.alloc.clone(),
                                )),
                                const_arguments,
                            },
                            */
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
                _ => {
                    // Function call.

                    if bp::CALL < min_bp || (skipped_line_break && !calls_across_line_breaks) {
                        break;
                    }

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
