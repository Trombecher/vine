#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]
#![feature(if_let_guard)]
#![feature(let_chains)]

use crate::ast::*;
pub use buffered::*;
use error::Error;
use lex::token::{Keyword, Symbol, Token, TokenIterator};
use lex::Span;
use warning::Warning;

mod buffered;
pub mod ast;
pub mod bp;

pub struct ParseContext<'a, T: TokenIterator<'a>> {
    pub iter: Buffered<'a, T>,
}

impl<'a, T: TokenIterator<'a>> ParseContext<'a, T> {
    #[inline]
    pub const fn new(iter: Buffered<'a, T>) -> ParseContext<'a, T> {
        Self { iter }
    }

    /// Adds a warning with the span of the token returned by [Buffered::peek].
    #[inline]
    fn omit_single_token_warning(&mut self, warning: Warning) {
        let source = self.iter.peek().source.clone();
        self.iter.warnings_mut().push(Span {
            value: warning,
            source,
        })
    }

    /// Uses [Buffered::advance_skip_lb] to advance the iterator while skipping a [Token::LineBreak].
    /// If a line break was encountered, it adds the warning with the source of the token of [Buffered::peek]
    /// before [Buffered::advance_skip_lb] was called.
    #[inline]
    fn opt_omit_unnecessary_delimiter_warning(&mut self, warning: Warning) -> Result<(), Error> {
        // We need to capture this source outside,
        // or else the borrow checker will get mad.
        let source = self.iter.peek().source.clone();

        if self.iter.advance_skip_lb()? {
            self.iter.warnings_mut().push(Span {
                value: warning,
                source,
            })
        }
        Ok(())
    }

    /// If `peek()` is a '<', then it parses type parameters.
    #[inline]
    pub fn parse_opt_type_parameter_declarations(&mut self) -> Result<TypeParameters<'a>, Error> {
        Ok(match self.iter.peek().value {
            Token::Symbol(Symbol::LeftAngle) => {
                self.iter.advance_skip_lb()?;
                self.parse_type_parameter_declarations()?
            }
            _ => Vec::new()
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
    pub fn parse_type_parameter_declarations(&mut self) -> Result<TypeParameters<'a>, Error> {
        let mut params = Vec::new();

        loop {
            match self.iter.peek().value {
                Token::Identifier(id) => {
                    params.push(Span {
                        value: TypeParameter {
                            id,
                            traits: vec![],
                        },
                        source: self.iter.peek().source.clone(),
                    }); // TODO: Add traits
                }
                Token::Symbol(Symbol::RightAngle) => break,
                _ => todo!()
            }

            self.iter.advance_skip_lb()?;

            // TODO: Add lf for tp separation
            match self.iter.peek().value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {
                    self.iter.advance_skip_lb()?;
                }
                _ => todo!()
            }
        }

        self.iter.advance_skip_lb()?;

        Ok(params)
    }

    /// Parses a block.
    ///
    /// Expects `peek()` to be [Symbol::LeftBrace]. Ends on [Symbol::RightBrace].
    fn parse_block(&mut self) -> Result<Span<Vec<Span<StatementOrExpression<'a>>>>, Error> {
        let start = self.iter.peek().source.start;

        self.iter.advance_skip_lb()?;

        let mut items = Vec::new();

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
                _ => todo!()
            }
        }

        Ok(Span {
            value: items,
            source: start..self.iter.peek().source.end,
        })
    }

    /// Expects that the first token of the type is accessible via `peek()`. Ends on the token after the type.
    pub(crate) fn parse_type(&mut self) -> Result<Span<Type<'a>>, Error> {
        let source = self.iter.peek().source.clone();

        Ok(match &self.iter.peek().value {
            Token::Keyword(kw) if let Ok(ty) = Type::try_from(*kw) => {
                self.iter.advance()?;
                Span {
                    value: ty,
                    source,
                }
            }
            Token::Identifier(id) => {
                self.parse_item_path(id)?.map(|path| {
                    Type::ItemPath {
                        path,
                        generics: vec![], // TODO: add generics
                    }
                })
            }
            token => todo!("{:?}", token)
        })
    }

    pub(crate) fn parse_use_child(&mut self) -> Result<Span<UseChild<'a>>, Error> {
        self.iter.skip_lb()?;
        self.iter.advance_skip_lb()?;

        Ok(match self.iter.peek().value {
            Token::Symbol(Symbol::Star) => {
                let source = self.iter.peek().source.clone();
                self.iter.advance()?;

                Span {
                    value: UseChild::All,
                    source
                }
            }
            Token::Symbol(Symbol::LeftParenthesis) => {
                let mut source = self.iter.peek().source.clone();

                let mut vec = Vec::new();

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
                        _ => todo!()
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
                        _ => todo!()
                    }
                }

                Span {
                    value: UseChild::Multiple(vec),
                    source
                }
            }
            Token::Identifier(id) => self.parse_use(id)?
                .map(|u| UseChild::Single(Box::new(u))),
            _ => todo!()
        })
    }

    /// Expects the . to not be consumed. Ends on the token after the use-statement
    pub(crate) fn parse_use(&mut self, id: &'a str) -> Result<Span<Use<'a>>, Error> {
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
            },
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
    pub(crate) fn parse_item_path(&mut self, mut first_id: &'a str) -> Result<Span<ItemPath<'a>>, Error> {
        let mut source = self.iter.peek().source.clone();

        self.iter.advance_skip_lb()?;

        // This match-statement hell prevents unnecessary allocation of an empty vec.
        let parents = match self.iter.peek().value {
            Token::Symbol(Symbol::Dot) => {
                let mut parents = Vec::new();

                loop {
                    parents.push(first_id);

                    self.iter.advance_skip_lb()?;

                    first_id = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => todo!()
                    };

                    source.end = self.iter.peek().source.end;

                    self.iter.advance()?;

                    match self.iter.peek_non_lb()?.0.value {
                        Token::Symbol(Symbol::Dot) => {}
                        _ => break,
                    }
                }

                parents
            }
            _ => Vec::new(),
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
    pub(crate) fn parse_fn_parameters(&mut self) -> Result<Vec<Parameter<'a>>, Error> {
        let mut parameters = Vec::<Parameter<'a>>::new();

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
                _ => todo!()
            };

            self.iter.advance_skip_lb()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::Colon) => {}
                _ => todo!()
            }

            self.iter.advance_skip_lb()?;

            let ty = self.parse_type()?;

            parameters.push(Parameter { id, is_mutable, ty });

            let lb = self.iter.skip_lb()?;

            match &self.iter.peek().value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Symbol(Symbol::Comma) => self.iter.advance()?,
                _ if lb => {}
                token => todo!("{:?}", token)
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
    pub(crate) fn try_parse_statement(&mut self) -> Result<Option<Span<Statement<'a>>>, Error> {
        let mut source = self.iter.peek().source.clone();

        let mut doc_comments = Vec::new();

        // Against code repetition:
        macro_rules! error_doc_comments {
            () => {{
                if doc_comments.len() > 0 {
                    todo!("Error: you can only add doc comments to items; last token: {:?}", self.iter.peek())
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

        let mut annotations = Vec::new();

        // Collect all available annotations associated with the statement.
        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::At) => {}
                _ => break
            }

            self.iter.advance_skip_lb()?;

            let id = match self.iter.peek().value {
                Token::Identifier(id) => id,
                _ => todo!()
            };

            let path = self.parse_item_path(id)?;

            annotations.push(Annotation {
                path,
                arguments: vec![], // TODO: arguments of annotations
            });
        }

        let statement_kind: Option<StatementKind<'a>> = match self.iter.peek().value {
            Token::Keyword(Keyword::Fn) => {
                self.iter.advance_skip_lb()?;

                let tps = self.parse_opt_type_parameter_declarations()?;

                let mut id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!()
                };

                let mut fn_target = None;

                if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek().value {
                    fn_target = Some(id);

                    self.iter.skip_lb()?;

                    id = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => todo!()
                    };
                }

                self.iter.advance_skip_lb()?;

                match &self.iter.peek().value {
                    Token::Symbol(Symbol::LeftParenthesis) => {}
                    Token::Symbol(Symbol::Dot) => {
                        todo!("On struct function decl")
                    }
                    token => todo!("invalid token: {:?}", token)
                }

                self.iter.advance_skip_lb()?;

                let parameters = self.parse_fn_parameters()?;

                self.iter.advance_skip_lb()?;

                let return_type: Option<Span<Type<'a>>> = if let Token::Symbol(Symbol::MinusRightAngle) = self.iter.peek().value {
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
                    _ => todo!(),
                }

                let body = self.parse_block()?;
                source.end = body.source.end;
                
                self.iter.advance()?;

                Some(StatementKind::Declaration {
                    doc_comments,
                    is_mutable: false,
                    ty: None,
                    id,
                    value: Some(Box::new(Span {
                        value: Expression::Function {
                            signature: FunctionSignature {
                                return_type,
                                parameters,
                                has_this_parameter: false,
                                tps,
                            },
                            body: Box::new(body.map(Expression::Block)),
                        },
                        source: source.clone(),
                    })),
                })
            }
            Token::Keyword(Keyword::Mod) => {
                self.iter.advance_skip_lb()?;

                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::E0071)
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
                            _ => todo!()
                        }

                        source.end = self.iter.peek().source.end;

                        self.iter.advance()?;

                        Some(content)
                    }
                    (Span { value: Token::EndOfInput | Token::Symbol(Symbol::RightBrace), .. }, _) => {
                        self.iter.skip_lb()?;
                        None
                    },
                    (Span { value: Token::Symbol(Symbol::Semicolon), .. }, _) => None, // The caller handles this
                    (_, true) => None,
                    token => todo!("{:?}", token)
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
                    _ => todo!()
                };

                let mut fields = Vec::new();

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

                            let id = match &self.iter.peek().value {
                                Token::Identifier(id) => *id,
                                token => todo!("{:?}", token)
                            };

                            self.iter.advance_skip_lb()?;

                            let ty = match self.iter.peek().value {
                                Token::Symbol(Symbol::Colon) => {
                                    self.iter.advance_skip_lb()?;
                                    self.parse_type()?
                                }
                                _ => todo!()
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

                            match &self.iter.peek().value {
                                Token::Symbol(Symbol::Comma) => {
                                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ if lb => {}
                                token => todo!("{:?}", token)
                            }
                        }

                        source.end = self.iter.peek().source.end;

                        self.iter.advance()?;
                    }
                    (_, true) => {}
                    _ => todo!()
                }

                Some(StatementKind::Struct {
                    id,
                    tps,
                    fields,
                    doc_comments,
                })
            }

            // Schema (brackets denote optionals, angles denote other constructs):
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
                    _ => todo!(),
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
                    _ => todo!()
                };

                Some(StatementKind::Declaration {
                    doc_comments,
                    is_mutable,
                    ty,
                    id,
                    value: value.map(Box::new),
                })
            }

            // use a
            // use b.c
            // use d.*
            // use d.(x, y, z.*)
            Token::Keyword(Keyword::Use) => {
                if doc_comments.len() > 0 {
                    todo!("Error: cannot add a doc comment to a use-statement")
                }

                self.iter.advance_skip_lb()?;

                let root_id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!(),
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
                        _ => todo!(),
                    }

                    self.iter.advance()?;
                    
                    Some(StatementKind::TypeParameterAlias {
                        tps,
                        content,
                    })
                } else {
                    error_doc_comments!();

                    if annotations.len() > 0 {
                        return Err(Error::E0041);
                    }

                    None
                }
            }
            _ => {
                error_doc_comments!();

                if annotations.len() > 0 {
                    return Err(Error::E0041);
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
    /// In this case, a line break before may have been skipped.
    /// - [Token::LineBreak]. In this case the token after the line break
    /// was already generated and may be anything (did not continue the expression).
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<Expression<'a>>, Error> {
        let first_term = self.parse_expression_first_term()?;
        self.parse_expression_remaining_terms(first_term, min_bp)
    }
    
    pub fn parse_expression_first_term(&mut self) -> Result<Span<Expression<'a>>, Error> {
        let mut first_source = self.iter.peek().source.clone();

        Ok(Span {
            value: match &self.iter.peek().value {
                Token::String(s) => {
                    let s = s.process()?;
                    self.iter.advance()?;
                    Expression::String(s)
                }
                Token::Number(n) => {
                    let n = *n;
                    self.iter.advance()?;
                    Expression::Number(n)
                }
                Token::Identifier(id) => {
                    let id = *id;
                    self.iter.advance()?;
                    Expression::Identifier(id)
                }
                Token::Symbol(Symbol::LeftParenthesis) => {
                    self.iter.advance_skip_lb()?;

                    let mut fields = Vec::new();

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
                            _ => todo!("expected id or mut")
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
                            _ => todo!("Error")
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
                                    self.iter.warnings_mut().push(Span {
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
                        _ => todo!("Expected `(`")
                    };

                    self.iter.advance_skip_lb()?;

                    let (return_type, body) = match self.iter.peek().value {
                        Token::Symbol(Symbol::MinusRightAngle) => todo!("Return type of function expression"),
                        _ => (None, self.parse_expression(0)?),
                    };

                    Expression::Function {
                        signature: FunctionSignature {
                            return_type,
                            parameters,
                            has_this_parameter: false,
                            tps,
                        },
                        body: Box::new(body),
                    }
                }
                Token::Keyword(Keyword::If) => {
                    self.iter.advance_skip_lb()?;

                    let condition = self.parse_expression(0)?;
                    self.iter.skip_lb()?;

                    match self.iter.peek().value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => todo!()
                    }

                    let body = self.parse_block()?;

                    let mut else_ifs = Vec::new();

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
                            _ => todo!()
                        }

                        // else-if-branch

                        self.iter.advance_skip_lb()?;

                        let condition = self.parse_expression(0)?;
                        self.iter.skip_lb()?;

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::LeftBrace) => {}
                            _ => todo!()
                        }

                        let body = self.parse_block()?;

                        else_ifs.push(If {
                            condition: Box::new(condition),
                            body,
                        })
                    };

                    Expression::If {
                        base: If {
                            condition: Box::new(condition),
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
                        _ => todo!()
                    }

                    let body = self.parse_block()?;
                    first_source.end = body.source.end;
                    self.iter.advance()?;

                    Expression::While {
                        condition: Box::new(condition),
                        body,
                    }
                }
                token => todo!("Unexpected token: {:?}", token)
            },
            source: first_source,
        })
    }

    pub fn parse_expression_remaining_terms(
        &mut self,
        mut first_term: Span<Expression<'a>>,
        min_bp: u8
    ) -> Result<Span<Expression<'a>>, Error> {
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
                        left: Box::new(first_term),
                        operation: $op,
                        right: Box::new(right)
                    }
                )
            }};
        }

        loop {
            // At this point, a line break must not have been skipped.
            let (token, line_break) = self.iter.peek_non_lb()?;

            let (end, value) = match &token.value {
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

                    let mut maybe_arg = self.parse_expression_first_term()?;
                    
                    let arguments: CallArguments<'a> = if let Span {
                        value: Expression::Identifier(mut arg),
                        mut source
                    } = maybe_arg && let Token::Symbol(Symbol::Equals) = self.iter.peek_non_lb()?.0.value {
                        // Named
                        
                        self.iter.skip_lb()?;
                        self.iter.advance_skip_lb()?;

                        let mut args = Vec::new();

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
                                        source,
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
                                    source = self.iter.peek().source.clone();
                                    
                                    self.iter.advance_skip_lb()?;

                                    match self.iter.peek().value {
                                        Token::Symbol(Symbol::Equals) => {}
                                        _ => todo!("error")
                                    }

                                    id
                                }
                                _ => todo!("error")
                            };

                            parse_that!();
                        }

                        CallArguments::Named(args)
                    } else {
                        // Unnamed

                        let mut args = Vec::new();

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
                    };

                    let end = self.iter.peek().source.end;

                    self.iter.advance_skip_lb()?;

                    (
                        end,
                        Expression::Call {
                            target: Box::new(first_term),
                            arguments,
                        }
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
                token => todo!("{:?}", token)
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
    pub fn parse_module_content(&mut self) -> Result<ModuleContent<'a>, Error> {
        let mut items = Vec::new();

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
                statement: self.try_parse_statement()?
                    .expect("This has to be a statement"), // TODO: turn this into an error
            });

            match self.iter.peek().value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                }
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) | Token::EndOfInput => break,
                _ => todo!()
            }
        }

        Ok(ModuleContent(items))
    }
}