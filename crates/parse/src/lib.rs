#![feature(str_from_raw_parts)]
#![feature(ptr_sub_ptr)]

use std::str::from_raw_parts;
pub use buffered::*;
use error::Error;
use lex::Span;
use lex::token::{Keyword, Symbol, Token, TokenIterator};
use crate::ast::*;

mod buffered;
pub mod ast;
pub mod bp;

pub struct ParseContext<'a, T: TokenIterator<'a>> {
    pub iter: Buffered<'a, T>,
}

/// Merges two `str`s of the same memory region.
fn merge<'a>(a: &'a str, b: &'a str) -> &'a str {
    unsafe { from_raw_parts(a.as_ptr(), b.as_ptr().sub_ptr(a.as_ptr()) + b.len()) }
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
    pub fn parse_type_parameter_declarations(&mut self) -> Result<Vec<&'a str>, Error> {
        let mut params = Vec::new();

        loop {
            self.iter.skip_ws()?;

            match self.iter.peek().value {
                Token::Identifier(id) => params.push(id),
                Token::Symbol(Symbol::RightAngle) => break,
                _ => todo!()
            } // TODO: Add traits

            self.iter.skip_ws()?;

            match self.iter.peek().value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Symbol(Symbol::Comma) => {}
                _ => todo!()
            }
        }
        
        Ok(params)
    }
    
    /// Tries to parse a statement.
    ///
    /// # Tokens
    ///
    /// Expects the first token of the statement to already been consumed.
    /// If it matches nothing, `None` will be returned.
    ///
    /// Ends on [Symbol::Semicolon] or [Symbol::RightBrace] if [Some].
    pub fn try_parse_statement(&mut self) -> Result<Option<Span<'a, Statement<'a>>>, Error> {
        let start = self.iter.peek().source.as_ptr();

        let mut annotations = Vec::new();

        loop {
            match self.iter.peek().value {
                Token::Symbol(Symbol::At) => {}
                _ => break
            }

            self.iter.advance()?;

            let id = match self.iter.next()?.value {
                Token::Identifier(id) => id,
                _ => todo!()
            };

            annotations.push(Annotation {
                path: ItemPath(vec![id]),
                arguments: vec![], // TODO: Path + arguments of annotations
            });

            self.iter.advance()?;
        }

        let statement_kind: Option<StatementKind<'a>> = match self.iter.peek().value {
            Token::Keyword(Keyword::Fn) => {
                let function_start = self.iter.peek().source.as_ptr();
                
                self.iter.advance()?;

                let type_parameters = if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek().value {
                    self.iter.advance()?;
                    self.parse_type_parameter_declarations()?
                } else {
                    Vec::new()
                };

                self.iter.skip_ws()?;

                let mut id = match self.iter.peek().value {
                    Token::Identifier(id) => id,
                    _ => todo!()
                };

                let mut fn_target = None;
                
                if let Token::Symbol(Symbol::LeftAngle) = self.iter.peek().value {
                    fn_target = Some(id);

                    self.iter.skip_ws()?;

                    id = match self.iter.peek().value {
                        Token::Identifier(id) => id,
                        _ => todo!()
                    };
                }

                self.iter.skip_ws()?;

                match self.iter.peek().value {
                    Token::Symbol(Symbol::LeftParenthesis) => {
                        
                    },
                    Token::Symbol(Symbol::Dot) => {
                        todo!("On struct function decl")
                    }
                    _ => todo!()
                }
                
                // let body = self.parse_block();
                
                let function_end = self.iter.peek().source;
                self.iter.advance()?;
                
                Some(StatementKind::Declaration {
                    is_mutable: false,
                    ty: None,
                    id,
                    value: Some(Box::new(Span {
                        value: Expression::Function {
                            signature: FunctionSignature {
                                return_type: None,
                                parameters: vec![],
                                has_this_parameter: false,
                                tps: vec![],
                            },
                            body: Box::new(Span {
                                value: todo!(),
                                source: unsafe {
                                    from_raw_parts(todo!(), todo!())
                                }
                            }),
                        },
                        source: unsafe {
                            from_raw_parts(function_start, todo!())
                        },
                    })),
                })
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
                    return Err(Error::E0041)
                }
                None
            }
        };

        Ok(statement_kind.map(|statement_kind| Span {
            value: Statement {
                annotations,
                statement_kind
            },
            source: "",
        }))
    }

    /// Expects the first token to have already been generated.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Span<'a, Expression<'a>>, Error>  {
        let first_source = self.iter.peek().source;

        let (value, last_source) = match self.iter.peek().value {
            Token::String(s) => {
                self.iter.advance()?;
                (Expression::String(s.process()?), first_source)
            },
            Token::Number(n) => {
                self.iter.advance()?;
                (Expression::Number(n), first_source)
            },
            token => todo!("Unexpected token: {:?}", token)
        };
        
        let mut left_side = Span {
            value,
            source: merge(first_source, last_source)
        };

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.iter.advance()?;
                self.iter.skip_ws()?;

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
            let line_breaks = if let Token::LineBreak = self.iter.peek().value {
                self.iter.advance()?;
                self.iter.skip_ws()?;
                true
            } else {
                false
            };

            let (value, last_source) = match self.iter.peek().value {
                Token::Symbol(Symbol::Plus) => op!(Operation::PA(PAOperation::Addition), bp::ADDITIVE),
                Token::Symbol(Symbol::Minus) => op!(Operation::PA(PAOperation::Subtraction), bp::ADDITIVE),
                Token::Symbol(Symbol::Star) => op!(Operation::PA(PAOperation::Multiplication), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Slash) => op!(Operation::PA(PAOperation::Division), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Percent) => op!(Operation::PA(PAOperation::Remainder), bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::StarStar) => op!(Operation::PA(PAOperation::Exponentiation), bp::EXPONENTIAL),
                Token::Keyword(Keyword::Let) if line_breaks => todo!("Yes, this is allowed!"),
                Token::EndOfInput => break,
                token => todo!("{:?}", token)
            };

            left_side = Span {
                value,
                source: merge(first_source, last_source),
            };
        }

        Ok(left_side)
    }

    pub fn parse_module_content(&mut self) -> Result<ModuleContent, Error> {
        let mut items = Vec::new();

        loop {
            let is_public = match self.iter.peek().value {
                Token::EndOfInput | Token::Symbol(Symbol::RightBrace) => break,
                Token::Keyword(Keyword::Pub) => {
                    self.iter.skip_ws()?;
                    true
                }
                _ => false
            };

            items.push(TopLevelItem {
                is_public,
                statement: self.try_parse_statement()?.expect("This has to be a statement"),
            })
        }

        Ok(ModuleContent(items))
    }
}