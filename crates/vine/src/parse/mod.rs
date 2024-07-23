mod tests;

use lex::token::{Keyword, Symbol, Token, TokenIterator};
use crate::{Error, Span};

use parse::ast::*;
use parse::bp;

pub struct ParseContext<'s, T: TokenIterator<'s>> {
    token_iterator: T,
    last_token: Span<Token<'s>>,
}

impl<'s, T: TokenIterator<'s>> ParseContext<'s, T> {
    #[inline]
    pub fn new(mut token_iterator: T) -> Result<ParseContext<'s, T>, Error> {
        Ok(Self {
            last_token: token_iterator.next_token()?,
            token_iterator,
        })
    }

    #[inline]
    pub fn token_iterator(&self) -> &T {
        &self.token_iterator
    }

    #[inline]
    fn next_token(&mut self) -> Result<(), Error> {
        match self.token_iterator.next_token() {
            Ok(x) => {
                self.last_token = x;
                Ok(())
            }
            Err(x) => Err(x),
        }
    }

    #[inline]
    pub fn last_token(&self) -> &Span<Token> {
        &self.last_token
    }

    /// Assumes that the first token of the module has already been generated.
    /// Ends on [Symbol::RightBrace] or [Token::EndOfInput].
    pub fn parse_module(&mut self) -> Result<ModuleContent<'s>, Error> {
        let mut items = Vec::new();

        loop {
            let is_public = match &self.last_token.value {
                Token::EndOfInput
                | Token::Symbol(Symbol::RightBrace) => break,
                Token::Keyword(Keyword::Pub) => {
                    self.next_token()?;
                    true
                }
                _ => false
            };

            items.push(TopLevelItem {
                is_public,
                statement: self.try_parse_statement()?.ok_or_else(|| Error::E0039)?,
            });

            self.next_token()?;
        }

        Ok(ModuleContent(items))
    }

    /// Parses type parameters.
    ///
    /// First token is `<` or other. Ends on the token after `>`.
    fn parse_tps(&mut self) -> Result<Vec<TypeParameter<'s>>, Error> {
        match &self.last_token.value {
            Token::Symbol(Symbol::LeftAngle) => {}
            _ => return Ok(Vec::new()),
        };

        let mut tps = Vec::new();

        self.next_token()?;

        loop {
            let id = match &self.last_token.value {
                Token::Identifier(id) => *id,
                Token::Symbol(Symbol::RightAngle) => break,
                _ => return Err(Error::E0040),
            };

            tps.push(TypeParameter {
                id,
                traits: Vec::new(), // TODO: traits
            });

            self.next_token()?;
            match &self.last_token.value {
                Token::Symbol(Symbol::Colon) => todo!("traits"),
                Token::Symbol(Symbol::Comma) => self.next_token()?,
                _ => {}
            }
        }

        self.next_token()?;

        Ok(tps)
    }

    /// Tries to parse a statement.
    ///
    /// # Tokens
    ///
    /// Expects the first token of the statement to already been consumed.
    /// If it matches nothing, `None` will be returned.
    ///
    /// Ends on [Symbol::Semicolon] or [Symbol::RightBrace] if [Some].
    pub fn try_parse_statement(&mut self) -> Result<Option<Span<Statement<'s>>>, Error> {
        let start = self.last_token.start;

        let mut annotations = Vec::new();

        loop {
            match &self.last_token.value {
                Token::Symbol(Symbol::At) => {}
                _ => break,
            }

            self.next_token()?;

            let id = match &self.last_token.value {
                Token::Identifier(id) => *id,
                _ => return Err(Error::E0042)
            };

            annotations.push(Annotation {
                path: ItemPath(vec![id]),
                arguments: vec![], // TODO: Path + arguments of annotations
            });

            self.next_token()?;
        }

        let statement_kind = match &self.last_token.value {
            Token::Keyword(Keyword::Mod) => {
                self.next_token()?;

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => return Err(Error::E0071),
                };

                self.next_token()?;

                Some(StatementKind::Module {
                    id,
                    content: match &self.last_token.value {
                        Token::Symbol(Symbol::LeftBrace) => {
                            self.next_token()?;

                            let module = self.parse_module()?;

                            match &self.last_token.value {
                                Token::Symbol(Symbol::RightBrace) => {}
                                _ => return Err(Error::E0072)
                            }

                            Some(module)
                        }
                        Token::Symbol(Symbol::Semicolon) => None,
                        _ => return Err(Error::E0073),
                    },
                })
            }
            Token::Keyword(Keyword::Fn) => {
                self.next_token()?;

                let tps = self.parse_tps()?;

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => return Err(Error::E0074),
                };

                self.next_token()?;

                match &self.last_token.value {
                    Token::Symbol(Symbol::LeftParenthesis) => {}
                    _ => return Err(Error::E0076),
                }

                let (parameters, has_this_parameter) = self.parse_function_parameters()?;

                self.next_token()?;

                let return_type = Some(match &self.last_token.value {
                    Token::Symbol(Symbol::LeftBrace) => Type::Nil,
                    Token::Symbol(Symbol::MinusRightAngle) => {
                        let ty = self.parse_type()?;
                        
                        match &self.last_token.value {
                            Token::Symbol(Symbol::LeftBrace) => {}
                            _ => return Err(Error::E0077)
                        }

                        ty
                    },
                    _ => return Err(Error::E0075)
                });

                let body = Box::new(Span {
                    start: self.last_token.start,
                    value: Expression::Scope(self.parse_scope()?),
                    end: self.last_token.end,
                });

                Some(StatementKind::Declaration {
                    id,
                    ty: None,
                    value: Some(Box::new(Span {
                        value: Expression::Function {
                            signature: FunctionSignature {
                                return_type,
                                parameters,
                                has_this_parameter,
                                tps,
                            },
                            body,
                        },
                        start,
                        end: self.last_token.end,
                    })),
                    is_mutable: false,
                })
            }
            Token::Keyword(Keyword::Use) => {
                self.next_token()?;

                match &self.last_token.value {
                    Token::Identifier(id) => Some(StatementKind::Use(self.parse_use(*id)?)),
                    Token::Symbol(Symbol::Dot) => {
                        Some(StatementKind::RootUse(self.try_parse_use_child()?.ok_or_else(|| Error::E0047)?))
                    }
                    token => return Err(Error::E0048),
                }
            }
            Token::Keyword(Keyword::Let) => {
                self.next_token()?;

                let is_mutable = match &self.last_token.value {
                    Token::Keyword(Keyword::Mut) => {
                        self.next_token()?;
                        true
                    }
                    _ => false,
                };

                let id = match &self.last_token.value {
                    Token::Identifier(identifier) => *identifier,
                    _ => return Err(Error::E0050),
                };

                self.next_token()?;

                let ty = match &self.last_token.value {
                    Token::Symbol(Symbol::Colon) => Some(self.parse_type()?),
                    _ => None
                };

                let value = match &self.last_token.value {
                    Token::Symbol(Symbol::Equals) => {
                        self.next_token()?;
                        Some(Box::new(self.parse_expression(bp::COMMA_AND_SEMICOLON)?))
                    }
                    _ => None,
                };

                Some(StatementKind::Declaration {
                    is_mutable,
                    ty,
                    value,
                    id,
                })
            }
            Token::Keyword(Keyword::Struct) => {
                self.next_token()?;

                // Parse type parameters
                let tps = self.parse_tps()?;

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => return Err(Error::E0050),
                };

                self.next_token()?;

                match &self.last_token.value {
                    Token::Symbol(Symbol::LeftParenthesis) => {}
                    _ => return Err(Error::E0052),
                }

                let mut fields = Vec::new();

                self.next_token()?;

                loop {
                    let start = self.last_token.start;

                    let is_public = match &self.last_token.value {
                        Token::Symbol(Symbol::RightParenthesis) => break,
                        Token::Keyword(Keyword::Pub) => {
                            self.next_token()?;
                            true
                        }
                        _ => false,
                    };

                    let id = match &self.last_token.value {
                        Token::Identifier(id) => *id,
                        _ => return Err(Error::E0053),
                    };

                    self.next_token()?;

                    let ty = match &self.last_token.value {
                        Token::Symbol(Symbol::Colon) => Some(self.parse_type()?),
                        _ => None,
                    };

                    fields.push(Span {
                        start,
                        value: StructField {
                            is_public,
                            id,
                            ty,
                        },
                        end: self.last_token.end,
                    });

                    match &self.last_token.value {
                        Token::Symbol(Symbol::Comma) => self.next_token()?,
                        Token::Symbol(Symbol::RightParenthesis) => {}
                        _ => return Err(Error::E0054),
                    }
                }

                Some(StatementKind::Struct {
                    id,
                    tps,
                    fields,
                })
            }
            _ => {
                if annotations.len() != 0 {
                    return Err(Error::E0041);
                }
                None
            }
        };

        Ok(statement_kind.map(|statement_kind| Span {
            start,
            value: Statement {
                annotations,
                statement_kind,
            },
            end: self.last_token.end,
        }))
    }

    /// Expects the first token to be generated. If it returns `None`, it does nothing,
    /// otherwise it ends on either [Symbol::Comma], [Symbol::RightBrace] or [Symbol::Semicolon].
    fn try_parse_use_child(&mut self) -> Result<Option<UseChild<'s>>, Error> {
        Ok(match &self.last_token.value {
            Token::Symbol(Symbol::Semicolon)
            | Token::Symbol(Symbol::Comma)
            | Token::Symbol(Symbol::RightBrace) => None,
            Token::Symbol(Symbol::Dot) => {
                self.next_token()?;

                match &self.last_token.value {
                    Token::Identifier(id) => Some(UseChild::Single(Box::new(self.parse_use(*id)?))),
                    Token::Symbol(Symbol::LeftBrace) => {
                        self.next_token()?;

                        let mut children = Vec::new();

                        loop {
                            match &self.last_token.value {
                                Token::Identifier(id) => children.push(self.parse_use(*id)?),
                                Token::Symbol(Symbol::RightBrace) => break,
                                _ => return Err(Error::E0043),
                            }

                            match &self.last_token.value {
                                Token::Symbol(Symbol::Comma) => self.next_token()?,
                                Token::Symbol(Symbol::RightBrace) => {}
                                _ => return Err(Error::E0044),
                            }
                        }

                        Some(UseChild::Multiple(children))
                    }
                    Token::Symbol(Symbol::Star) => {
                        self.next_token()?;
                        Some(UseChild::All)
                    }
                    _ => return Err(Error::E0045)
                }
            }
            token => return Err(Error::E0046)
        })
    }

    /// Expects the first token to be the identifier. Ends on either [Symbol::Comma], [Symbol::RightBrace] or [Symbol::Semicolon].
    ///
    /// # Syntax
    ///
    /// ```vine
    /// use x; // no child
    /// use x.y; // single child
    /// use x.{y, z}; // multiple children
    /// use x.*; // all children
    /// ```
    #[inline]
    fn parse_use(&mut self, id: &'s str) -> Result<Use<'s>, Error> {
        self.next_token()?;

        Ok(Use {
            id,
            child: self.try_parse_use_child()?,
        })
    }

    /// Assumes that the last token is `{`. Ends on `}`.
    fn parse_scope(&mut self) -> Result<Vec<Span<StatementOrExpression<'s>>>, Error> {
        let mut body = Vec::new();

        self.next_token()?;

        loop {
            body.push(self.parse_statement_or_expression(bp::COMMA_AND_SEMICOLON)?);

            match &self.last_token.value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.next_token()?;

                    if let Token::Symbol(Symbol::RightBrace) = &self.last_token.value {
                        body.push(Span {
                            value: StatementOrExpression::Expression(Expression::Object(Vec::new())),
                            start: self.last_token.start,
                            end: self.last_token.end,
                        });
                        break;
                    }
                }
                Token::Symbol(Symbol::RightBrace) => break,
                _ => return Err(Error::E0049)
            }
        }

        Ok(body)
    }

    fn parse_markup_element(&mut self, identifier: &'s str, start: u64) -> Result<MarkupElement<'s>, Error> {
        let mut attributes = Vec::new();

        let children = loop {
            self.next_token()?;

            let key = match &self.last_token.value {
                Token::MarkupKey(key) => *key,
                Token::MarkupStartTagEnd => {
                    // Parse children

                    self.next_token()?;

                    let mut children = Vec::new();

                    loop {
                        children.push(match &self.last_token.value {
                            Token::Symbol(Symbol::LeftBrace) => {
                                let block = MarkupChild::Insert(Expression::Scope(self.parse_scope()?));
                                self.next_token()?;
                                block
                            }
                            Token::MarkupStartTag(tag_name) => {
                                MarkupChild::Element(self.parse_markup_element(tag_name, start)?)
                            }
                            Token::MarkupEndTag(tag_name) => {
                                if *tag_name != identifier {
                                    // TODO: Display tag names.
                                    return Err(Error::E0055);
                                }
                                self.next_token()?;
                                break;
                            }
                            Token::MarkupText(text) => {
                                let text = MarkupChild::Text(text);
                                self.next_token()?;
                                text
                            }
                            token => unreachable!("Got token: {:?}. This token should not have been generated by the lexer.", token)
                        });
                    }

                    break children;
                }
                Token::MarkupClose => {
                    self.next_token()?;
                    break Vec::new();
                }
                token => unreachable!("Got token: {:?}. This token should not have been generated by the lexer.", token)
            };

            self.next_token()?;

            let value = match &self.last_token.value {
                Token::Symbol(Symbol::LeftBrace) => Expression::Scope(self.parse_scope()?),
                Token::String(_) => {
                    Expression::String(unsafe { self.take_string_unchecked() })
                }
                token => unreachable!("Got token: {:?}. This token should not have been generated by the lexer.", token)
            };

            attributes.push((key, value));
        };

        Ok(MarkupElement {
            identifier,
            attributes,
            children,
        })
    }

    /// Assumes that the last token is `(`. Always ends on `)`.
    fn parse_function_parameters(&mut self) -> Result<(Vec<Parameter<'s>>, bool), Error> {
        let mut parameters = Vec::new();
        let mut has_this_parameter = false;

        self.next_token()?;

        loop {
            let is_mutable = match &self.last_token.value {
                Token::Symbol(Symbol::RightParenthesis) => break,
                Token::Keyword(Keyword::Mut) => {
                    self.next_token()?;
                    true
                }
                Token::Keyword(Keyword::This) => {
                    has_this_parameter = true;

                    self.next_token()?;
                    match &self.last_token.value {
                        Token::Symbol(Symbol::Comma) => self.next_token()?,
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return Err(Error::E0056),
                    }

                    continue;
                }
                _ => false
            };

            let identifier = match &self.last_token.value {
                Token::Identifier(identifier) => *identifier,
                _ => return Err(Error::E0057)
            };

            self.next_token()?;

            let ty = match &self.last_token.value {
                Token::Symbol(Symbol::Colon) => Some(self.parse_type()?),
                _ => None,
            };

            match &self.last_token.value {
                Token::Symbol(Symbol::Comma) => self.next_token()?,
                _ => {}
            };

            parameters.push(Parameter {
                identifier,
                is_mutable,
                ty,
            })
        }

        Ok((parameters, has_this_parameter))
    }

    /// Assumes that the first token was not consumed. Consumes next.
    fn parse_type(&mut self) -> Result<Type<'s>, Error> {
        self.next_token()?;

        Ok(match &self.last_token.value {
            Token::Symbol(Symbol::ExclamationMark) => {
                self.next_token()?;
                Type::Never
            }
            Token::Identifier(id) => {
                let mut path = Vec::new();
                path.push(*id);

                loop {
                    self.next_token()?;

                    match &self.last_token.value {
                        Token::Symbol(Symbol::Dot) => {
                            self.next_token()?;

                            match &self.last_token.value {
                                Token::Identifier(id) => path.push(*id),
                                _ => return Err(Error::E0058)
                            }
                        }
                        Token::Symbol(Symbol::LeftAngle) => todo!("type parameters"),
                        _ => break
                    }
                }

                Type::ItemPath {
                    generics: vec![],
                    path: ItemPath(path),
                }
            }
            Token::Keyword(Keyword::Fn) => {
                todo!("Function")
            }
            token => return Err(Error::E0059),
        })
    }

    fn parse_statement_or_expression(&mut self, min_bp: u8) -> Result<Span<StatementOrExpression<'s>>, Error> {
        Ok(if let Some(statement) = self.try_parse_statement()? {
            statement
                .map(|s| StatementOrExpression::Statement(s))
        } else {
            self.parse_expression(min_bp)?
                .map(|e| StatementOrExpression::Expression(e))
        })
    }

    /// Assumes that the first token was already consumed.
    /// Ends on [Symbol::Semicolon] or [Symbol::Comma] or [Symbol::RightParenthesis]
    /// or [Symbol::RightBrace] or [Symbol::RightBracket] or [Symbol::LeftBrace] or [Token::EndOfInput].
    fn parse_expression(&mut self, min_bp: u8) -> Result<Span<Expression<'s>>, Error> {
        let start = self.last_token.start;

        let mut left_side = Span {
            start,
            value: match &self.last_token.value {
                Token::Number(number) => {
                    let number = Expression::Number(*number);
                    self.next_token()?;
                    number
                }
                Token::String(_) => {
                    let string = unsafe { self.take_string_unchecked() };
                    self.next_token()?;
                    Expression::String(string)
                }
                Token::Symbol(Symbol::LeftBrace) => {
                    let scope = Expression::Scope(self.parse_scope()?);
                    self.next_token()?;
                    scope
                }
                Token::MarkupStartTag(element) => Expression::Markup(self.parse_markup_element(*element, start)?),
                Token::Identifier(identifier) => {
                    let identifier = Expression::Identifier(identifier);
                    self.next_token()?;
                    identifier
                }
                Token::Keyword(Keyword::Fn) => {
                    self.next_token()?;

                    let tps = self.parse_tps()?;

                    match &self.last_token.value {
                        Token::Symbol(Symbol::LeftParenthesis) => {}
                        _ => return Err(Error::E0060),
                    }

                    let (parameters, has_this_parameter) = self.parse_function_parameters()?;

                    self.next_token()?;

                    let return_type = match &self.last_token.value {
                        Token::Symbol(Symbol::MinusRightAngle) => {
                            let ty = self.parse_type()?;
                            match &self.last_token.value {
                                Token::Symbol(Symbol::LeftBrace) => {}
                                _ => return Err(Error::E0061),
                            }
                            Some(ty)
                        },
                        _ => None
                    };
                    
                    Expression::Function {
                        signature: FunctionSignature {
                            return_type,
                            parameters,
                            has_this_parameter,
                            tps,
                        },
                        body: Box::new(self.parse_expression(bp::COMMA_AND_SEMICOLON)?),
                    }
                }
                Token::Keyword(Keyword::False) => {
                    self.next_token()?;
                    Expression::False
                }
                Token::Keyword(Keyword::True) => {
                    self.next_token()?;
                    Expression::True
                }
                Token::Symbol(Symbol::ExclamationMark) => {
                    self.next_token()?;
                    Expression::Not(Box::new(Span {
                        value: self.parse_expression(bp::NEGATE_AND_NOT)?.value,
                        start,
                        end: self.last_token.end,
                    }))
                }
                Token::Keyword(Keyword::If) => {
                    self.next_token()?;

                    let condition = self.parse_expression(0)?;

                    match &self.last_token.value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return Err(Error::E0062)
                    }

                    let body = self.parse_scope()?;

                    let mut else_ifs = Vec::new();

                    let else_body = loop {
                        let start = self.last_token.start;

                        self.next_token()?;

                        match &self.last_token.value {
                            Token::Keyword(Keyword::Else) => {
                                self.next_token()?;

                                match &self.last_token.value {
                                    Token::Keyword(Keyword::If) => {}
                                    Token::Symbol(Symbol::LeftBrace) => {
                                        let value = self.parse_scope()?;

                                        self.next_token()?;

                                        break Some(Span {
                                            value,
                                            start,
                                            end: self.last_token.end,
                                        });
                                    }
                                    token => return Err(Error::E0063),
                                }

                                self.next_token()?;

                                let condition = self.parse_expression(0)?;

                                let start = self.last_token.start;

                                match &self.last_token.value {
                                    Token::Symbol(Symbol::LeftBrace) => {}
                                    _ => return Err(Error::E0064),
                                }

                                else_ifs.push(If {
                                    condition: Box::new(condition),
                                    body: Span {
                                        start,
                                        value: self.parse_scope()?,
                                        end: self.last_token.end,
                                    },
                                })
                            }
                            _ => break None
                        }
                    };

                    Expression::If {
                        base: If {
                            condition: Box::new(condition),
                            body: Span {
                                value: body,
                                start,
                                end: self.last_token.end,
                            },
                        },
                        else_ifs,
                        else_body,
                    }
                }
                Token::Keyword(Keyword::Return) => {
                    self.next_token()?;
                    Expression::Return(Box::new(self.parse_expression(bp::RETURN)?))
                }
                Token::Keyword(Keyword::Continue) => {
                    self.next_token()?;
                    Expression::Continue
                }
                Token::Keyword(Keyword::Break) => {
                    self.next_token()?;
                    Expression::Break
                }
                Token::Keyword(Keyword::This) => {
                    self.next_token()?;
                    Expression::This
                }
                Token::Keyword(Keyword::While) => {
                    self.next_token()?;

                    let condition = self.parse_expression(bp::COMMA_AND_SEMICOLON)?;

                    match &self.last_token.value {
                        Token::Symbol(Symbol::LeftBrace) => {}
                        _ => return Err(Error::E0065),
                    }

                    let body = self.parse_scope()?;
                    self.next_token()?;

                    Expression::While {
                        condition: Box::new(condition),
                        body: Span {
                            start: self.last_token.start,
                            value: body,
                            end: self.last_token.end,
                        },
                    }
                }
                token => return Err(Error::E0066),
            },
            end: self.last_token.end,
        };

        macro_rules! op {
            ($op: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.next_token()?;

                Expression::Operation {
                    left: Box::new(left_side),
                    operation: $op,
                    right: Box::new(self.parse_expression($bp.1)?)
                }
            }};
        }

        loop {
            left_side = Span {
                start,
                value: match &self.last_token.value {
                    Token::Symbol(Symbol::Plus) => op!(Operation::PA(PAOperation::Addition), bp::ADDITIVE),
                    Token::Symbol(Symbol::Minus) => op!(Operation::PA(PAOperation::Subtraction), bp::ADDITIVE),
                    Token::Symbol(Symbol::Star) => op!(Operation::PA(PAOperation::Multiplication), bp::MULTIPLICATIVE),
                    Token::Symbol(Symbol::Slash) => op!(Operation::PA(PAOperation::Division), bp::MULTIPLICATIVE),
                    Token::Symbol(Symbol::Percent) => op!(Operation::PA(PAOperation::Remainder), bp::MULTIPLICATIVE),
                    Token::Symbol(Symbol::StarStar) => op!(Operation::PA(PAOperation::Exponentiation), bp::EXPONENTIAL),
                    Token::Symbol(Symbol::Pipe) => op!(Operation::PA(PAOperation::BitwiseOr), bp::BITWISE_OR),
                    Token::Symbol(Symbol::Ampersand) => op!(Operation::PA(PAOperation::BitwiseAnd), bp::BITWISE_AND),
                    Token::Symbol(Symbol::Caret) => {
                        op!(Operation::PA(PAOperation::BitwiseExclusiveOr), bp::BITWISE_XOR)
                    }
                    Token::Symbol(Symbol::PipePipe) => op!(Operation::PA(PAOperation::LogicalOr), bp::LOGICAL_OR),
                    Token::Symbol(Symbol::AmpersandAmpersand) => {
                        op!(Operation::PA(PAOperation::LogicalAnd), bp::LOGICAL_AND)
                    }
                    Token::Symbol(Symbol::LeftAngleLeftAngle) => op!(Operation::PA(PAOperation::ShiftLeft), bp::SHIFT),
                    Token::Symbol(Symbol::RightAngleRightAngle) => {
                        op!(Operation::PA(PAOperation::ShiftRight), bp::SHIFT)
                    }

                    Token::Symbol(Symbol::EqualsEquals) => op!(Operation::Comp(ComparativeOperation::Equals), bp::EQUALITY),
                    Token::Symbol(Symbol::ExclamationMarkEquals) => {
                        op!(Operation::Comp(ComparativeOperation::NotEquals), bp::EQUALITY)
                    }
                    Token::Symbol(Symbol::LeftAngle) => op!(Operation::Comp(ComparativeOperation::LessThan), bp::RELATIONAL),
                    Token::Symbol(Symbol::LeftAngleEquals) => {
                        op!(Operation::Comp(ComparativeOperation::LessThanOrEqual), bp::RELATIONAL)
                    }
                    Token::Symbol(Symbol::RightAngle) => op!(Operation::Comp(ComparativeOperation::GreaterThan), bp::RELATIONAL),
                    Token::Symbol(Symbol::RightAngleEquals) => {
                        op!(Operation::Comp(ComparativeOperation::GreaterThanOrEqual), bp::RELATIONAL)
                    }

                    Token::Symbol(Symbol::Equals) => {
                        if bp::ASSIGNMENT.0 < min_bp {
                            break;
                        }

                        self.next_token()?;

                        let Span {
                            start,
                            value,
                            end
                        } = left_side;

                        if let Ok(target) = AssignmentTarget::try_from(value) {
                            Expression::Assignment {
                                target: Box::new(Span {
                                    value: target,
                                    start,
                                    end,
                                }),
                                operation: None,
                                value: Box::new(self.parse_expression(bp::RELATIONAL.1)?),
                            }
                        } else {
                            return Err(Error::E0067)
                        }
                    }

                    Token::Symbol(Symbol::Semicolon)
                    | Token::Symbol(Symbol::Comma)
                    | Token::EndOfInput
                    | Token::Symbol(Symbol::LeftBrace)
                    | Token::Symbol(Symbol::RightBrace)
                    | Token::Symbol(Symbol::RightBracket)
                    | Token::Symbol(Symbol::RightParenthesis) => break,

                    Token::Symbol(Symbol::Dot) => {
                        if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                            break;
                        }

                        self.next_token()?;

                        let identifier = match &self.last_token.value {
                            Token::Identifier(identifier) => *identifier,
                            _ => return Err(Error::E0068),
                        };

                        self.next_token()?;

                        Expression::Access(Access {
                            property: identifier,
                            target: Box::new(left_side),
                        })
                    }
                    Token::Symbol(Symbol::LeftParenthesis) => {
                        if bp::CALL < min_bp {
                            break;
                        }

                        self.next_token()?;

                        let mut arguments = Vec::new();

                        loop {
                            if let Token::Symbol(Symbol::RightParenthesis) = &self.last_token.value {
                                break;
                            }

                            arguments.push(self.parse_expression(bp::COMMA_AND_SEMICOLON)?);

                            match &self.last_token.value {
                                Token::Symbol(Symbol::RightParenthesis) => {}
                                Token::Symbol(Symbol::Comma) => self.next_token()?,
                                _ => return Err(Error::E0069)
                            }
                        }

                        self.next_token()?;

                        Expression::Call {
                            target: Box::new(left_side),
                            arguments,
                        }
                    }
                    token => return Err(Error::E0070),
                },
                end: self.last_token.end,
            };
        }

        Ok(left_side)
    }
}