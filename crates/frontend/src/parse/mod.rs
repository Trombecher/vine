mod errors;
mod warnings;
mod tests;

pub mod bp;
pub mod ast;

use crate::{Box, Vec};
use crate::buffered::Buffered;
use crate::lex::{Keyword, Lexer, Symbol, Token, TokenIterator};
use ast::*;
use bytes::{Index, Span};
use core::fmt::Debug;
use core::ops::Range;
use bumpalo::Bump;
pub use errors::*;
pub use warnings::*;

pub struct ParseContext<'source: 'alloc, 'alloc, T: TokenIterator<'source>> {
    pub iter: Buffered<'source, T>,
    pub alloc: &'alloc Bump,
    pub warnings: Vec<'alloc, Span<Warning>>,
}

impl<'source: 'alloc, 'alloc, T: TokenIterator<'source>> ParseContext<'source, 'alloc, T> {
    #[inline]
    pub const fn new(iter: Buffered<'source, T>, alloc: &'alloc Bump) -> ParseContext<'source, 'alloc, T> {
        Self {
            warnings: Vec::new_in(alloc),
            iter,
            alloc,
        }
    }

    /// Adds a warning with the span of the token returned by [Buffered::peek].
    #[inline]
    fn omit_single_token_warning(&mut self, warning: Warning) {
        let new_source = self.iter.peek().source.clone();

        // If there is a last added warning that is equal to the new warning and has is extendable, extend it!
        if warning.is_extendable() && let Some(Span { value, source }) = self.warnings.last_mut() && *value == warning {
            source.end = new_source.end;
            return;
        }

        self.warnings.push(Span {
            value: warning,
            source: new_source,
        })
    }

    /// Yields a warning if the delimiter is not needed due to a line break.
    ///
    /// Uses [Buffered::advance_skip_lb] to advance the iterator while skipping a [Token::LineBreak].
    /// If a line break was encountered, it adds the warning with the source of the token of [Buffered::peek]
    /// before [Buffered::advance_skip_lb] was called.
    #[inline]
    fn opt_omit_unnecessary_delimiter_warning(&mut self, warning: Warning) -> Result<(), Error> {
        // We need to capture this source outside,
        // or else the borrow checker will get mad.
        let source = self.iter.peek().source.clone();

        if self.iter.advance_skip_lb()? {
            self.warnings.push(Span {
                value: warning,
                source,
            })
        }
        Ok(())
    }

    /// If `peek()` is a '<', then it parses type parameters.
    #[inline]
    fn parse_opt_type_parameter_declarations(&mut self) -> Result<TypeParameters<'source, 'alloc>, Error> {
        Ok(match self.iter.peek().value {
            Token::Symbol(Symbol::LeftAngle) => {
                self.iter.advance_skip_lb()?;
                self.parse_type_parameter_declarations()?
            }
            _ => Vec::new_in(self.alloc)
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
    fn parse_type_parameter_declarations(&mut self) -> Result<TypeParameters<'source, 'alloc>, Error> {
        let mut params = Vec::new_in(self.alloc);

        loop {
            match self.iter.peek().value {
                Token::Identifier(id) => {
                    params.push(Span {
                        value: TypeParameter {
                            id,
                            traits: Vec::new_in(self.alloc),
                        },
                        source: self.iter.peek().source.clone(),
                    }); // TODO: Add traits
                }
                Token::Symbol(Symbol::RightAngle) => break,
                _ => return Err(Error::ExpectedTypeParameter)
            }

            self.iter.advance_skip_lb()?;

            // TODO: Add lf for tp separation
            match self.iter.peek().value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {
                    self.iter.advance_skip_lb()?;
                }
                _ => return Err(Error::ExpectedTypeParameterDelimiter)
            }
        }

        self.iter.advance_skip_lb()?;

        Ok(params)
    }

    /// Parses a block.
    ///
    /// Expects `peek()` to be [Symbol::LeftBrace]. Ends on [Symbol::RightBrace].
    fn parse_block(&mut self) -> Result<Span<Vec<'alloc, Span<StatementOrExpression<'source, 'alloc>>>>, Error> {
        let start = self.iter.peek().source.start;

        self.iter.advance_skip_lb()?;

        let mut items = Vec::new_in(self.alloc);

        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::RightBrace) => break,
                Token::Symbol(Symbol::Semicolon) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon);
                    self.iter.advance_skip_lb()?;
                }
                _ => {}
            }

            if let Some(statement) = self.try_parse_statement()? {
                items.push(statement.map(|s| StatementOrExpression::Statement(s)));
            } else {
                items.push(self.parse_expression(0)?.map(|e| StatementOrExpression::Expression(e)));
            }

            match self.iter.peek().value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) => break,
                _ => return Err(Error::UnimplementedError)
            }
        }

        Ok(Span {
            value: items,
            source: start..self.iter.peek().source.end,
        })
    }

    /// Expects that the first token of the type is accessible via `peek()`. Ends on the token after the type.
    fn parse_type(&mut self) -> Result<Span<Type<'source, 'alloc>>, Error> {
        let source = self.iter.peek().source.clone();

        Ok(match self.iter.peek().value {
            Token::Symbol(Symbol::ExclamationMark) => {
                self.iter.advance()?;

                Span {
                    value: Type::Never,
                    source,
                }
            }
            Token::Identifier(id) => {
                let first = self.parse_item_path(id)?;

                let tps = if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek_non_lb()?.0.value {
                    self.iter.skip_lb()?;

                    let mut tps = Span {
                        value: Vec::new_in(self.alloc),
                        source: self.iter.peek().source.start..0,
                    };

                    self.iter.advance_skip_lb()?;

                    loop {
                        match self.iter.peek().value {
                            Token::Symbol(Symbol::RightAngle) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessarySemicolon)
                            }
                            _ => {}
                        }

                        tps.value.push(self.parse_type()?);

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::RightAngle) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                            }
                            Token::LineBreak => self.iter.advance()?,
                            _ => return Err(Error::UnimplementedError),
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

                let first = RawType::Item(ItemRef {
                    path: first,
                    tps,
                });

                let remaining: Vec<'alloc, RawType<'source, 'alloc>> = if let Token::Symbol(Symbol::Pipe) = self.iter.peek_non_lb()?.0.value {
                    return Err(Error::Unimplemented)

                    // self.iter.advance()?;
                    // self.iter.advance_skip_lb()?;
                } else {
                    Vec::new_in(self.alloc)
                };

                Span {
                    source: source.start..remaining.last().map_or(first.source_span().end, |last| last.source_span().end),
                    value: Type::Union {
                        first,
                        remaining,
                    },
                }
            }
            _ => return Err(Error::UnimplementedError)
        })
    }

    fn parse_use_child(&mut self) -> Result<Span<UseChild<'source, 'alloc>>, Error> {
        self.iter.skip_lb()?;
        self.iter.advance_skip_lb()?;

        Ok(match self.iter.peek().value {
            Token::Symbol(Symbol::Star) => {
                let source = self.iter.peek().source.clone();
                self.iter.advance()?;

                Span {
                    value: UseChild::All,
                    source,
                }
            }
            Token::Symbol(Symbol::LeftParenthesis) => {
                let mut source = self.iter.peek().source.clone();

                let mut vec = Vec::new_in(self.alloc);

                loop {
                    self.iter.advance_skip_lb()?;

                    let value = match self.iter.peek().value {
                        Token::Identifier(id) => self.parse_use(id)?,
                        Token::Symbol(Symbol::Comma) => {
                            self.omit_single_token_warning(Warning::UnnecessaryComma);
                            continue;
                        }
                        Token::Symbol(Symbol::RightParenthesis) => {
                            source.end = self.iter.peek().source.end;
                            self.iter.advance()?;
                            break;
                        }
                        _ => return Err(Error::UnimplementedError)
                    };

                    vec.push(value);

                    match self.iter.peek_non_lb()? {
                        (Span { value: Token::Symbol(Symbol::Comma), .. }, _) => {
                            self.iter.skip_lb()?; // Consume ','
                            self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                        }
                        (Span { value: Token::Symbol(Symbol::RightParenthesis), .. }, _) => {
                            self.iter.skip_lb()?; // Consume ','
                            source.end = self.iter.peek().source.end;
                            self.iter.advance()?;
                            break;
                        }
                        (_, true) => self.iter.advance()?,
                        _ => return Err(Error::UnimplementedError)
                    }
                }

                Span {
                    value: UseChild::Multiple(vec),
                    source,
                }
            }
            Token::Identifier(id) => self.parse_use(id)?
                .map(|u| UseChild::Single(Box::new_in(u, self.alloc))),
            _ => return Err(Error::UnimplementedError)
        })
    }

    /// Expects the . to not be consumed. Ends on the token after the use-statement
    fn parse_use(&mut self, id: &'source str) -> Result<Span<Use<'source, 'alloc>>, Error> {
        let source = self.iter.peek().source.clone();

        self.iter.advance()?;

        Ok(match self.iter.peek_non_lb()? {
            (Span { value: Token::Symbol(Symbol::Dot), .. }, _) => {
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
                value: Use {
                    id,
                    child: None,
                },
                source,
            }
        })
    }

    /// Expects [Buffered::peek] to yield [Token::Identifier].
    /// Ends on the token after the last path segment (greedy).
    fn parse_item_path(&mut self, mut first_id: &'source str) -> Result<Span<ItemPath<'source, 'alloc>>, Error> {
        let mut source = self.iter.peek().source.clone();

        self.iter.advance()?;

        let parents = if let Token::Symbol(Symbol::Dot) = self.iter.peek_non_lb()?.0.value {
            let mut parents = Vec::new_in(self.alloc);

            loop {
                parents.push(first_id);

                self.iter.advance_skip_lb()?;

                first_id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::UnimplementedError)
                };

                source.end = self.iter.peek().source.end;

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

    /// Expects peek() to be the token after `(`. Ends on `)`.
    fn parse_fn_parameters(&mut self) -> Result<Vec<'alloc, Parameter<'source, 'alloc>>, Error> {
        let mut parameters: Vec<'alloc, Parameter<'source, 'alloc>> = Vec::new_in(self.alloc);

        loop {
            let is_mutable = match self.iter.peek().value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Keyword(Keyword::Mut) => {
                    self.iter.advance_skip_lb()?;
                    true
                }
                _ => false,
            };

            let id = match self.iter.peek().value {
                Token::Identifier(id) => id,
                _ => return Err(Error::UnimplementedError)
            };

            self.iter.advance_skip_lb()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::Colon) => {}
                _ => return Err(Error::UnimplementedError)
            }

            self.iter.advance_skip_lb()?;

            let ty = self.parse_type()?;

            parameters.push(Parameter { id, is_mutable, ty });

            let lb = self.iter.skip_lb()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Symbol(Symbol::Comma) => self.iter.advance()?,
                _ if lb => {}
                _ => return Err(Error::UnimplementedError)
            }
        }

        Ok(parameters)
    }

    /// Tries to parse a statement. If nothing matches, `None` will be returned.
    ///
    /// # Tokens
    ///
    /// - Expects `peek()` to correspond to the first non-lb token of the statement (pre-advance).
    /// - Ends on the token after the statement. The caller must validate that token.
    /// **This may be a [Token::LineBreak]!**
    fn try_parse_statement(&mut self) -> Result<Option<Span<Statement<'source, 'alloc>>>, Error> {
        let mut source = self.iter.peek().source.clone();

        let mut doc_comments = Vec::new_in(self.alloc);

        // Against code repetition:
        macro_rules! error_doc_comments {
            () => {{
                if doc_comments.len() > 0 {
                    return Err(Error::UnboundDocComment)
                }
            }};
        }

        // Collect all doc comments.
        loop {
            match self.iter.peek().value {
                Token::DocComment(doc_comment) => doc_comments.push(doc_comment),
                _ => break,
            }
            self.iter.advance()?;
        }

        let mut annotations = Vec::new_in(self.alloc);

        // Collect all available annotations associated with the statement.
        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::At) => {}
                _ => {
                    self.iter.skip_lb()?;
                    break;
                }
            }

            self.iter.advance_skip_lb()?;

            let id = match self.iter.peek().value {
                Token::Identifier(id) => id,
                _ => return Err(Error::UnimplementedError)
            };

            let path = self.parse_item_path(id)?;

            annotations.push(Annotation {
                path,
                arguments: Vec::new_in(self.alloc), // TODO: arguments of annotations
            });
        }

        let statement_kind: Option<StatementKind<'source, 'alloc>> = match self.iter.peek().value {
            Token::Keyword(Keyword::Fn) => {
                self.iter.advance_skip_lb()?;

                let tps = self.parse_opt_type_parameter_declarations()?;

                let mut id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::UnimplementedError)
                };

                let mut fn_target = None;

                if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek().value {
                    fn_target = Some(id);

                    self.iter.skip_lb()?;

                    id = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => return Err(Error::UnimplementedError)
                    };
                }

                self.iter.advance_skip_lb()?;

                match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftParenthesis) => {}
                    Token::Symbol(Symbol::Dot) => {
                        return Err(Error::Unimplemented)
                    }
                    _ => return Err(Error::UnimplementedError)
                }

                self.iter.advance_skip_lb()?;

                let parameters = self.parse_fn_parameters()?;

                self.iter.advance_skip_lb()?;

                let return_type: Option<Span<Type<'source, 'alloc>>> = if let Token::Symbol(Symbol::MinusRightAngle) = self.iter.peek().value {
                    self.iter.advance_skip_lb()?;

                    let ty = self.parse_type()?;
                    self.iter.skip_lb()?;

                    Some(ty)
                } else {
                    // No return type found (note: this is different from the empty type)
                    None
                };

                // Validate that the block starts with `{`
                match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftBrace) => {}
                    _ => return Err(Error::UnimplementedError),
                }

                let body = self.parse_block()?;
                source.end = body.source.end;

                self.iter.advance()?;

                Some(StatementKind::Declaration {
                    doc_comments,
                    is_mutable: false,
                    ty: None,
                    id,
                    value: Some(Box::new_in(Span {
                        value: Expression::Function {
                            signature: FunctionSignature {
                                return_type,
                                parameters,
                                has_this_parameter: false,
                                tps,
                            },
                            body: Box::new_in(body.map(Expression::Block), self.alloc),
                        },
                        source: source.clone(),
                    }, self.alloc)),
                })
            }
            Token::Keyword(Keyword::Mod) => {
                self.iter.advance_skip_lb()?;

                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::ExpectedIdentifierOfModule)
                };

                self.iter.advance()?;

                let content: Option<_> = match self.iter.peek_non_lb()? {
                    // Code: mod xyz { ... }
                    (Span { value: Token::Symbol(Symbol::LeftBrace), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance_skip_lb()?;

                        let content = self.parse_module_content()?;

                        // Validate that the module has ended on `}`:
                        match self.iter.peek().value {
                            Token::Symbol(Symbol::RightBrace) => {}
                            _ => return Err(Error::UnimplementedError)
                        }

                        source.end = self.iter.peek().source.end;

                        self.iter.advance()?;

                        Some(content)
                    }
                    (Span { value: Token::EndOfInput | Token::Symbol(Symbol::RightBrace), .. }, _) => {
                        self.iter.skip_lb()?;
                        None
                    }
                    (Span { value: Token::Symbol(Symbol::Semicolon), .. }, _) => None, // The caller handles this
                    (_, true) => None,
                    _ => return Err(Error::UnimplementedError)
                };

                Some(StatementKind::Module {
                    id,
                    content,
                    doc_comments,
                })
            }
            Token::Keyword(Keyword::Struct) => {
                self.iter.advance_skip_lb()?;

                let tps = self.parse_opt_type_parameter_declarations()?;

                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::UnimplementedError)
                };

                let mut fields = Vec::new_in(self.alloc);

                self.iter.advance()?;

                match self.iter.peek_non_lb()? {
                    (Span { value: Token::Symbol(Symbol::LeftParenthesis), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance_skip_lb()?;

                        loop {
                            let start = self.iter.peek().source.start;

                            let is_public = match self.iter.peek().value {
                                Token::Keyword(Keyword::Pub) => {
                                    self.iter.advance_skip_lb()?;
                                    true
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ => false
                            };

                            let is_mutable = match self.iter.peek().value {
                                Token::Keyword(Keyword::Mut) => {
                                    self.iter.advance_skip_lb()?;
                                    true
                                }
                                _ => false
                            };

                            let id = match self.iter.peek().value {
                                Token::Identifier(id) => id,
                                _ => return Err(Error::UnimplementedError)
                            };

                            self.iter.advance_skip_lb()?;

                            let ty = match self.iter.peek().value {
                                Token::Symbol(Symbol::Colon) => {
                                    self.iter.advance_skip_lb()?;
                                    self.parse_type()?
                                }
                                _ => return Err(Error::UnimplementedError)
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

                            match self.iter.peek().value {
                                Token::Symbol(Symbol::Comma) => {
                                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ if lb => {}
                                _ => return Err(Error::UnimplementedError)
                            }
                        }

                        source.end = self.iter.peek().source.end;

                        self.iter.advance()?;
                    }
                    (_, true) => {}
                    _ => return Err(Error::UnimplementedError)
                }

                Some(StatementKind::Struct {
                    id,
                    tps,
                    fields,
                    doc_comments,
                })
            }

            // Schema (brackets denote optionals, 'allocngles denote other constructs):
            //
            // let [mut] <variable_name>[: <type>] [= <expr>]
            Token::Keyword(Keyword::Let) => {
                self.iter.advance_skip_lb()?;

                let is_mutable = match self.iter.peek().value {
                    Token::Keyword(Keyword::Mut) => {
                        self.iter.advance_skip_lb()?;
                        true
                    }
                    _ => false,
                };

                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::UnimplementedError),
                };

                // Source end is at least the integer end
                source.end = self.iter.peek().source.end;

                self.iter.advance()?;

                let ty = match self.iter.peek_non_lb()? {
                    (Span { value: Token::Symbol(Symbol::Colon), .. }, _) => {
                        self.iter.skip_lb()?;
                        self.iter.advance_skip_lb()?;
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
                        self.iter.advance_skip_lb()?;

                        let expr = self.parse_expression(0)?;
                        source.end = expr.source.end; // Adjust end of statement
                        Some(expr)
                    }

                    // If it is not a '=' and there is a line break between this token and the previous,
                    // then this token does not belong to this statement and there is no value.
                    (_, true) => None,

                    // Else (there is a token which is not separated by a line break), short-circuit.
                    _ => return Err(Error::UnimplementedError)
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
                    return Err(Error::DocCommentOnUse);
                }

                self.iter.advance_skip_lb()?;

                let root_id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::UnimplementedError),
                };

                let Span { source: src, value } = self.parse_use(root_id)?;
                source = src;
                Some(StatementKind::Use(value))
            }

            // for<A, B> { ... }
            Token::Keyword(Keyword::For) => {
                error_doc_comments!();

                if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek_after()?.value {
                    self.iter.advance()?;
                    self.iter.advance_skip_lb()?;

                    let tps = self.parse_type_parameter_declarations()?;

                    self.iter.advance_skip_lb()?;

                    let content = self.parse_module_content()?;

                    match self.iter.peek().value {
                        Token::Symbol(Symbol::RightBrace) => {}
                        _ => return Err(Error::UnimplementedError),
                    }

                    self.iter.advance()?;

                    Some(StatementKind::TypeParameterAlias {
                        tps,
                        content,
                    })
                } else {
                    error_doc_comments!();

                    if annotations.len() > 0 {
                        return Err(Error::UnboundAnnotations);
                    }

                    None
                }
            }
            _ => {
                error_doc_comments!();

                if annotations.len() > 0 {
                    return Err(Error::UnboundAnnotations);
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
    /// Expects `peek()` to be the first token of the expression.
    /// Ends on the following tokens:
    ///
    /// - [Token::EndOfInput], [Symbol::RightBrace], [Symbol::Semicolon],
    /// [Symbol::Comma], [Symbol::RightParenthesis] or [Symbol::RightBracket].
    /// In this case, 'alloc line break before may have been skipped.
    /// - [Token::LineBreak]. In this case the token after the line break
    /// was already generated and may be anything (did not continue the expression).
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<Expression<'source, 'alloc>>, Error> {
        let first_term = self.parse_expression_first_term()?;
        self.parse_expression_remaining_terms(first_term, min_bp)
    }

    fn parse_expression_first_term(&mut self) -> Result<Span<Expression<'source, 'alloc>>, Error> {
        let mut first_source = self.iter.peek().source.clone();

        Ok(Span {
            value: match self.iter.peek().value {
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
                    self.iter.advance_skip_lb()?;

                    let mut fields = Vec::new_in(self.alloc);

                    loop {
                        let is_mutable = match self.iter.peek().value {
                            Token::Symbol(Symbol::RightParenthesis) => break,
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessaryComma);
                                self.iter.advance_skip_lb()?;
                                continue;
                            }
                            Token::Keyword(Keyword::Mut) => {
                                self.iter.advance_skip_lb()?;
                                true
                            }
                            _ => false,
                        };

                        let id = match self.iter.peek().value {
                            Token::Identifier(id) => id,
                            _ => return Err(Error::UnimplementedError)
                        };

                        let id_source = self.iter.peek().source.clone();

                        self.iter.advance()?;

                        let ty = match self.iter.peek_non_lb()? {
                            (Span { value: Token::Symbol(Symbol::Colon), .. }, _) => {
                                self.iter.skip_lb()?;
                                self.iter.advance_skip_lb()?;

                                Some(self.parse_type()?)
                            }
                            _ => None,
                        };

                        let init = match self.iter.peek_non_lb()? {
                            (Span { value: Token::Symbol(Symbol::Equals), .. }, _) => {
                                self.iter.skip_lb()?;
                                self.iter.advance_skip_lb()?;
                                self.parse_expression(0)?
                            }
                            (Span { value: Token::Symbol(Symbol::RightParenthesis | Symbol::Comma), .. }, _) | (_, true) => Span {
                                value: Expression::Identifier(id),
                                source: id_source,
                            },
                            _ => return Err(Error::UnimplementedError)
                        };

                        fields.push(InstanceFieldInit {
                            is_mutable,
                            id,
                            ty,
                            init,
                        });

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::Comma) => {
                                let source = self.iter.peek().source.clone();

                                // Capture ",\n" and ",)" groups
                                if self.iter.advance_skip_lb()? || match self.iter.peek().value {
                                    Token::Symbol(Symbol::RightParenthesis) => true,
                                    _ => false
                                } {
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

                    first_source.end = self.iter.peek().source.end;
                    self.iter.advance()?;

                    Expression::Instance(fields)
                }
                Token::Symbol(Symbol::LeftBrace) => {
                    let block = self.parse_block()?;
                    self.iter.advance()?;
                    first_source.end = block.source.end;
                    Expression::Block(block.value)
                }
                Token::Keyword(Keyword::Fn) => {
                    self.iter.advance_skip_lb()?;

                    let tps = self.parse_opt_type_parameter_declarations()?;

                    let parameters = match self.iter.peek().value {
                        Token::Symbol(Symbol::LeftParenthesis) => {
                            self.iter.advance_skip_lb()?;
                            self.parse_fn_parameters()?
                        }
                        _ => return Err(Error::UnimplementedError)
                    };

                    self.iter.advance_skip_lb()?;

                    let (return_type, body) = match self.iter.peek().value {
                        Token::Symbol(Symbol::MinusRightAngle) => return Err(Error::Unimplemented),
                        _ => (None, self.parse_expression(0)?),
                    };

                    Expression::Function {
                        signature: FunctionSignature {
                            return_type,
                            parameters,
                            has_this_parameter: false,
                            tps,
                        },
                        body: Box::new_in(body, self.alloc),
                    }
                }
                Token::Keyword(Keyword::If) => {
                    self.iter.advance_skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek().value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return Err(Error::UnimplementedError)
                    }

                    let body = self.parse_block()?;

                    let mut else_ifs = Vec::new_in(self.alloc);

                    let else_body = loop {
                        self.iter.advance()?;

                        match self.iter.peek_non_lb()?.0.value {
                            Token::Keyword(Keyword::Else) => {}
                            _ => break None,
                        }

                        self.iter.advance_skip_lb()?;

                        match self.iter.peek().value {
                            Token::Keyword(Keyword::If) => {}
                            Token::Symbol(Symbol::LeftBrace) => {
                                let block = self.parse_block()?;
                                self.iter.advance()?;
                                break Some(block);
                            }
                            _ => return Err(Error::UnimplementedError)
                        }

                        // else-if-branch

                        self.iter.advance_skip_lb()?;

                        let condition = self.parse_expression(0)?;
                        self.iter.skip_lb()?;

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::LeftBrace) => {}
                            _ => return Err(Error::UnimplementedError)
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
                    self.iter.advance_skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek().value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return Err(Error::UnimplementedError)
                    }

                    let body = self.parse_block()?;
                    first_source.end = body.source.end;
                    self.iter.advance()?;

                    Expression::While {
                        condition: Box::new_in(condition, self.alloc),
                        body,
                    }
                }
                _ => return Err(Error::UnimplementedError)
            },
            source: first_source,
        })
    }

    fn parse_expression_remaining_terms(
        &mut self,
        mut first_term: Span<Expression<'source, 'alloc>>,
        min_bp: u8,
    ) -> Result<Span<Expression<'source, 'alloc>>, Error> {
        let start = first_term.source.start;

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.iter.skip_lb()?;
                self.iter.advance_skip_lb()?;

                let right = self.parse_expression($bp.1)?;

                (
                    right.source.end,
                    Expression::Operation {
                        left: Box::new_in(first_term, self.alloc),
                        operation: $op,
                        right: Box::new_in(right, self.alloc)
                    }
                )
            }};
        }

        loop {
            // At this point, 'alloc line break must not have been skipped.
            let (token, line_break) = self.iter.peek_non_lb()?;

            let (end, value) = match token.value {
                // Potential assignment operations
                Token::Symbol(Symbol::Plus) => op!(Operation::PA(PAOperation::Addition), bp::ADDITIVE),
                Token::Symbol(Symbol::Minus) => op!(Operation::PA(PAOperation::Subtraction), bp::ADDITIVE),
                Token::Symbol(Symbol::Star) => op!(Operation::PA(PAOperation::Multiplication), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Slash) => op!(Operation::PA(PAOperation::Division), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Percent) => op!(Operation::PA(PAOperation::Remainder), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::StarStar) => op!(Operation::PA(PAOperation::Exponentiation), bp::EXPONENTIAL),
                Token::Symbol(Symbol::Pipe) => op!(Operation::PA(PAOperation::BitwiseOr), bp::BITWISE_OR),
                Token::Symbol(Symbol::Ampersand) => op!(Operation::PA(PAOperation::BitwiseAnd), bp::BITWISE_AND),
                Token::Symbol(Symbol::Caret) => op!(Operation::PA(PAOperation::BitwiseExclusiveOr), bp::BITWISE_XOR),
                Token::Symbol(Symbol::PipePipe) => op!(Operation::PA(PAOperation::LogicalOr), bp::LOGICAL_OR),
                Token::Symbol(Symbol::AmpersandAmpersand) => op!(Operation::PA(PAOperation::LogicalAnd), bp::LOGICAL_AND),
                Token::Symbol(Symbol::LeftAngleLeftAngle) => op!(Operation::PA(PAOperation::ShiftLeft), bp::SHIFT),
                Token::Symbol(Symbol::RightAngleRightAngle) => op!(Operation::PA(PAOperation::ShiftRight), bp::SHIFT),

                // Comparative operations
                Token::Symbol(Symbol::EqualsEquals) => op!(Operation::Comp(ComparativeOperation::Equals), bp::EQUALITY),
                Token::Symbol(Symbol::LeftAngle) => op!(Operation::Comp(ComparativeOperation::LessThan), bp::EQUALITY),
                Token::Symbol(Symbol::RightAngle) => op!(Operation::Comp(ComparativeOperation::GreaterThan), bp::EQUALITY),
                Token::Symbol(Symbol::ExclamationMarkEquals) => op!(Operation::Comp(ComparativeOperation::NotEquals), bp::EQUALITY),
                Token::Symbol(Symbol::LeftAngleEquals) => op!(Operation::Comp(ComparativeOperation::LessThanOrEqual), bp::EQUALITY),
                Token::Symbol(Symbol::RightAngleEquals) => op!(Operation::Comp(ComparativeOperation::GreaterThanOrEqual), bp::EQUALITY),

                // Other
                Token::Symbol(Symbol::LeftParenthesis) => {
                    if bp::CALL < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance_skip_lb()?;

                    // Skip initial commas
                    loop {
                        match self.iter.peek().value {
                            Token::Symbol(Symbol::Comma) => {
                                self.omit_single_token_warning(Warning::UnnecessaryComma);
                                self.iter.advance_skip_lb()?;
                            }
                            _ => break
                        }
                    }

                    let arguments: CallArguments<'source, 'alloc> = if let Token::Symbol(Symbol::RightParenthesis) = self.iter.peek().value {
                        // There are no arguments. We must handle this special case,
                        // because we cannot parse an expression.
                        CallArguments::Unnamed(Vec::new_in(self.alloc))
                    } else {
                        let mut maybe_arg = self.parse_expression_first_term()?;

                        if let Span {
                            value: Expression::Identifier(mut arg),
                            ref mut source
                        } = maybe_arg && let Token::Symbol(Symbol::Equals) = self.iter.peek_non_lb()?.0.value {
                            // Named

                            self.iter.skip_lb()?;
                            self.iter.advance_skip_lb()?;

                            let mut args = Vec::new_in(self.alloc);

                            macro_rules! parse_that {
                                () => {{
                                    let expr = self.parse_expression(0)?;
    
                                    match self.iter.peek().value {
                                        Token::LineBreak => self.iter.advance()?,
                                        Token::Symbol(Symbol::Comma) => {
                                            self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                        }
                                        _ => {}
                                    }
        
                                    args.push((
                                        Span {
                                            value: arg,
                                            source: source.clone(),
                                        },
                                        expr
                                    ));
                                }};
                            }

                            parse_that!();

                            loop {
                                arg = match self.iter.peek().value {
                                    Token::Symbol(Symbol::Comma) => {
                                        self.omit_single_token_warning(Warning::UnnecessaryComma);
                                        self.iter.advance_skip_lb()?;
                                        continue;
                                    }
                                    Token::Symbol(Symbol::RightParenthesis) => break,
                                    Token::Identifier(id) => {
                                        *source = self.iter.peek().source.clone();

                                        self.iter.advance_skip_lb()?;

                                        match self.iter.peek().value {
                                            Token::Symbol(Symbol::Equals) => {}
                                            _ => return Err(Error::UnimplementedError)
                                        }

                                        id
                                    }
                                    _ => return Err(Error::UnimplementedError)
                                };

                                parse_that!();
                            }

                            CallArguments::Named(args)
                        } else {
                            // Unnamed

                            let mut args = Vec::new_in(self.alloc);

                            let expr = self.parse_expression_remaining_terms(maybe_arg, 0)?;

                            match self.iter.peek().value {
                                Token::LineBreak => self.iter.advance()?,
                                Token::Symbol(Symbol::Comma) => {
                                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                }
                                _ => {}
                            }

                            args.push(expr);

                            loop {
                                match self.iter.peek().value {
                                    Token::Symbol(Symbol::Comma) => {
                                        self.omit_single_token_warning(Warning::UnnecessaryComma);
                                        self.iter.advance_skip_lb()?;
                                        continue;
                                    }
                                    Token::Symbol(Symbol::RightParenthesis) => break,
                                    _ => {}
                                }

                                maybe_arg = self.parse_expression_first_term()?;
                                let expr = self.parse_expression_remaining_terms(maybe_arg, 0)?;

                                match self.iter.peek().value {
                                    Token::LineBreak => self.iter.advance()?,
                                    Token::Symbol(Symbol::Comma) => {
                                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                    }
                                    _ => {}
                                }

                                args.push(expr);
                            }

                            CallArguments::Unnamed(args)
                        }
                    };

                    let end = self.iter.peek().source.end;

                    self.iter.advance_skip_lb()?;

                    (
                        end,
                        Expression::Call {
                            target: Box::new_in(first_term, self.alloc),
                            arguments,
                        }
                    )
                }
                Token::Symbol(Symbol::Dot) => {
                    if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance_skip_lb()?;

                    let property = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => return Err(Error::UnimplementedError)
                    };

                    let end = self.iter.peek().source.end;

                    self.iter.advance()?;

                    (
                        end,
                        Expression::Access(Access {
                            target: Box::new_in(first_term, self.alloc),
                            property,
                        })
                    )
                }
                Token::Symbol(Symbol::QuestionMarkDot) => {
                    if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                        break;
                    }

                    self.iter.skip_lb()?;
                    self.iter.advance_skip_lb()?;

                    let property = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => return Err(Error::UnimplementedError)
                    };

                    let end = self.iter.peek().source.end;

                    self.iter.advance()?;

                    (
                        end,
                        Expression::OptionalAccess(Access {
                            target: Box::new_in(first_term, self.alloc),
                            property,
                        })
                    )
                }
                Token::EndOfInput
                | Token::Symbol(Symbol::LeftBrace) // Necessary for if-statements to work
                | Token::Symbol(Symbol::RightBrace)
                | Token::Symbol(Symbol::Semicolon)
                | Token::Symbol(Symbol::Comma)
                | Token::Symbol(Symbol::RightParenthesis)
                | Token::Symbol(Symbol::RightBracket) => {
                    // TODO: maybe bug below:
                    // self.iter.skip_lb()?;
                    break;
                }
                _ if line_break => break,
                _ => return Err(Error::UnimplementedError)
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
    fn parse_module_content(&mut self) -> Result<ModuleContent<'source, 'alloc>, Error> {
        let mut items = Vec::new_in(self.alloc);

        loop {
            let is_public = match self.iter.peek().value {
                // Ignore semicolons
                Token::Symbol(Symbol::Semicolon) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon);
                    self.iter.advance_skip_lb()?;
                    continue;
                }
                Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => break,
                Token::Keyword(Keyword::Pub) => {
                    self.iter.advance_skip_lb()?;
                    true
                }
                _ => false
            };

            items.push(TopLevelItem {
                is_public,
                statement: self.try_parse_statement()?.ok_or(Error::UnimplementedError)?,
            });

            match self.iter.peek().value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) | Token::EndOfInput => break,
                _ => return Err(Error::UnimplementedError)
            }
        }

        Ok(ModuleContent(items))
    }

    fn parse_module(&mut self) -> Result<ModuleContent<'source, 'alloc>, Error> {
        let content = self.parse_module_content()?;
        match self.iter.peek().value {
            Token::EndOfInput => Ok(content),
            _ => Err(Error::TrailingClosingBraceInTopLevelModule),
        }
    }
}

/// Parses a source module commonly obtained from file content.
pub fn parse_module<'source: 'lex_alloc + 'parse_alloc, 'lex_alloc: 'parse_alloc, 'parse_alloc>(
    source: &'source [u8],
    lex_alloc: &'lex_alloc Bump,
    parse_alloc: &'parse_alloc Bump,
) -> (
    Result<ModuleContent<'source, 'parse_alloc>, (Error, Range<Index>)>,
    Vec<'parse_alloc, Span<Warning>>
) {
    let mut lex = Lexer::new(source, lex_alloc);
    let buf = Buffered::new_init(match lex.next_token() {
        Ok(init) => init,
        Err(err) => return (
            Err((
                Error::from(err),
                lex.index() - 1..lex.index()
            )),
            Vec::new_in(parse_alloc)
        )
    }, lex);

    let mut context = ParseContext::new(buf, parse_alloc);
    let maybe_module = context.parse_module();

    let ParseContext { iter, warnings, .. } = context;

    (maybe_module.map_err(|err| (err, iter.peek().source.clone())), warnings)
}