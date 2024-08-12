#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]
#![feature(if_let_guard)]

use std::str::from_raw_parts;

pub use buffered::*;
use error::Error;
use lex::Span;
use lex::token::{Keyword, Symbol, Token, TokenIterator};
use warning::Warning;
use crate::ast::*;

mod buffered;
pub mod ast;
pub mod bp;

pub struct ParseContext<'a, T: TokenIterator<'a>> {
    pub iter: Buffered<'a, T>,
}

/// Merges two `str`s of the same memory region.
#[inline]
fn merge(a_ptr: *const u8, b: &str) -> &str {
    unsafe { from_raw_parts(a_ptr, b.as_ptr().sub_ptr(a_ptr) + b.len()) }
}

impl<'a, T: TokenIterator<'a>> ParseContext<'a, T> {
    #[inline]
    pub const fn new(iter: Buffered<'a, T>) -> ParseContext<'a, T> {
        Self { iter }
    }

    /// Adds a warning with the span of the token returned by [Buffered::peek].
    #[inline]
    fn omit_single_token_warning(&mut self, warning: Warning) {
        let source = self.iter.peek().source;
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
        let source = self.iter.peek().source;
        
        if self.iter.advance_skip_lb()? {
            self.iter.warnings_mut().push(Span {
                value: warning,
                source,
            })
        }
        Ok(())
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
                        source: self.iter.peek().source,
                    }); // TODO: Add traits
                },
                Token::Symbol(Symbol::RightAngle) => break,
                _ => todo!()
            }

            self.iter.advance_skip_lb()?;

            // TODO: Add lf for tp separation
            match self.iter.peek().value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {
                    self.iter.advance_skip_lb()?;
                },
                _ => todo!()
            }
        }
        
        self.iter.advance_skip_lb()?;

        Ok(params)
    }

    /// Parses a block.
    ///
    /// Expects `peek()` to be [Symbol::LeftBrace]. Ends on [Symbol::RightBrace].
    fn parse_block(&mut self) -> Result<Span<'a, Vec<Span<'a, StatementOrExpression<'a>>>>, Error> {
        let start = self.iter.peek().source.as_ptr();

        self.iter.advance_skip_lb()?;

        let mut items = Vec::new();

        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::RightBrace) => break,
                Token::Symbol(Symbol::Semicolon) => {
                    self.omit_single_token_warning(Warning::UnnecessarySemicolon);
                    self.iter.advance_skip_lb()?;
                }
                _ => {
                    if let Some(statement) = self.try_parse_statement()? {
                        items.push(statement.map(|s| StatementOrExpression::Statement(s)));
                    } else {
                        items.push(self.parse_expression(0)?.map(|e| StatementOrExpression::Expression(e)));

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::Semicolon) => {
                                self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                            }
                            Token::LineBreak => self.iter.advance()?,
                            Token::Symbol(Symbol::RightBrace) => break,
                            _ => todo!()
                        }
                    }
                }
            }
        }

        Ok(Span {
            value: items,
            source: merge(start, self.iter.peek().source),
        })
    }

    /// Expects that the first token of the type is accessible via `peek()`. Ends on the token after the type.
    pub(crate) fn parse_type(&mut self) -> Result<Span<'a, Type<'a>>, Error> {
        let start = self.iter.peek().source;

        Ok(match &self.iter.peek().value {
            Token::Keyword(kw) if let Ok(ty) = Type::try_from(*kw) => {
                self.iter.advance()?;
                Span {
                    value: ty,
                    source: start,
                }
            },
            Token::Identifier(id) => {
                self.parse_item_path(id)?.map(|path| {
                    Type::ItemPath {
                        path,
                        generics: vec![], // TODO: add generics
                    }
                })
            },
            token => todo!("{:?}", token)
        })
    }

    /// Expects the . to not be consumed.
    pub(crate) fn parse_use(&mut self, id: &'a str) -> Result<(Use<'a>, bool), Error> {
        // TODO: I don't believe in this code. I'm certain there are some bugs here.
        
        let mut lb = self.iter.advance_skip_lb()?;
        
        let child = match self.iter.peek().value {
            Token::Symbol(Symbol::Dot) => {
                self.iter.advance_skip_lb()?;

                Some(match self.iter.peek().value {
                    Token::Symbol(Symbol::Star) => {
                        lb = self.iter.advance_skip_lb()?;
                        UseChild::All
                    },
                    Token::Symbol(Symbol::LeftParenthesis) => {
                        self.iter.advance_skip_lb()?;
                        
                        let mut vec = Vec::new();

                        loop {
                            let (value, lb) = match self.iter.peek().value {
                                Token::Identifier(id) => self.parse_use(id)?,
                                Token::Symbol(Symbol::Comma) => {
                                    self.omit_single_token_warning(Warning::UnnecessaryComma);
                                    self.iter.advance_skip_lb()?;
                                    continue
                                },
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ => todo!()
                            };
                            
                            vec.push(value);

                            match self.iter.peek().value {
                                _ if lb => {}
                                Token::Symbol(Symbol::Comma) => {
                                    self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessaryComma)?;
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ => todo!()
                            }
                        }

                        lb = self.iter.advance_skip_lb()?;
                        
                        UseChild::Multiple(vec)
                    },
                    Token::Identifier(id) => {
                        let (u, p) = self.parse_use(id)?;
                        lb = p;
                        UseChild::Single(Box::new(u))
                    },
                    _ => todo!()
                })
            }
            _ => None
        };
        
        Ok((Use {
            id,
            child,
        }, lb))
    }
    
    /// Expects [Buffered::peek] to yield [Token::Identifier].
    /// Ends on the token after the last path segment (greedy).
    pub(crate) fn parse_item_path(&mut self, mut first_id: &'a str) -> Result<Span<'a, ItemPath<'a>>, Error> {
        let start = self.iter.peek().source;
        let mut end = start;

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

                    end = first_id;

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
            source: merge(start.as_ptr(), end),
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
    
    /// Tries to parse a statement.
    ///
    /// # Tokens
    ///
    /// Expects the first token of the statement to already been consumed.
    /// If it matches nothing, `None` will be returned.
    ///
    /// Ends on the next non-lb token after the statement. The caller must validate that token.
    /// The caller is free to omit warnings for any semicolons encountered.
    pub(crate) fn try_parse_statement(&mut self) -> Result<Option<Span<'a, Statement<'a>>>, Error> {
        macro_rules! after_brace {
            ($end:expr) => {{
                self.iter.advance()?;
                
                match self.iter.peek().value {
                    Token::LineBreak => {
                        // We do not care what this token is; the caller must handle it.
                        self.iter.advance()?;
                    }
                    Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => {}
                    Token::Symbol(Symbol::Semicolon) => {
                        // Now it ends on the semicolon.
                        $end = self.iter.peek().source;
                        
                        if self.iter.advance_skip_lb()? {
                            self.iter.warnings_mut().push(Span {
                                value: Warning::UnnecessarySemicolon,
                                source: $end
                            })
                        }
                    }
                    _ => todo!()
                }
            }};
        }

        let start = self.iter.peek().source.as_ptr();

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

        let statement_kind: Option<(StatementKind<'a>, &'a str)> = match self.iter.peek().value {
            Token::Keyword(Keyword::Fn) => {
                let function_start = self.iter.peek().source.as_ptr();

                self.iter.advance_skip_lb()?;

                let type_parameters = if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek().value {
                    self.iter.advance()?;
                    self.parse_type_parameter_declarations()?
                } else {
                    Vec::new()
                };

                let mut id = match &self.iter.peek().value {
                    Token::Identifier(id) => *id,
                    token => todo!("{:?}", token)
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

                let return_type: Option<Span<'a, Type<'a>>> = if let Token::Symbol(Symbol::MinusRightAngle) = self.iter.peek().value {
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
                let mut end = self.iter.peek().source;
                
                after_brace!(end);

                Some((
                    StatementKind::Declaration {
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
                                    tps: type_parameters,
                                },
                                body: Box::new(body.map(|vec| Expression::Block(vec))),
                            },
                            source: merge(function_start, end),
                        })),
                    },
                    end,
                ))
            }
            Token::Keyword(Keyword::Mod) => {
                self.iter.advance_skip_lb()?;

                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::E0071)
                };

                let mut end: &'a str = self.iter.peek().source;
                let line_break = self.iter.advance_skip_lb()?;
                
                let content: Option<_> = match &self.iter.peek().value {
                    // Code: mod xyz { ... }
                    Token::Symbol(Symbol::LeftBrace) => {
                        self.iter.advance_skip_lb()?;
                        
                        let content = self.parse_module_content()?;

                        // Validate that the module has ended on `}`:
                        match self.iter.peek().value {
                            Token::Symbol(Symbol::RightBrace) => {}
                            _ => todo!()
                        }

                        // Span ends on the line break.
                        end = self.iter.peek().source;
                        
                        after_brace!(end);
                        
                        Some(content)
                    }
                    Token::Symbol(Symbol::Semicolon) if !line_break => {
                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                        None
                    }
                    Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => None,
                    _ if line_break => None,
                    token => todo!("{:?}", token)
                };
                
                Some((
                    StatementKind::Module {
                        id,
                        content,
                        doc_comments,
                    },
                    end
                ))
            }
            Token::Keyword(Keyword::Struct) => {
                self.iter.advance_skip_lb()?;
                
                let tps = match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftAngle) => {
                        self.iter.advance_skip_lb()?;
                        self.parse_type_parameter_declarations()?
                    }
                    _ => Vec::new()
                };
                
                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!()
                };
                
                let mut end = self.iter.peek().source;
                let mut fields = Vec::new();
                
                let lb = self.iter.advance_skip_lb()?;
                
                match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftParenthesis) => {
                        self.iter.advance_skip_lb()?;
                        
                        loop {
                            let start = self.iter.peek().source.as_ptr();
                            
                            let is_public = match self.iter.peek().value {
                                Token::Keyword(Keyword::Pub) => {
                                    self.iter.advance_skip_lb()?;
                                    true
                                }
                                Token::Symbol(Symbol::RightParenthesis) => break,
                                _ => false
                            };

                            let is_mutable = if let Token::Keyword(Keyword::Mut) = self.iter.peek().value {
                                self.iter.advance_skip_lb()?;
                                true
                            } else {
                                false
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

                            let end = ty.source;

                            fields.push(Span {
                                value: StructField {
                                    is_public,
                                    is_mutable,
                                    id,
                                    ty,
                                },
                                source: merge(start, end),
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
                    
                        end = self.iter.peek().source;
                        self.iter.advance_skip_lb()?;
                    }
                    _ if lb => {},
                    _ => todo!()
                }
                
                Some((
                    StatementKind::Struct {
                        id,
                        tps,
                        fields,
                        doc_comments,
                    },
                    end
                ))
            }
            Token::Keyword(Keyword::Let) => {
                self.iter.advance_skip_lb()?;
                
                let is_mutable = match self.iter.peek().value {
                    Token::Keyword(Keyword::Mut) => {
                        self.iter.advance_skip_lb()?;
                        true
                    },
                    _ => false,
                };
                
                let id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!(),
                };
                
                let mut lb = self.iter.advance_skip_lb()?;

                let ty = match self.iter.peek().value {
                    Token::Symbol(Symbol::Colon) => {
                        self.iter.advance_skip_lb()?;
                        let ty = self.parse_type()?;
                        lb = self.iter.skip_lb()?;
                        Some(ty)
                    }
                    _ => None,
                };
                
                let value = match self.iter.peek().value {
                    Token::Symbol(Symbol::Equals) => {
                        self.iter.advance_skip_lb()?;
                        let expr = self.parse_expression(0)?;

                        match self.iter.peek().value {
                            Token::Symbol(Symbol::RightParenthesis)
                            | Token::Symbol(Symbol::Comma)
                            | Token::Symbol(Symbol::RightBracket) => todo!(),
                            Token::Symbol(Symbol::RightBrace)
                            | Token::EndOfInput
                            | Token::Symbol(Symbol::Semicolon) => {}
                            Token::LineBreak => self.iter.advance()?,
                            _ => unreachable!()
                        }
                        
                        Some(expr)
                    }
                    _ if lb => None,
                    _ => todo!()
                };

                let end = if let Some(Span { source, .. }) = &value {
                    source
                } else {
                    self.iter.peek().source // TODO: Major bug here
                };
                
                Some((
                    StatementKind::Declaration {
                        doc_comments,
                        is_mutable,
                        ty,
                        id,
                        value: value.map(Box::new),
                    },
                    end
                ))
            }
            Token::Keyword(Keyword::Use) => {
                if doc_comments.len() > 0 {
                    todo!("Error: cannot add a doc comment to a use-statement")
                }

                self.iter.advance_skip_lb()?;
                
                let root_id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!(),
                };
                
                let mut end = self.iter.peek().source; // TODO: fix this bug
                
                let (u, lb) = self.parse_use(root_id)?;
                
                match self.iter.peek().value {
                    Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => {}
                    _ if lb => {}
                    Token::Symbol(Symbol::Semicolon) => {
                        self.opt_omit_unnecessary_delimiter_warning(Warning::UnnecessarySemicolon)?;
                    }
                    _ => todo!("{:?}", self.iter.peek().value)
                }
                
                Some((
                    StatementKind::Use(u),
                    end
                ))
            }
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
                    
                    let mut end = self.iter.peek().source;
                    after_brace!(end);
                    
                    Some((
                        StatementKind::TypeParameterAlias {
                            tps,
                            content,
                        },
                        end
                    ))
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

        Ok(statement_kind.map(|(statement_kind, end)| Span {
            value: Statement {
                annotations,
                statement_kind,
            },
            source: merge(start, end),
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
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<'a, Expression<'a>>, Error> {
        let first_source = self.iter.peek().source;

        let mut left_side: Span<'a, Expression<'a>> = match &self.iter.peek().value {
            Token::String(s) => {
                let s = s.process()?;
                self.iter.advance()?;

                Span {
                    value: Expression::String(s),
                    source: first_source,
                }
            }
            Token::Number(n) => {
                let n = *n;
                self.iter.advance()?;

                Span {
                    value: Expression::Number(n),
                    source: first_source,
                }
            }
            Token::Identifier(id) => {
                let id = *id;
                self.iter.advance()?;
                Span {
                    value: Expression::Identifier(id),
                    source: first_source,
                }
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
                            continue
                        }
                        Token::Keyword(Keyword::Mut) => {
                            self.iter.advance_skip_lb()?;
                            true
                        },
                        _ => false,
                    };

                    let id = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => todo!("expected id or mut")
                    };

                    let mut lb = self.iter.advance_skip_lb()?;

                    let ty = match self.iter.peek().value {
                        Token::Symbol(Symbol::Colon) => {
                            self.iter.advance_skip_lb()?;

                            let ty = self.parse_type()?;
                            lb = self.iter.skip_lb()?;
                            Some(ty)
                        }
                        _ => None,
                    };

                    // (mut x = 20\n

                    let init = match self.iter.peek().value {
                        Token::Symbol(Symbol::Equals) => {
                            self.iter.advance_skip_lb()?;
                            self.parse_expression(0)?
                        },
                        Token::Symbol(Symbol::RightParenthesis)
                        | Token::Symbol(Symbol::Comma) => Span {
                            value: Expression::Identifier(id),
                            source: id
                        },
                        _ if lb => Span {
                            value: Expression::Identifier(id),
                            source: id
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
                            let source = self.iter.peek().source;

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
                        _ => {} // Validity is checked at the start of the loop
                    }
                }

                let end = self.iter.peek().source;

                self.iter.advance()?;

                Span {
                    value: Expression::Instance(fields),
                    source: merge(first_source.as_ptr(), end),
                }
            }
            Token::Symbol(Symbol::LeftBrace) => {
                let span = self.parse_block()?;
                self.iter.advance()?;
                span.map(|block| Expression::Block(block))
            }
            Token::Keyword(Keyword::Fn) => {
                self.iter.advance_skip_lb()?;

                let tps = match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftAngle) => {
                        self.iter.advance_skip_lb()?;
                        self.parse_type_parameter_declarations()?
                    }
                    _ => Vec::new(),
                };

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

                Span {
                    source: merge(first_source.as_ptr(), body.source),
                    value: Expression::Function {
                        signature: FunctionSignature {
                            return_type,
                            parameters,
                            has_this_parameter: false,
                            tps,
                        },
                        body: Box::new(body),
                    },
                }
            }
            token => todo!("Unexpected token: {:?}", token)
        };

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.iter.skip_lb()?;
                self.iter.advance_skip_lb()?;

                let right = self.parse_expression($bp.1)?;
                let source = right.source;

                (Expression::Operation {
                    left: Box::new(left_side),
                    operation: $op,
                    right: Box::new(right)
                }, source)
            }};
        }

        loop {
            // At this point, a line break must not have been skipped.
            let (token, line_break) = self.iter.peek_non_lb()?;

            let (value, last_source) = match &token.value {
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

                    enum Res<'a> {
                        Unnamed,
                        Named(&'a str)
                    }

                    let res = match self.iter.peek().value {
                        Token::Identifier(id) => {
                            match self.iter.peek_after()?.value {
                                Token::Symbol(Symbol::Equals) => {
                                    // Skip the identifier and the =
                                    self.iter.advance()?;
                                    self.iter.advance_skip_lb()?;

                                    Res::Named(id)
                                },
                                _ => Res::Unnamed
                            }
                        },
                        _ => Res::Unnamed
                    };

                    let arguments: CallArguments<'a> = match res {
                        Res::Unnamed => {
                            let mut args = Vec::new();

                            loop {
                                match self.iter.peek().value {
                                    Token::Symbol(Symbol::Comma) => {
                                        self.omit_single_token_warning(Warning::UnnecessaryComma);
                                        self.iter.advance_skip_lb()?;
                                        continue
                                    }
                                    Token::Symbol(Symbol::RightParenthesis) => break,
                                    _ => {}
                                }

                                let expr = self.parse_expression(0)?;

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
                        Res::Named(mut arg) => {
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
                                            value: (),
                                            source: arg,
                                        },
                                        expr
                                    ));
                                }};
                            }

                            parse_that!();
                            
                            loop {
                                arg = match self.iter.peek().value {
                                    Token::Symbol(Symbol::Comma) => {
                                        self.iter.advance_skip_lb()?;
                                        continue
                                    }
                                    Token::Symbol(Symbol::RightParenthesis) => break,
                                    Token::Identifier(id) => {
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
                        }
                    };

                    let last_source = self.iter.peek().source;
                    
                    self.iter.advance_skip_lb()?;

                    (Expression::Call {
                        target: Box::new(left_side),
                        arguments,
                    }, last_source)
                }
                Token::EndOfInput
                | Token::Symbol(Symbol::RightBrace)
                | Token::Symbol(Symbol::Semicolon)
                | Token::Symbol(Symbol::Comma)
                | Token::Symbol(Symbol::RightParenthesis)
                | Token::Symbol(Symbol::RightBracket) => {
                    self.iter.skip_lb()?;
                    break
                },
                _ if line_break => break,
                token => todo!("{:?}", token)
            };

            // TODO: maybe move into `match`
            left_side = Span {
                value,
                source: merge(first_source.as_ptr(), last_source),
            };
        }

        Ok(left_side)
    }

    /// Expects that the first token after `{` was already consumed.
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
                    continue
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
        }

        Ok(ModuleContent(items))
    }
}