#![cfg(test)]

use std::iter::Copied;
use std::slice;
use crate::lex::token::{Keyword, Symbol, Token, TokenIterator};
use crate::{Error, Span};
use crate::parse::ast::{Expression, FunctionSignature, ItemPath, Parameter, StatementOrExpression, Type, TypeParameter};
use crate::parse::{bp, Parser};

impl<'a, T> TokenIterator<'a> for T where T: Iterator<Item = Token<'a>> {
    fn next_token(&mut self) -> Result<Span<Token<'a>>, Error> {
        match self.next() {
            None => Ok(Span::zero(Token::EndOfInput)),
            Some(result) => Ok(Span::zero(result)),
        }
    }
}

fn test<const N: usize, F: FnOnce(&mut Parser<'static, Copied<slice::Iter<Token<'static>>>>) -> Result<(), Error>>(tokens: [Token<'static>; N], test: F) {
    let mut parser = Parser::new(tokens.iter().copied()).unwrap();
    if let Err(e) = test(&mut parser) {
        panic!("Error: {:?}\n\nLast token: {:?}", e, parser.last_token);
    }
}

#[test]
fn scope() {
    let scope = Parser::new([
        Token::Symbol(Symbol::LeftBrace),
        Token::Number(10.0),
        Token::Symbol(Symbol::RightBrace),
    ].iter().copied())
        .unwrap()
        .parse_scope()
        .unwrap();

    assert_eq!(scope, vec![Span::zero(StatementOrExpression::Expression(Expression::Number(10.0)))])
}

#[test]
fn tps() {
    let tps = Parser::new([
        Token::Symbol(Symbol::LeftAngle),
        Token::Identifier("T"),
        Token::Symbol(Symbol::Comma),
        Token::Identifier("U"),
        Token::Symbol(Symbol::Comma),
        Token::Symbol(Symbol::RightAngle),
    ].iter().copied())
        .unwrap()
        .parse_tps()
        .unwrap();
    
    assert_eq!(tps, vec![TypeParameter {
        id: "T",
        traits: vec![],
    }, TypeParameter {
        id: "U",
        traits: vec![],
    }])
}

#[test]
fn ex_fn() {
    fn handle(parser: &mut Parser<'static, Copied<slice::Iter<Token<'static>>>>) -> Result<(), Error> {
        assert_eq!(
            parser.parse_expression(bp::COMMA_AND_SEMICOLON)?,
            Span::zero(Expression::Function {
                signature: FunctionSignature {
                    return_type: Some(Type::Never),
                    parameters: vec![
                        Parameter {
                            identifier: "test",
                            is_mutable: true,
                            ty: Some(Type::Never),
                        },
                        Parameter {
                            identifier: "test2",
                            is_mutable: false,
                            ty: Some(Type::ItemPath {
                                generics: vec![],
                                path: ItemPath(vec!["SomeStruct"]),
                            }),
                        },
                        Parameter {
                            identifier: "test3",
                            is_mutable: false,
                            ty: None,
                        }
                    ],
                    has_this_parameter: true,
                    tps: vec![
                        TypeParameter {
                            id: "A",
                            traits: vec![],
                        },
                        TypeParameter {
                            id: "B",
                            traits: vec![],
                        }
                    ],
                },
                body: Box::new(Span::zero(Expression::Scope(vec![
                    Span::zero(StatementOrExpression::Expression(Expression::Number(1.2))),
                ]))),
            })
        );

        Ok(())
    }
    
    // fn<A, B,>(this, mut test: !, test2: SomeStruct, test3,) -> ! { 1.2 }
    test([
        Token::Keyword(Keyword::Fn),
        Token::Symbol(Symbol::LeftAngle),
        Token::Identifier("A"),
        Token::Symbol(Symbol::Comma),
        Token::Identifier("B"),
        Token::Symbol(Symbol::Comma),
        Token::Symbol(Symbol::RightAngle),
        Token::Symbol(Symbol::LeftParenthesis),
        Token::Keyword(Keyword::This),
        Token::Symbol(Symbol::Comma),
        Token::Keyword(Keyword::Mut),
        Token::Identifier("test"),
        Token::Symbol(Symbol::Colon),
        Token::Symbol(Symbol::ExclamationMark),
        Token::Symbol(Symbol::Comma),
        Token::Identifier("test2"),
        Token::Symbol(Symbol::Colon),
        Token::Identifier("SomeStruct"),
        Token::Symbol(Symbol::Comma),
        Token::Identifier("test3"),
        Token::Symbol(Symbol::Comma),
        Token::Symbol(Symbol::RightParenthesis),
        Token::Symbol(Symbol::MinusRightAngle),
        Token::Symbol(Symbol::ExclamationMark),
        Token::Symbol(Symbol::LeftBrace),
        Token::Number(1.2),
        Token::Symbol(Symbol::RightBrace),
    ], handle);
}