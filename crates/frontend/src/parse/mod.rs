mod tests;
mod warnings;

pub mod ast;
pub mod bp;

use crate::lex::{Keyword, Lexer, Symbol, Token};
use alloc::boxed::Box;
use alloc::vec::Vec;
use fallible_iterator::FallibleIterator;
use labuf::LookaheadBuffer;
use ast::*;
use errors::*;
use span::Span;
pub use warnings::*;

pub struct ParseContext<'source, I: FallibleIterator<Item = Span<Token<'source>>, Error = Error>> {
    pub iter: LookaheadBuffer<I>,
    pub warnings: Vec<Span<Warning>>,
}

impl<'source, I: FallibleIterator<Item = Span<Token<'source>>, Error = Error>> ParseContext<'source, I> {
    #[inline]
    pub const fn new(iter: LookaheadBuffer<I>) -> Self {
        Self {
            warnings: Vec::new(),
            iter,
        }
    }

    /// Adds a warning with the span of the token returned by [Buffered::peek].
    #[inline]
    fn omit_single_token_warning(&mut self, warning: Warning) -> Result<(), Error> {
        let new_source = self.iter.peek()?.source.clone();

        // If there is a last added warning that is equal to the new warning and has is extendable, extend it!
        if warning.is_extendable()
            && let Some(Span { value, source }) = self.warnings.last_mut()
            && *value == warning
        {
            source.end = new_source.end;
            return Ok(());
        }

        self.warnings.push(Span {
            value: warning,
            source: new_source,
        });

        Ok(())
    }

    /// Yields a warning if the delimiter is not needed due to a line break.
    ///
    /// Advances one token and then skips an optional line break. If a line break was encountered,
    /// it adds the warning with the source of the token of [LookaheadBuffer::peek]
    /// before [LookaheadBuffer::skip_lb] was called.
    #[inline]
    fn opt_omit_unnecessary_delimiter_warning(&mut self, warning: Warning) -> Result<(), Error> {
        // We need to capture this source outside,
        // or else the borrow checker will get mad.
        let source = self.iter.peek()?.source.clone();

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
    fn parse_opt_const_parameters(
        &mut self,
    ) -> Result<ConstParameters<'source, A>, Error> {
        Ok(match self.iter.peek()?.value {
            Token::Symbol(Symbol::LeftAngle) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;
                self.parse_const_parameters()?
            }
            _ => Vec::new_in(self.alloc),
        })
    }

    /// Parses type declarations like this:
    ///
    /// ```text
    /// fn<A, B>
    ///    ^
    /// ```
    ///
    /// Expects the next token to be the marked. Ends on the non-lb token after `>`.
    fn parse_const_parameters(&mut self) -> Result<ConstParameters<'source, A>, Error> {
        let mut params = Vec::new_in(self.alloc);

        loop {
            match self.iter.peek()?.value {
                Token::Identifier(id) => {
                    params.push(Span {
                        value: ConstParameter::Type {
                            id,
                            trait_bounds: Vec::new_in(self.alloc),
                        },
                        source: self.iter.peek()?.source.clone(),
                    }); // TODO: Add traits
                }
                Token::Symbol(Symbol::RightAngle) => break,
                _ => return error!("Expected type parameter"),
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            // TODO: Add lf for tp separation
            match self.iter.peek()?.value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {
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
    ) -> Result<Span<Vec<Span<StatementOrExpression<'source, A>>, A>>, Error> {
        let start = self.iter.peek()?.source.start;

        self.iter.advance()?;
        self.iter.skip_lb()?;

        let mut items = Vec::new_in(self.alloc);

        loop {
            match self.iter.peek()?.value {
                Token::Symbol(Symbol::RightBrace) => break,
                Token::Symbol(Symbol::Semicolon) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                    continue;
                }
                _ => {}
            }

            if let Some(statement) = self.try_parse_statement()? {
                items.push(statement.map(|s| StatementOrExpression::Statement(s)));
            } else {
                items.push(
                    self.parse_expression(0)?
                        .map(|e| StatementOrExpression::Expression(e)),
                );
            }

            match self.iter.peek()?.value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) => break,
                _ => return error!("Expected ';', '}' or a line break"),
            }
        }

        Ok(Span {
            value: items,
            source: start..self.iter.peek()?.source.end,
        })
    }

    /// Expects that the first token of the type is accessible via `peek()`. Ends on the token after the type.
    fn parse_type(&mut self) -> Result<Span<Type<'source, A>>, Error> {
        let source = self.iter.peek()?.source.clone();

        Ok(match self.iter.peek()?.value {
            Token::Symbol(Symbol::ExclamationMark) => {
                self.iter.advance()?;

                Span {
                    value: Type::Never,
                    source,
                }
            }
            Token::Identifier(id) => {
                let first = self.parse_item_path(id)?;

                let const_parameters = if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek_non_lb()?.0.value
                {
                    self.iter.skip_lb()?;

                    let mut tps = Span {
                        value: Vec::new_in(self.alloc),
                        source: self.iter.peek()?.source.start..0,
                    };

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    loop {
                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightAngle) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;
                            }
                            _ => {}
                        }

                        tps.value.push(self.parse_type()?);

                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightAngle) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            Token::LineBreak => self.iter.advance()?,
                            _ => return error!("Expected ',', '}' or a line break"),
                        }
                    }

                    self.iter.advance()?;

                    tps
                } else {
                    Span {
                        value: Vec::new_in(self.alloc),
                        source: first.source.end..first.source.end,
                    }
                };

                let first = RawType::Item(ItemRef { path: first, const_parameters });

                let remaining: Vec<RawType<'source, A>, A> =
                    if let Token::Symbol(Symbol::Pipe) = self.iter.peek_non_lb()?.0.value {
                        return error!("TODO: this ain't be implemented :(");

                        // self.iter.advance()?;
                        // self.iter.advance_skip_lb()?;
                    } else {
                        Vec::new_in(self.alloc)
                    };

                Span {
                    source: source.start
                        ..remaining
                            .last()
                            .map_or(first.source_span().end, |last| last.source_span().end),
                    value: Type::Union { first, remaining },
                }
            }
            _ => return error!("Expected an identifier or '!' (the never type)"),
        })
    }

    fn parse_use_child(&mut self) -> Result<Span<UseChild<'source, A>>, Error> {
        self.iter.skip_lb()?;
        self.iter.advance()?;
        self.iter.skip_lb()?;

        Ok(match self.iter.peek()?.value {
            Token::Symbol(Symbol::Star) => {
                let source = self.iter.peek()?.source.clone();
                self.iter.advance()?;

                Span {
                    value: UseChild::All,
                    source,
                }
            }
            Token::Symbol(Symbol::LeftParenthesis) => {
                let mut source = self.iter.peek()?.source.clone();

                let mut vec = Vec::new_in(self.alloc);

                loop {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let value = match self.iter.peek()?.value {
                        Token::Identifier(id) => self.parse_use(id)?,
                        Token::Symbol(Symbol::Comma) => {
                            self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                            continue;
                        }
                        Token::Symbol(Symbol::RightParenthesis) => {
                            source.end = self.iter.peek()?.source.end;
                            self.iter.advance()?;
                            break;
                        }
                        _ => return error!("Expected an identifier or ')'"),
                    };

                    vec.push(value);

                    match self.iter.peek_non_lb()? {
                        (
                            Span {
                                value: Token::Symbol(Symbol::Comma),
                                ..
                            },
                            _,
                        ) => {
                            self.iter.skip_lb()?;
                            self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                        }
                        (
                            Span {
                                value: Token::Symbol(Symbol::RightParenthesis),
                                ..
                            },
                            _,
                        ) => {
                            self.iter.skip_lb()?;
                            source.end = self.iter.peek()?.source.end;
                            self.iter.advance()?;
                            break;
                        }
                        (_, true) => self.iter.advance()?,
                        _ => return error!("Expected ',', ')' or a line break"),
                    }
                }

                Span {
                    value: UseChild::Multiple(vec),
                    source,
                }
            }
            Token::Identifier(id) => self
                .parse_use(id)?
                .map(|u| UseChild::Single(Box::new_in(u, self.alloc))),
            _ => return error!("Expected an identifier, '*' or '('"),
        })
    }

    /// Expects the . to not be consumed. Ends on the token after the use-statement
    fn parse_use(&mut self, id: &'source str) -> Result<Span<Use<'source, A>>, Error> {
        let source = self.iter.peek()?.source.clone();

        self.iter.advance()?;

        Ok(match self.iter.peek_non_lb()? {
            (
                Span {
                    value: Token::Symbol(Symbol::Dot),
                    ..
                },
                _,
            ) => {
                let child = self.parse_use_child()?;

                Span {
                    source: source.start..child.source.end,
                    value: Use {
                        id,
                        child: Some(child),
                    },
                }
            }
            _ => Span {
                value: Use { id, child: None },
                source,
            },
        })
    }

    /// Expects [Buffered::peek] to yield [Token::Identifier].
    /// Ends on the token after the last path segment (greedy).
    fn parse_item_path(
        &mut self,
        mut first_id: &'source str,
    ) -> Result<Span<ItemPath<'source, A>>, Error> {
        let mut source = self.iter.peek()?.source.clone();

        self.iter.advance()?;

        let parents = if let Token::Symbol(Symbol::Dot) = self.iter.peek_non_lb()?.0.value {
            let mut parents = Vec::new_in(self.alloc);

            loop {
                parents.push(first_id);

                self.iter.advance()?;
                self.iter.skip_lb()?;

                first_id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                source.end = self.iter.peek()?.source.end;

                self.iter.advance()?;

                match self.iter.peek_non_lb()?.0.value {
                    Token::Symbol(Symbol::Dot) => {}
                    _ => break,
                }
            }

            parents
        } else {
            Vec::new_in(self.alloc)
        };

        Ok(Span {
            value: ItemPath {
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
        parameters: &mut Vec<Parameter<'source, A>, A>,
    ) -> Result<(), Error> {
        loop {
            let is_mutable = match self.iter.peek()?.value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Keyword(Keyword::Mut) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;
                    true
                }
                _ => false,
            };

            let id = match self.iter.peek()?.value {
                Token::Identifier(id) => id,
                _ => return error!("Expected an identifier"),
            };

            self.iter.advance()?;
            self.iter.skip_lb()?;

            match self.iter.peek()?.value {
                Token::Symbol(Symbol::Colon) => {}
                _ => return error!("Expected ':'"),
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let ty = self.parse_type()?;

            parameters.push(Parameter { id, is_mutable, ty });

            let lb = self.iter.skip_lb()?;

            match self.iter.peek()?.value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Symbol(Symbol::Comma) => self.iter.advance()?,
                _ if lb => {}
                _ => return error!("Expected ',', ')' or a line break"),
            }
        }

        Ok(())
    }

    /// Tries to parse a statement. If nothing matches, `None` will be returned.
    ///
    /// # Tokens
    ///
    /// - Expects `peek()` to correspond to the first non-lb token of the statement (pre-advance).
    /// - Ends on the token after the statement. The caller must validate that token.
    /// **This may be a [Token::LineBreak]!**
    fn try_parse_statement(&mut self) -> Result<Option<Span<Statement<'source, A>>>, Error> {
        let mut source = self.iter.peek()?.source.clone();

        let mut doc_comments = Vec::new_in(self.alloc);

        // Collect all doc comments.
        loop {
            match self.iter.peek()?.value {
                Token::DocComment(doc_comment) => doc_comments.push(doc_comment),
                _ => break,
            }
            self.iter.advance()?;
        }

        let mut annotations = Vec::new_in(self.alloc);

        // Collect all available annotations associated with the statement.
        loop {
            match self.iter.peek()?.value {
                Token::Symbol(Symbol::At) => {}
                _ => {
                    self.iter.skip_lb()?;
                    break;
                }
            }

            self.iter.advance()?;
            self.iter.skip_lb()?;

            let id = match self.iter.peek()?.value {
                Token::Identifier(id) => id,
                _ => return error!("Expected an identifier"),
            };

            let path = self.parse_item_path(id)?;

            annotations.push(Annotation {
                path,
                arguments: Vec::new_in(self.alloc),
            });
        }

        let statement_kind: Option<StatementKind<'source, A>> = match self.iter.peek()?.value {
            Token::Keyword(Keyword::Fn) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let const_parameters = self.parse_opt_const_parameters()?;

                let id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                self.iter.advance()?;
                self.iter.skip_lb()?;

                match self.iter.peek()?.value {
                    Token::Symbol(Symbol::LeftParenthesis) => {}
                    Token::Symbol(Symbol::LeftAngle) => {
                        return error!(
                            "Type parameter are declared after 'fn', not after the function name."
                        )
                    }
                    _ => return error!("Expected '('"),
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let mut parameters = Vec::new_in(self.alloc);

                let this_parameter = match self.iter.peek()?.value {
                    Token::Keyword(Keyword::This) => {
                        self.iter.advance()?;
                        let lb = self.iter.skip_lb()?;

                        match self.iter.peek()?.value {
                            _ if lb => {}
                            Token::Symbol(Symbol::Comma) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            _ => return error!("Expected ',' or a line break"),
                        }

                        Some(ThisParameter::This)
                    }
                    Token::Keyword(Keyword::Mut) => {
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        match self.iter.peek()?.value {
                            // Case: `mut this`
                            Token::Keyword(Keyword::This) => {
                                self.iter.advance()?;
                                let lb = self.iter.skip_lb()?;

                                match self.iter.peek()?.value {
                                    _ if lb => {}
                                    Token::Symbol(Symbol::RightParenthesis) => {}
                                    Token::Symbol(Symbol::Comma) => {
                                        self.opt_omit_unnecessary_delimiter_warning(
                                            Warning::UnnecessaryComma,
                                        )?;
                                    }
                                    _ => return error!("Expected ',', ')' or a line break"),
                                }

                                Some(ThisParameter::ThisMut)
                            }

                            // In this case we parse the first parameter ourselves.
                            Token::Identifier(id) => {
                                self.iter.advance()?;
                                self.iter.skip_lb()?;

                                match self.iter.peek()?.value {
                                    Token::Symbol(Symbol::Colon) => {}
                                    _ => return error!("Expected ':'"),
                                }

                                self.iter.advance()?;
                                self.iter.skip_lb()?;

                                let ty = self.parse_type()?;

                                parameters.push(Parameter {
                                    id,
                                    is_mutable: true,
                                    ty,
                                });

                                let lb = self.iter.skip_lb()?;

                                match self.iter.peek()?.value {
                                    _ if lb => {}
                                    Token::Symbol(Symbol::RightParenthesis) => {}
                                    Token::Symbol(Symbol::Comma) => {
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

                let return_type: Option<Span<Type<'source, A>>> =
                    if let Token::Symbol(Symbol::MinusRightAngle) = self.iter.peek()?.value {
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let ty = self.parse_type()?;
                        self.iter.skip_lb()?;

                        Some(ty)
                    } else {
                        // No return type found (note: this is different from the empty type)
                        None
                    };

                // Validate that the block starts with `{`
                match self.iter.peek()?.value {
                    Token::Symbol(Symbol::LeftBrace) => {}
                    _ => return error!("Expected '{'"),
                }

                let body = self.parse_block()?;
                source.end = body.source.end;

                self.iter.advance()?;

                Some(StatementKind::Function {
                    signature: FunctionSignature {
                        const_parameters,
                        parameters,
                        return_type,
                    },
                    id,
                    this_parameter,
                    body: Box::new_in(body.map(Expression::Block), self.alloc),
                })
            }
            Token::Keyword(Keyword::Mod) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                self.iter.advance()?;

                let content: Option<_> = match self.iter.peek_non_lb()? {
                    // Code: mod xyz { ... }
                    (Span { value: Token::Symbol(Symbol::LeftBrace), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let content = self.parse_module_content()?;

                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightBrace) => {}
                            _ => return error!("Expected '}'"),
                        }

                        source.end = self.iter.peek()?.source.end;

                        self.iter.advance()?;

                        Some(content)
                    }
                    (Span { value: Token::EndOfInput | Token::Symbol(Symbol::RightBrace), .. }, _) => {
                        self.iter.skip_lb()?;
                        None
                    }
                    (Span { value: Token::Symbol(Symbol::Semicolon), .. }, _) => None, // The caller handles this
                    (_, true) => None,
                    _ => return error!("Expected module content (starting with '{') or a delimiter (';' or a line break)"),
                };

                Some(StatementKind::Module {
                    id,
                    content,
                    doc_comments,
                })
            }
            Token::Keyword(Keyword::Struct) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let const_parameters = self.parse_opt_const_parameters()?;

                let id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                let mut fields = Vec::new_in(self.alloc);

                self.iter.advance()?;

                match self.iter.peek_non_lb()? {
                    (Span { value: Token::Symbol(Symbol::LeftParenthesis), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        loop {
                            let start = self.iter.peek()?.source.start;

                            let is_public = match self.iter.peek()?.value {
                                Token::Keyword(Keyword::Pub) => {
                                    self.iter.advance()?;
                                    self.iter.skip_lb()?;
                                    true
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ => false
                            };

                            let is_mutable = match self.iter.peek()?.value {
                                Token::Keyword(Keyword::Mut) => {
                                    self.iter.advance()?;
                                    self.iter.skip_lb()?;
                                    true
                                }
                                _ => false
                            };

                            let id = match self.iter.peek()?.value {
                                Token::Identifier(id) => id,
                                _ => return error!("Expected an identifier"),
                            };

                            self.iter.advance()?;
                            self.iter.skip_lb()?;

                            let ty = match self.iter.peek()?.value {
                                Token::Symbol(Symbol::Colon) => {
                                    self.iter.advance()?;
                                    self.iter.skip_lb()?;
                                    self.parse_type()?
                                }
                                _ => return error!("Expected ':'"),
                            };

                            fields.push(Span {
                                source: start..ty.source.end,
                                value: StructField {
                                    is_public,
                                    is_mutable,
                                    id,
                                    ty,
                                },
                            });

                            let lb = self.iter.skip_lb()?;

                            match self.iter.peek()?.value {
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                Token::Symbol(Symbol::Comma) => {
                                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                }
                                _ if lb => {}
                                _ => return error!("Expected ',', ')' or a line break")
                            }
                        }

                        source.end = self.iter.peek()?.source.end;

                        self.iter.advance()?;
                    }
                    (_, true) => {}
                    _ => return error!("Expected struct content (starting with '(') or a delimiter (';' or a line break)"),
                }

                Some(StatementKind::Struct {
                    id,
                    const_parameters,
                    fields,
                    doc_comments,
                })
            }

            // Schema (brackets denote optionals, angles denote other constructs):
            //
            // let [mut] <variable_name>[: <type>] [= <expr>]
            Token::Keyword(Keyword::Let) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;

                let is_mutable = match self.iter.peek()?.value {
                    Token::Keyword(Keyword::Mut) => {
                        self.iter.advance()?;
                        self.iter.skip_lb()?;
                        true
                    }
                    _ => false,
                };

                let id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                // Source end is at least the integer end
                source.end = self.iter.peek()?.source.end;

                self.iter.advance()?;

                let ty = match self.iter.peek_non_lb()? {
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
                        let ty = self.parse_type()?;

                        source.end = ty.source.end; // Adjust end of statement
                        Some(ty)
                    }
                    _ => None,
                };

                let value = match self.iter.peek_non_lb()? {
                    // If the next non-line-break token is '=', then an expression is parsed.
                    (Span { value: Token::Symbol(Symbol::Equals), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        let expr = self.parse_expression(0)?;
                        source.end = expr.source.end; // Adjust end of statement
                        Some(expr)
                    }

                    // If it is not a '=' and there is a line break between this token and the previous,
                    // then this token does not belong to this statement and there is no value.
                    (_, true) => None,

                    // Else (there is a token which is not separated by a line break), short-circuit.
                    _ => return error!("Expected an initialization (starting with '=') or a delimiter (';' or a line break)"),
                };

                Some(StatementKind::Declaration {
                    doc_comments,
                    is_mutable,
                    ty,
                    id,
                    value: value.map(|v| Box::new_in(v, self.alloc)),
                })
            }

            // use a
            // use b.c
            // use d.*
            // use d.(x, y, z.*)
            Token::Keyword(Keyword::Use) => {
                if doc_comments.len() > 0 {
                    // TODO: Maybe change this from being an error (?)
                    return error!("Doc comments cannot be attached to use statements");
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

                let root_id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return error!("Expected an identifier"),
                };

                let Span { source: src, value } = self.parse_use(root_id)?;
                source = src;
                Some(StatementKind::Use(value))
            }
            Token::Keyword(Keyword::Break) => {
                self.iter.advance()?;
                self.iter.skip_lb()?;
                Some(StatementKind::Break)
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
            value: Statement {
                annotations,
                statement_kind,
            },
            source,
        }))
    }

    /// Parses an expression.
    ///
    /// Expects `peek()` to be the first token of the expression. May end on anything.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<Expression<'source, A>>, Error> {
        let first_term = self.parse_expression_first_term()?;
        self.parse_expression_remaining_terms(first_term, min_bp)
    }

    fn parse_expression_first_term(&mut self) -> Result<Span<Expression<'source, A>>, Error> {
        let mut first_source = self.iter.peek()?.source.clone();

        Ok(Span {
            value: match self.iter.peek()?.value {
                Token::String(s) => {
                    let s = s.process(self.alloc)?;
                    self.iter.advance()?;
                    Expression::String(s.into())
                }
                Token::Number(n) => {
                    self.iter.advance()?;
                    Expression::Number(n)
                }
                Token::Identifier(id) => {
                    self.iter.advance()?;
                    Expression::Identifier(id)
                }
                Token::Symbol(Symbol::LeftParenthesis) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let mut fields = Vec::new_in(self.alloc);

                    loop {
                        let is_mutable = match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightParenthesis) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                                self.iter.advance()?;
                                self.iter.skip_lb()?;
                                continue;
                            }
                            Token::Keyword(Keyword::Mut) => {
                                self.iter.advance()?;
                                self.iter.skip_lb()?;
                                true
                            }
                            _ => false,
                        };

                        let id = match self.iter.peek()?.value {
                            Token::Identifier(id) => id,
                            _ => return error!("Expected an identifier"),
                        };

                        let id_source = self.iter.peek()?.source.clone();

                        self.iter.advance()?;

                        let ty = match self.iter.peek_non_lb()? {
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

                        let init = match self.iter.peek_non_lb()? {
                            (Span { value: Token::Symbol(Symbol::Equals), .. }, _) => {
                                self.iter.skip_lb()?;
                                self.iter.advance()?;
                                self.iter.skip_lb()?;
                                self.parse_expression(0)?
                            }
                            (Span { value: Token::Symbol(Symbol::RightParenthesis | Symbol::Comma), .. }, _) | (_, true) => Span {
                                value: Expression::Identifier(id),
                                source: id_source,
                            },
                            _ => return error!("Expected an initialization (starting with '='), a type (starting with ':'), ')' or a delimiter (';' or a line break)"),
                        };

                        fields.push(InstanceFieldInit {
                            is_mutable,
                            id,
                            ty,
                            init,
                        });

                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::Comma) => {
                                let source = self.iter.peek()?.source.clone();

                                self.iter.advance()?;

                                // Capture ",\n" and ",)" groups
                                if self.iter.skip_lb()?
                                    || match self.iter.peek()?.value {
                                        Token::Symbol(Symbol::RightParenthesis) => true,
                                        _ => false,
                                    }
                                {
                                    self.warnings.push(Span {
                                        value: Warning::UnnecessaryComma,
                                        source,
                                    })
                                }
                            }
                            Token::LineBreak => self.iter.advance()?,
                            _ => {}
                        }
                    }

                    // peek() == ')'

                    first_source.end = self.iter.peek()?.source.end;
                    self.iter.advance()?;

                    Expression::Instance(fields)
                }
                Token::Symbol(Symbol::LeftBracket) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let mut items = Vec::new_in(self.alloc);

                    loop {
                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightBracket) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessaryComma)?;
                                self.iter.advance()?;
                                self.iter.skip_lb()?;
                                continue;
                            }
                            _ => {}
                        }

                        items.push(self.parse_expression(0)?);

                        match self.iter.peek()?.value {
                            Token::LineBreak => self.iter.advance()?,
                            Token::Symbol(Symbol::Comma) => {
                                self.opt_omit_unnecessary_delimiter_warning(
                                    Warning::UnnecessaryComma,
                                )?;
                            }
                            _ => {}
                        }
                    }

                    self.iter.advance()?;

                    Expression::Array(items)
                }
                Token::Symbol(Symbol::LeftBrace) => {
                    let block = self.parse_block()?;
                    self.iter.advance()?;
                    first_source.end = block.source.end;
                    Expression::Block(block.value)
                }
                Token::Keyword(Keyword::Fn) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let const_parameters = self.parse_opt_const_parameters()?;

                    let mut parameters = Vec::new_in(self.alloc);

                    match self.iter.peek()?.value {
                        Token::Symbol(Symbol::LeftParenthesis) => {}
                        _ => return error!("Expected '('."),
                    };

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    self.parse_fn_parameters(&mut parameters)?;

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let (return_type, body) = match self.iter.peek()?.value {
                        Token::Symbol(Symbol::MinusRightAngle) => {
                            return error!("Closures cannot have return type annotations.")
                        }
                        _ => (None, self.parse_expression(0)?),
                    };

                    Expression::Function {
                        signature: FunctionSignature {
                            const_parameters,
                            return_type,
                            parameters,
                        },
                        body: Box::new_in(body, self.alloc),
                    }
                }
                Token::Keyword(Keyword::If) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek()?.value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return error!("Expected '{'"),
                    }

                    let body = self.parse_block()?;

                    let mut else_ifs = Vec::new_in(self.alloc);

                    let else_body = loop {
                        self.iter.advance()?;

                        match self.iter.peek_non_lb()?.0.value {
                            Token::Keyword(Keyword::Else) => {}
                            _ => break None,
                        }

                        self.iter.advance()?;
                        self.iter.skip_lb()?;

                        match self.iter.peek()?.value {
                            Token::Keyword(Keyword::If) => {}
                            Token::Symbol(Symbol::LeftBrace) => {
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

                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::LeftBrace) => {}
                            _ => return error!("Expected '{' after 'else if' condition"),
                        }

                        let body = self.parse_block()?;

                        else_ifs.push(If {
                            condition: Box::new_in(condition, self.alloc),
                            body,
                        })
                    };

                    Expression::If {
                        base: If {
                            condition: Box::new_in(condition, self.alloc),
                            body,
                        },
                        else_ifs,
                        else_body,
                    }
                }
                Token::Keyword(Keyword::True) => {
                    self.iter.advance()?;
                    Expression::True
                }
                Token::Keyword(Keyword::False) => {
                    self.iter.advance()?;
                    Expression::False
                }
                Token::Keyword(Keyword::While) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek()?.value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return error!("Expected '{'"),
                    }

                    let body = self.parse_block()?;
                    first_source.end = body.source.end;
                    self.iter.advance()?;

                    Expression::While {
                        condition: Box::new_in(condition, self.alloc),
                        body,
                    }
                }
                Token::MarkupStartTag(tag_name) => {
                    let target_source = self.iter.peek()?.source.clone();
                    let mut params = Vec::new_in(self.alloc);

                    loop {
                        self.iter.advance()?;

                        let key = Span {
                            value: match self.iter.peek()?.value {
                                Token::MarkupClose => break,
                                Token::MarkupKey(key) => key,
                                _ => todo!("markup children"),
                            },
                            source: self.iter.peek()?.source.clone(),
                        };

                        self.iter.advance()?;

                        let value = match self.iter.peek()?.value {
                            Token::String(str) => Span {
                                value: Expression::String(str.process(self.alloc)?.into()),
                                source: self.iter.peek()?.source.clone(),
                            },
                            Token::Symbol(Symbol::LeftBrace) => {
                                self.iter.advance()?;
                                self.iter.skip_lb()?;
                                self.parse_expression(0)?
                            }
                            _ => unreachable!(),
                        };

                        params.push((key, value));
                    }

                    self.iter.advance()?;

                    Expression::Call {
                        target: Box::new_in(
                            Span {
                                value: Expression::Identifier(tag_name),
                                source: target_source,
                            },
                            self.alloc,
                        ),
                        arguments: CallArguments::Named(params),
                    }
                }
                _ => return error!("Invalid start of expression"),
            },
            source: first_source,
        })
    }

    /// Parses call arguments. Expects `peek()` to be '(' or a line break followed by a '('.
    /// Ends on the token after ')'.
    fn parse_call_arguments(&mut self) -> Result<CallArguments<'source, A>, Error> {
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
            let mut arguments = Vec::new_in(self.alloc);

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

                match self.iter.peek()?.value {
                    Token::LineBreak => self.iter.advance()?,
                    Token::Symbol(Symbol::Comma) => {
                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                    }
                    Token::Symbol(Symbol::RightParenthesis) => {}
                    _ => return error!("Expected ')', ',' or a line break"),
                }
            }

            CallArguments::Named(arguments)
        } else if let Token::Symbol(Symbol::RightParenthesis) = self.iter.peek()?.value {
            // There are no arguments.
            CallArguments::Named(Vec::new_in(self.alloc))
        } else {
            // Single, unnamed argument.

            let expr = self.parse_expression(bp::COMMA_AND_SEMICOLON)?;

            self.iter.skip_lb()?; // A line break here has no semantic meaning.

            match self.iter.peek()?.value {
                Token::Symbol(Symbol::RightParenthesis) => {}
                _ => return error!("Expected ')'. If you want to call with multiple parameters, you have to name them like `a = ?, b = ?`...")
            };

            CallArguments::Single(Box::new_in(expr, self.alloc))
        };

        self.iter.advance()?; // Skip ')'

        Ok(arguments)
    }

    fn parse_expression_remaining_terms(
        &mut self,
        mut first_term: Span<Expression<'source, A>>,
        min_bp: u8,
    ) -> Result<Span<Expression<'source, A>>, Error> {
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
                    Expression::Operation {
                        left: Box::new_in(first_term, self.alloc),
                        operation: $op,
                        right: Box::new_in(right, self.alloc),
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
                        value: Expression::Access(access),
                        source,
                    } => Span {
                        value: AssignmentTarget::Access(access),
                        source,
                    },
                    Span {
                        value: Expression::Identifier(id),
                        source,
                    } => Span {
                        value: AssignmentTarget::Identifier(id),
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
                    Expression::Assignment {
                        target: Box::new_in(at, self.alloc),
                        operation: $op,
                        value: Box::new_in(right, self.alloc),
                    },
                )
            }};
        }

        loop {
            let (token, _line_break) = self.iter.peek_non_lb()?;

            let (end, value) = match token.value {
                // Potential assignment operations
                Token::Symbol(Symbol::Plus) => {
                    op!(Operation::PA(PAOperation::Addition), bp::ADDITIVE)
                }
                Token::Symbol(Symbol::Minus) => {
                    op!(Operation::PA(PAOperation::Subtraction), bp::ADDITIVE)
                }
                Token::Symbol(Symbol::Star) => op!(
                    Operation::PA(PAOperation::Multiplication),
                    bp::MULTIPLICATIVE
                ),
                Token::Symbol(Symbol::Slash) => {
                    op!(Operation::PA(PAOperation::Division), bp::MULTIPLICATIVE)
                }
                Token::Symbol(Symbol::Percent) => {
                    op!(Operation::PA(PAOperation::Remainder), bp::MULTIPLICATIVE)
                }
                Token::Symbol(Symbol::StarStar) => {
                    op!(Operation::PA(PAOperation::Exponentiation), bp::EXPONENTIAL)
                }
                Token::Symbol(Symbol::Pipe) => {
                    op!(Operation::PA(PAOperation::BitwiseOr), bp::BITWISE_OR)
                }
                Token::Symbol(Symbol::Ampersand) => {
                    op!(Operation::PA(PAOperation::BitwiseAnd), bp::BITWISE_AND)
                }
                Token::Symbol(Symbol::Caret) => op!(
                    Operation::PA(PAOperation::BitwiseExclusiveOr),
                    bp::BITWISE_XOR
                ),
                Token::Symbol(Symbol::PipePipe) => {
                    op!(Operation::PA(PAOperation::LogicalOr), bp::LOGICAL_OR)
                }
                Token::Symbol(Symbol::AmpersandAmpersand) => {
                    op!(Operation::PA(PAOperation::LogicalAnd), bp::LOGICAL_AND)
                }
                Token::Symbol(Symbol::LeftAngleLeftAngle) => {
                    op!(Operation::PA(PAOperation::ShiftLeft), bp::SHIFT)
                }
                Token::Symbol(Symbol::RightAngleRightAngle) => {
                    op!(Operation::PA(PAOperation::ShiftRight), bp::SHIFT)
                }

                // Assignment operations
                Token::Symbol(Symbol::PlusEquals) => as_op!(Some(PAOperation::Addition)),
                Token::Symbol(Symbol::MinusEquals) => as_op!(Some(PAOperation::Subtraction)),
                Token::Symbol(Symbol::StarEquals) => as_op!(Some(PAOperation::Multiplication)),
                Token::Symbol(Symbol::SlashEquals) => as_op!(Some(PAOperation::Division)),
                Token::Symbol(Symbol::PercentEquals) => as_op!(Some(PAOperation::Remainder)),
                Token::Symbol(Symbol::StarStarEquals) => as_op!(Some(PAOperation::Exponentiation)),
                Token::Symbol(Symbol::PipeEquals) => as_op!(Some(PAOperation::BitwiseOr)),
                Token::Symbol(Symbol::AmpersandEquals) => as_op!(Some(PAOperation::BitwiseAnd)),
                Token::Symbol(Symbol::CaretEquals) => as_op!(Some(PAOperation::BitwiseExclusiveOr)),
                Token::Symbol(Symbol::PipePipeEquals) => as_op!(Some(PAOperation::LogicalOr)),
                Token::Symbol(Symbol::AmpersandAmpersandEquals) => {
                    as_op!(Some(PAOperation::LogicalAnd))
                }
                Token::Symbol(Symbol::LeftAngleLeftAngleEquals) => {
                    as_op!(Some(PAOperation::ShiftLeft))
                }
                Token::Symbol(Symbol::RightAngleRightAngleEquals) => {
                    as_op!(Some(PAOperation::ShiftRight))
                }

                // Comparative operations
                Token::Symbol(Symbol::EqualsEquals) => {
                    op!(Operation::Comp(ComparativeOperation::Equals), bp::EQUALITY)
                }
                Token::Symbol(Symbol::LeftAngle) => op!(
                    Operation::Comp(ComparativeOperation::LessThan),
                    bp::EQUALITY
                ),
                Token::Symbol(Symbol::RightAngle) => op!(
                    Operation::Comp(ComparativeOperation::GreaterThan),
                    bp::EQUALITY
                ),
                Token::Symbol(Symbol::ExclamationMarkEquals) => op!(
                    Operation::Comp(ComparativeOperation::NotEquals),
                    bp::EQUALITY
                ),
                Token::Symbol(Symbol::LeftAngleEquals) => op!(
                    Operation::Comp(ComparativeOperation::LessThanOrEqual),
                    bp::EQUALITY
                ),
                Token::Symbol(Symbol::RightAngleEquals) => op!(
                    Operation::Comp(ComparativeOperation::GreaterThanOrEqual),
                    bp::EQUALITY
                ),

                // Other
                Token::Symbol(Symbol::LeftParenthesis) => {
                    if bp::CALL < min_bp {
                        break;
                    }

                    let args = self.parse_call_arguments()?;
                    let end = self.iter.peek()?.source.end;

                    self.iter.advance()?;

                    (
                        end,
                        Expression::Call {
                            target: Box::new_in(first_term, self.alloc),
                            arguments: args,
                        },
                    )
                }
                Token::Symbol(Symbol::Dot) => {
                    if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek_n_non_lb(1)?.0.value {
                        let target = match first_term {
                            Span {
                                value: Expression::Access(a),
                                source,
                            } => Span {
                                value: ConstParametersCallTarget::Access(a),
                                source,
                            },
                            Span {
                                value: Expression::OptionalAccess(a),
                                source,
                            } => Span {
                                value: ConstParametersCallTarget::OptionalAccess(a),
                                source,
                            },
                            Span {
                                value: Expression::Identifier(i),
                                source,
                            } => Span {
                                value: ConstParametersCallTarget::Identifier(i),
                                source,
                            },
                            _ => return error!("Cannot call an expression with const parameters."),
                        };

                        self.iter.skip_lb()?;
                        self.iter.advance()?; // Skip '.'
                        self.iter.skip_lb()?;
                        self.iter.advance()?; // Skip '<'
                        self.iter.skip_lb()?;

                        let const_arguments = Vec::new_in(self.alloc);
                        
                        // TODO: implement this
                        
                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::RightAngle) => {}
                            _ => todo!()
                        }
                        
                        self.iter.advance()?;
                        self.iter.skip_lb()?;
                        
                        match self.iter.peek()?.value {
                            Token::Symbol(Symbol::LeftParenthesis) => {}
                            _ => return error!("Expected '(' (call arguments)"),
                        }
                        
                        let arguments = self.parse_call_arguments()?;
                        let end = self.iter.peek()?.source.end;

                        (
                            end,
                            Expression::CallWithConstParameters {
                                target: Box::new_in(target, self.alloc),
                                arguments,
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

                        let property = match self.iter.peek()?.value {
                            Token::Identifier(id) => id,
                            _ => return error!("Expected an identifier"),
                        };

                        let end = self.iter.peek()?.source.end;

                        self.iter.advance()?;

                        (
                            end,
                            Expression::Access(Access {
                                target: Box::new_in(first_term, self.alloc),
                                property,
                            }),
                        )
                    }
                }
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
                        Expression::OptionalAccess(Access {
                            target: Box::new_in(first_term, self.alloc),
                            property,
                        }),
                    )
                }
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
    fn parse_module_content(&mut self) -> Result<ModuleContent<'source, A>, Error> {
        let mut items = Vec::new_in(self.alloc);

        loop {
            let is_public = match self.iter.peek()?.value {
                // Ignore semicolons
                Token::Symbol(Symbol::Semicolon) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon)?;

                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    continue;
                }
                Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => break,
                Token::Keyword(Keyword::Pub) => {
                    self.iter.advance()?;
                    self.iter.skip_lb()?;

                    true
                }
                _ => false,
            };

            items.push(TopLevelItem {
                is_public,
                statement: if let Some(statement) = self.try_parse_statement()? {
                    statement
                } else {
                    return error!("Expected a statement");
                },
            });

            match self.iter.peek()?.value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) | Token::EndOfInput => break,
                _ => return error!("Expected ';', '}' or a line break"),
            }
        }

        Ok(ModuleContent(items))
    }

    fn parse_module(&mut self) -> Result<ModuleContent<'source, A>, Error> {
        let content = self.parse_module_content()?;
        match self.iter.peek()?.value {
            Token::EndOfInput => Ok(content),
            _ => error!("This '}' does not close anything; consider removing it"),
        }
    }
}

/// Parses a source module commonly obtained from file content.
pub fn parse_module<LexA: Allocator + Copy, ParseA: Allocator + Copy>(
    source: &[u8],
    lex_alloc: LexA,
    parse_alloc: ParseA,
) -> Result<(ModuleContent<ParseA>, Vec<Span<Warning>, ParseA>), (Error, Index)> {
    let buf = LookaheadBuffer::new(Lexer::new(source, lex_alloc));

    let mut context = ParseContext::new(buf, parse_alloc);
    let maybe_module = context.parse_module();
    let ParseContext { iter, warnings, .. } = context;

    maybe_module
        .map(|module| (module, warnings))
        .map_err(|err| (err, iter.iter().cursor().index()))
}
