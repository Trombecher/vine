pub mod ast;
pub mod bp;

use crate::lex::Lexer;
use crate::lex::token::{Keyword, Span, Symbol, Token};
use ast::*;

#[derive(Debug)]
pub enum Error {
    UnexpectedToken(UnexpectedTokenError),
    TagNamesDoNotMatch,
    InvalidAssignmentTarget,
}

#[derive(Debug)]
pub enum UnexpectedTokenError {
    ExpectedSemicolonOrRightBraceWhileParsingEndOfBlock,
    ExpectedKeywordFn,
    ExpectedIdentifierOrLeftParenthesis,
    ExpectedLeftParenthesis,
    NamedFunctionBodiesMustBeSurroundedByBraces,
}

pub struct Parser<'s> {
    lexer: Lexer<'s>,
    last_token: Span<Token<'s>>,
}

impl<'s> Parser<'s> {
    pub fn new(mut lexer: Lexer<'s>) -> Result<Parser<'s>, crate::Error> {
        Ok(Self {
            last_token: lexer.next()?,
            lexer,
        })
    }

    fn next_token(&mut self) -> Result<(), crate::Error> {
        self.last_token = self.lexer.next()?;
        Ok(())
    }

    pub fn last_token(&self) -> &Span<Token> {
        &self.last_token
    }

    /// Assumes that the first token of the module has already been generated.
    pub fn parse_module(&mut self, id: &'s str) -> Result<Module<'s>, crate::Error> {
        let mut items = Vec::new();

        loop {
            let is_public = match &self.last_token.value {
                Token::EndOfInput => break,
                Token::Keyword(Keyword::Pub) => {
                    self.next_token()?;
                    true
                }
                _ => false
            };

            items.push(TopLevelItem {
                is_public,
                statement: self.try_parse_statement()?.ok_or(crate::Error::Parser(
                    // TODO: error
                    Error::UnexpectedToken(UnexpectedTokenError::ExpectedSemicolonOrRightBraceWhileParsingEndOfBlock)
                ))?,
            });

            match &self.last_token.value {
                Token::Symbol(Symbol::Semicolon) => self.next_token()?,
                token => todo!("unexpected token: {:?}", token)
            }
        }

        Ok(Module {
            id,
            items: Some(items),
        })
    }

    /// Tries to parse a statement.
    ///
    /// # Tokens
    ///
    /// Expects the first token of the statement to already been consumed.
    /// If it matches nothing, `None` will be returned.
    ///
    /// Consumes the token after the statement if a statement is returned.
    pub fn try_parse_statement(&mut self) -> Result<Option<Span<Statement<'s>>>, crate::Error> {
        let start = self.last_token.start;

        Ok(match &self.last_token.value {
            Token::Keyword(Keyword::Mod) => {
                self.next_token()?;

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => todo!()
                };

                self.next_token()?;

                match &self.last_token.value {
                    // Token::Symbol(Symbol::LeftBrace) => {} // TODO: Module body
                    Token::Symbol(Symbol::Semicolon) => {}
                    _ => todo!()
                }

                self.next_token()?;

                Some(Span {
                    start,
                    value: Statement {
                        annotations: Vec::new(), // TODO: Annotations
                        statement_kind: StatementKind::Module(Module {
                            id,
                            items: None,
                        }),
                    },
                    end: self.last_token.end,
                })
            }
            Token::Keyword(Keyword::Async) => {
                self.next_token()?;

                match &self.last_token.value {
                    Token::Keyword(Keyword::Fn) => self.next_token()?,
                    _ => return Err(crate::Error::Parser(Error::UnexpectedToken(UnexpectedTokenError::ExpectedKeywordFn)))
                }

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => todo!()
                };

                Some(Span {
                    start,
                    value: Statement {
                        annotations: Vec::new(), // TODO: Annotations
                        statement_kind: StatementKind::Declaration(
                            self.parse_function_statement(true, id)?
                        ),
                    },
                    end: self.last_token.end,
                })
            }
            Token::Keyword(Keyword::Fn) => {
                self.next_token()?;

                let id = match &self.last_token.value {
                    Token::Identifier(id) => *id,
                    _ => todo!()
                };

                Some(Span {
                    start,
                    value: Statement {
                        annotations: vec![], // TODO: Annotations
                        statement_kind: StatementKind::Declaration(
                            self.parse_function_statement(false, id)?
                        ),
                    },
                    end: self.last_token.end,
                })
            }
            Token::Keyword(Keyword::Use) => {
                Some(Span {
                    start,
                    value: todo!("Use statements"),
                    end: 0,
                })
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

                let identifier = match &self.last_token.value {
                    Token::Identifier(identifier) => *identifier,
                    _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                        todo!()
                    )))
                };

                self.next_token()?;

                let value = match &self.last_token.value {
                    Token::Symbol(Symbol::Equals) => {
                        self.next_token()?;
                        Some(Box::new(self.parse_expression(bp::COMMA_AND_SEMICOLON)?))
                    }
                    _ => todo!()
                };

                Some(Span {
                    start,
                    value: Statement {
                        annotations: vec![],
                        statement_kind: StatementKind::Declaration(
                            Declaration {
                                is_mutable,
                                ty: Type::Unknown,
                                identifier,
                                value,
                            }
                        ),
                    },
                    end: self.last_token.end,
                })
            }
            _ => None
        })
    }

    /// Assumes that the last token is the one after `{`. Ends on `}`.
    fn parse_scope(&mut self) -> Result<Vec<Span<StatementOrExpression<'s>>>, crate::Error> {
        let mut body = Vec::new();

        loop {
            body.push(self.parse_statement_or_expression(bp::COMMA_AND_SEMICOLON)?);

            match &self.last_token.value {
                Token::Symbol(Symbol::Semicolon) => {
                    self.next_token()?;

                    if let Token::Symbol(Symbol::RightBrace) = &self.last_token.value {
                        // body.push(WithSpan {
                        //     value: Expression::Nil,
                        //     start: self.last_token.start,
                        //     end: self.last_token.end,
                        // });
                        break;
                    }
                }
                Token::Symbol(Symbol::RightBrace) => break,
                _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                    UnexpectedTokenError::ExpectedSemicolonOrRightBraceWhileParsingEndOfBlock
                ))),
            }
        }

        Ok(body)
    }

    fn parse_markup_element(&mut self, identifier: &'s str, start: usize) -> Result<MarkupElement<'s>, crate::Error> {
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
                                self.next_token()?;
                                let block = MarkupChild::Insert(Expression::Scope(self.parse_scope()?));
                                self.next_token()?;
                                block
                            }
                            Token::MarkupStartTag(tag_name) => {
                                MarkupChild::Element(self.parse_markup_element(tag_name, start)?)
                            }
                            Token::MarkupEndTag(tag_name) => {
                                if tag_name != &identifier {
                                    return Err(crate::Error::Parser(Error::TagNamesDoNotMatch));
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
                Token::Symbol(Symbol::LeftBrace) => {
                    self.next_token()?;
                    Expression::Scope(self.parse_scope()?)
                }
                Token::String(s) => Expression::String(s),
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
    fn parse_function_parameters(&mut self) -> Result<(Vec<Parameter<'s>>, bool), crate::Error> {
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
                        _ => todo!()
                    }

                    continue;
                }
                _ => false
            };

            let identifier = match &self.last_token.value {
                Token::Identifier(identifier) => *identifier,
                _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                    UnexpectedTokenError::ExpectedIdentifierOrLeftParenthesis
                )))
            };

            self.next_token()?;

            let ty = match &self.last_token.value {
                Token::Symbol(Symbol::RightParenthesis) => Type::Unknown,
                Token::Symbol(Symbol::Colon) => self.parse_type()?,
                Token::Symbol(Symbol::Comma) => {
                    self.next_token()?;
                    Type::Unknown
                }
                token => return Err(crate::Error::Parser(Error::UnexpectedToken(
                    todo!("{:?}", token)
                )))
            };

            parameters.push(Parameter {
                identifier,
                is_mutable,
                ty,
            })
        }

        Ok((parameters, has_this_parameter))
    }

    /// Assumes that the first token was not consumed.
    fn parse_type(&mut self) -> Result<Type<'s>, crate::Error> {
        self.next_token()?;
        
        Ok(match &self.last_token.value {
            Token::Keyword(Keyword::Underscore) => Type::Unknown,
            Token::Keyword(Keyword::Nil) => Type::Nil,
            Token::Symbol(Symbol::ExclamationMark) => Type::Never,
            Token::Identifier(id) => match *id {
                "number" => Type::Number,
                "bool" => Type::Boolean,
                "str" => Type::String,
                _ => {
                    let mut path = Vec::new();
                    path.push(*id);

                    loop {
                        self.next_token()?;
                        match &self.last_token.value {
                            Token::Symbol(Symbol::Dot) => {
                                self.next_token()?;

                                match &self.last_token.value {
                                    Token::Identifier(id) => path.push(*id),
                                    _ => todo!()
                                }
                            }
                            Token::Symbol(Symbol::LeftAngle) => todo!("Generics"),
                            _ => break
                        }
                    }

                    Type::ItemPath {
                        generics: vec![],
                        path: ItemPath(path),
                    }
                }
            },
            Token::Keyword(Keyword::Fn) => {
                todo!("Function")
            }
            _ => todo!()
        })
    }

    /// Assumes that the last token is `<`.
    fn parse_const_parameters(&mut self) -> Result<Vec<ConstParameter<'s>>, crate::Error> {
        let mut params = Vec::new();
        self.next_token()?;

        loop {
            match &self.last_token.value {
                Token::Symbol(Symbol::RightAngle) => break,
                Token::Identifier(s) => {
                    params.push(ConstParameter::Generic(s));
                    self.next_token()?;
                }
                _ => todo!()
            }

            match &self.last_token.value {
                Token::Symbol(Symbol::Comma) => self.next_token()?,
                _ => {}
            }
        }

        Ok(params)
    }

    /// Assumes that the last token is `(` or `<`.
    fn parse_function_expression(&mut self, is_async: bool) -> Result<Function<'s>, crate::Error> {
        let const_parameters = self.parse_const_parameters()?;
        let (parameters, has_this_parameter) = self.parse_function_parameters()?;

        self.next_token()?;

        Ok(Function {
            signature: FunctionSignature {
                return_type: Type::Unknown,
                is_async,
                parameters,
                has_this_parameter,
                const_parameters,
            },
            body: Box::new(self.parse_expression(bp::COMMA_AND_SEMICOLON)?),
        })
    }

    /// Assumes that the last token is the identifier. Ends on the token after `}`.
    fn parse_function_statement(&mut self, is_async: bool, identifier: &'s str) -> Result<Declaration<'s>, crate::Error> {
        let start = self.last_token.start;

        self.next_token()?;
        match &self.last_token.value {
            Token::Symbol(Symbol::LeftParenthesis) => {}
            _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                UnexpectedTokenError::ExpectedLeftParenthesis
            )))
        }

        let (parameters, has_this_parameter) = self.parse_function_parameters()?;

        self.next_token()?;

        let return_type = match &self.last_token.value {
            Token::Symbol(Symbol::LeftBrace) => Type::Nil,
            Token::Symbol(Symbol::MinusRightAngle) => self.parse_type()?,
            _ => todo!()
        };

        match &self.last_token.value {
            Token::Symbol(Symbol::LeftBrace) => {}
            _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                UnexpectedTokenError::NamedFunctionBodiesMustBeSurroundedByBraces
            )))
        }

        self.next_token()?;

        let body = Box::new(Span {
            start: self.last_token.start,
            value: Expression::Scope(self.parse_scope()?),
            end: self.last_token.end,
        });

        self.next_token()?;

        Ok(Declaration {
            is_mutable: false,
            ty: Type::Unknown,
            identifier,
            value: Some(Box::new(Span {
                value: Expression::Function(Function {
                    signature: FunctionSignature {
                        return_type,
                        is_async,
                        parameters,
                        has_this_parameter,
                        const_parameters: vec![], // TODO
                    },
                    body,
                }),
                start,
                end: self.last_token.end,
            })),
        })
    }

    fn parse_statement_or_expression(&mut self, min_bp: u8) -> Result<Span<StatementOrExpression<'s>>, crate::Error> {
        Ok(if let Some(statement) = self.try_parse_statement()? {
            statement
                .map(|s| StatementOrExpression::Statement(s))
        } else {
            self.parse_expression(min_bp)?
                .map(|e| StatementOrExpression::Expression(e))
        })
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Span<Expression<'s>>, crate::Error> {
        let start = self.last_token.start;

        let mut left_side = Span {
            start,
            value: match &self.last_token.value {
                Token::Number(number) => {
                    let number = Expression::Number(*number);
                    self.next_token()?;
                    number
                }
                Token::String(string) => {
                    let string = Expression::String(string);
                    self.next_token()?;
                    string
                }
                Token::Symbol(Symbol::LeftBrace) => {
                    self.next_token()?;
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
                Token::Keyword(Keyword::Async) => {
                    self.next_token()?;

                    match &self.last_token.value {
                        Token::Keyword(Keyword::Fn) => {
                            self.next_token()?;
                            Expression::Function(self.parse_function_expression(true)?)
                        }
                        _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                            UnexpectedTokenError::ExpectedKeywordFn
                        )))
                    }
                }
                Token::Keyword(Keyword::Fn) => {
                    self.next_token()?;
                    Expression::Function(self.parse_function_expression(false)?)
                }
                Token::Keyword(Keyword::False) => {
                    let e = Expression::False;
                    self.next_token()?;
                    e
                }
                Token::Keyword(Keyword::Nil) => {
                    let e = Expression::Nil;
                    self.next_token()?;
                    e
                }
                Token::Keyword(Keyword::True) => {
                    let e = Expression::True;
                    self.next_token()?;
                    e
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
                        _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                            todo!()
                        )))
                    }

                    self.next_token()?;

                    let body = self.parse_scope()?;

                    self.next_token()?;

                    match &self.last_token.value {
                        Token::Keyword(Keyword::Else) => todo!(),
                        _ => {}
                    }

                    Expression::If {
                        base: If {
                            condition: Box::new(condition),
                            body: Span {
                                value: body,
                                start,
                                end: self.last_token.end,
                            }
                        },
                        else_ifs: vec![], // TODO: Else if
                        else_body: None, // TODO: Else
                    }
                }
                Token::Keyword(Keyword::Return) => {
                    self.next_token()?;
                    Expression::Return(Box::new(self.parse_expression(bp::RETURN_AND_AWAIT)?))
                }
                Token::Keyword(Keyword::Await) => {
                    self.next_token()?;
                    Expression::Await(Box::new(self.parse_expression(bp::RETURN_AND_AWAIT)?))
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
                        _ => todo!()
                    }

                    self.next_token()?;

                    let expression = Expression::While {
                        condition: Box::new(condition),
                        body: Span {
                            start: self.last_token.start,
                            value: self.parse_scope()?,
                            end: self.last_token.end,
                        },
                    };

                    self.next_token()?;

                    expression
                }
                Token::Keyword(Keyword::Use) => {
                    self.next_token()?;

                    let id = match &self.last_token.value {
                        Token::Identifier(id) => *id,
                        _ => todo!()
                    };


                    todo!()
                }
                token => todo!("{:?}", token),
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
                            return Err(crate::Error::Parser(Error::InvalidAssignmentTarget));
                        }
                    }

                    Token::Symbol(Symbol::Semicolon)
                    | Token::Symbol(Symbol::Comma)
                    | Token::EndOfInput
                    | Token::Symbol(Symbol::LeftBrace) // Enables if-expressions
                    | Token::Symbol(Symbol::RightBrace)
                    | Token::Symbol(Symbol::RightParenthesis) => break,

                    Token::Symbol(Symbol::Dot) => {
                        if bp::ACCESS_AND_OPTIONAL_ACCESS < min_bp {
                            break;
                        }

                        self.next_token()?;

                        let identifier = match &self.last_token.value {
                            Token::Identifier(identifier) => *identifier,
                            _ => todo!(),
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
                                _ => return Err(crate::Error::Parser(Error::UnexpectedToken(
                                    todo!()
                                )))
                            }
                        }

                        self.next_token()?;

                        Expression::Call {
                            target: Box::new(left_side),
                            arguments,
                        }
                    }
                    token => todo!("{:?}", token),
                },
                end: self.last_token.end,
            };
        }

        Ok(left_side)
    }
}
