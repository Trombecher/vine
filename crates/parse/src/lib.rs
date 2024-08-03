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
    pub const fn new(iter: Buffered<'a, T>) -> ParseContext<'a, T> {
        Self { iter }
    }

    /*
    /// Takes the string out of the last token.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a string in the last token.
    #[inline]
    unsafe fn take_string_unchecked(&mut self) -> Result<String, Error> {
        let mut s = Token::EndOfInput; // dummy token
        swap(&mut s, &mut self.iter.peek_mut()?.value);

        if let Token::String(s) = s {
            Ok(s)
        } else {
            // SAFETY: We swapped a `self.last_token.value` (which we know it's a string)
            // with a dummy value. After the swap, there is a Token::String in `s`,
            // which we can unwrap unchecked.

            unreachable_unchecked()
        }
    }
     */

    /// Parses type declarations like this:
    ///
    /// ```text
    /// fn<A, B>
    ///    ^
    /// ```
    ///
    /// Expects the next token to be the marked.
    pub fn parse_type_parameter_declarations(&mut self) -> Result<Vec<TypeParameter<'a>>, Error> {
        let mut params = Vec::new();

        loop {
            self.iter.skip_lb()?;

            match self.iter.peek().value {
                Token::Identifier(id) => params.push(TypeParameter {
                    id,
                    traits: vec![],
                }),
                Token::Symbol(Symbol::RightAngle) => break,
                _ => todo!()
            } // TODO: Add traits

            self.iter.skip_lb()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {}
                _ => todo!()
            }
        }

        Ok(params)
    }

    /// Expects the `{` to be `peek()`. Ends on `}`.
    fn parse_block(&mut self) -> Result<Span<'a, Vec<Span<'a, StatementOrExpression<'a>>>>, Error> {
        let start = self.iter.peek().source.as_ptr();

        self.iter.advance()?;

        let mut items = Vec::new();

        loop {
            self.iter.skip_lb()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::RightBrace) => break,
                Token::Symbol(Symbol::Semicolon) => {}
                _ => {
                    if let Some(statement) = self.try_parse_statement()? {
                        items.push(statement.map(|s| StatementOrExpression::Statement(s)));
                    } else {
                        items.push(self.parse_expression(0)?.map(|e| StatementOrExpression::Expression(e)));
                    }
                }
            }
        }

        Ok(unsafe {
            Span::from_ends(
                items,
                start,
                {
                    let source = self.iter.peek().source;
                    source.as_ptr().add(source.len())
                },
            )
        })
    }

    /// Expects that the first token of the type is accessible via `peek()`. Ends on the token after the type.
    pub(crate) fn parse_type(&mut self) -> Result<(Span<'a, Type<'a>>, bool), Error> {
        let start = self.iter.peek().source.as_ptr();

        let value = match &self.iter.peek().value {
            Token::Keyword(kw) if let Ok(ty) = Type::try_from(*kw) => ty,
            Token::Identifier(id) => Type::ItemPath {
                generics: vec![],
                path: ItemPath(vec![id]),
            },
            token => todo!("{:?}", token)
        };

        let source = merge(start, self.iter.peek().source);

        self.iter.advance()?; // TODO: move these into the match-statement
        let lb = self.iter.skip_lb()?;

        Ok((Span {
            value,
            source,
        }, lb))
    }

    /// Tries to parse a statement.
    ///
    /// # Tokens
    ///
    /// Expects the first token of the statement to already been consumed.
    /// If it matches nothing, `None` will be returned.
    ///
    /// Ends on the token after the statement. The caller must validate that token.
    pub(crate) fn try_parse_statement(&mut self) -> Result<Option<Span<'a, Statement<'a>>>, Error> {
        let start = self.iter.peek().source.as_ptr();

        let mut annotations = Vec::new();

        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::At) => {}
                _ => break
            }

            self.iter.advance()?;

            let id = match self.iter.peek().value {
                Token::Identifier(id) => id,
                _ => todo!()
            };

            annotations.push(Annotation {
                path: ItemPath(vec![id]),
                arguments: vec![], // TODO: Path + arguments of annotations
            });

            self.iter.advance()?;
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
                    token => todo!("{:?}", token)
                }

                self.iter.advance_skip_lb()?;

                let mut parameters = Vec::<Parameter<'a>>::new();

                loop {
                    let is_mutable = match self.iter.peek().value {
                        Token::Symbol(Symbol::RightParenthesis) => break,
                        Token::Keyword(Keyword::Mut) => {
                            self.iter.advance()?;
                            self.iter.skip_lb()?;
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

                    let (ty, line_break) = self.parse_type()?;

                    parameters.push(Parameter { id, is_mutable, ty });

                    match &self.iter.peek().value {
                        Token::Symbol(Symbol::RightParenthesis) => break,
                        Token::Symbol(Symbol::Comma) => self.iter.advance()?,
                        _ if line_break => {}
                        token => todo!("{:?}", token)
                    }
                }

                // peek() = RightParenthesis

                self.iter.advance_skip_lb()?;

                let return_type: Span<'a, Type<'a>> = if let Token::Symbol(Symbol::MinusRightAngle) = self.iter.peek().value {
                    self.iter.advance_skip_lb()?;
                    self.parse_type()?.0
                } else {
                    // Create an empty Type::Nil.
                    Span {
                        value: Type::Nil,
                        source: unsafe {
                            from_raw_parts(self.iter.peek().source.as_ptr(), 0)
                        },
                    }
                };

                // Validate that the block start with `{`
                match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftBrace) => {}
                    _ => todo!(),
                }

                let body = self.parse_block()?;
                let end = self.iter.peek().source;

                self.iter.advance()?;

                Some((
                    StatementKind::Declaration {
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
                            source: merge(function_start, end), // `}`
                        })),
                    },
                    end
                ))
            }

            /*
            Token::Keyword(Keyword::Mod) => {
                self.iter.advance()?;

                let id = match self.iter.peek()?.value {
                    Token::Identifier(id) => id,
                    _ => return Err(Error::E0071)
                };

                self.iter.advance()?;

                Some(StatementKind::Module {
                    id,
                    content: match self.iter.peek() {
                        Ok(_) => {}
                        Err(_) => {}
                    },
                })
            }
             */

            _ => {
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

    /// Expects the first token to have already been generated.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<'a, Expression<'a>>, Error> {
        let first_source = self.iter.peek().source;

        let (value, last_source) = match &self.iter.peek().value {
            Token::String(s) => {
                let s = s.process()?;
                self.iter.advance()?;
                (Expression::String(s), first_source)
            }
            Token::Number(n) => {
                let n = *n;
                self.iter.advance()?;
                (Expression::Number(n), first_source)
            }
            Token::Identifier(id) => {
                let id = *id;
                self.iter.advance()?;
                (Expression::Identifier(id), first_source)
            }
            token => todo!("Unexpected token: {:?}", token)
        };

        let mut left_side = Span {
            value,
            source: merge(first_source.as_ptr(), last_source),
        };

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.iter.advance()?;
                self.iter.skip_lb()?;

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
            let line_breaks = self.iter.skip_lb()?;

            let (value, last_source) = match &self.iter.peek().value {
                Token::Symbol(Symbol::Plus) => op!(Operation::PA(PAOperation::Addition), bp::ADDITIVE),
                Token::Symbol(Symbol::Minus) => op!(Operation::PA(PAOperation::Subtraction), bp::ADDITIVE),
                Token::Symbol(Symbol::Star) => op!(Operation::PA(PAOperation::Multiplication), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Slash) => op!(Operation::PA(PAOperation::Division), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Percent) => op!(Operation::PA(PAOperation::Remainder), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::StarStar) => op!(Operation::PA(PAOperation::Exponentiation), bp::EXPONENTIAL),
                Token::Keyword(Keyword::Let) if line_breaks => todo!("Yes, this is allowed!"),
                Token::EndOfInput
                | Token::Symbol(Symbol::RightBrace)
                | Token::Symbol(Symbol::Semicolon)
                | Token::Symbol(Symbol::RightParenthesis)
                | Token::Symbol(Symbol::RightBracket) => break,
                _ if line_breaks => break,
                token => todo!("{:?}", token)
            };

            left_side = Span {
                value,
                source: merge(first_source.as_ptr(), last_source),
            };
        }

        Ok(left_side)
    }

    /// Ends on `}` or [Token::EndOfInput].
    pub fn parse_module_content(&mut self) -> Result<ModuleContent, Error> {
        let mut items = Vec::new();

        loop {
            let is_public = match self.iter.peek().value {
                Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => break,
                Token::Keyword(Keyword::Pub) => {
                    self.iter.skip_lb()?;
                    true
                }
                _ => false
            };

            items.push(TopLevelItem {
                is_public,
                statement: self.try_parse_statement()?.expect("This has to be a statement"),
            });

            match &self.iter.peek().value {
                Token::Symbol(Symbol::Semicolon) => {
                    let source = self.iter.peek().source;
                    self.iter.advance()?;
                    
                    match self.iter.peek().value {
                        Token::LineBreak => {
                            self.iter
                                .warnings_mut()
                                .push(Span {
                                    value: Warning::UnnecessarySemicolon,
                                    source,
                                });
                            self.iter.advance()?;
                        },
                        _ => {}
                    }
                },
                Token::LineBreak => self.iter.advance()?,
                Token::Symbol(Symbol::RightBrace) | Token::EndOfInput => break,
                token => todo!("{:?}", token)
            }
        }

        Ok(ModuleContent(items))
    }
}