use crate::ast::Expression;
use crate::lexer::Lexer;
use crate::token::{Symbol, Token, WithSpan};
use crate::{bp, ion};

#[derive(Debug)]
pub enum Error {
    UnexpectedToken,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    last_token: WithSpan<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, ion::Error> {
        Ok(Self {
            last_token: lexer.next()?,
            lexer,
        })
    }

    fn next_token(&mut self) -> Result<(), ion::Error> {
        self.last_token = self.lexer.next()?;
        // println!("last_token: {:?}", self.last_token);
        Ok(())
    }

    pub fn parse_block(&mut self) -> Result<Vec<WithSpan<Expression>>, ion::Error> {
        let mut expressions = Vec::new();

        loop {
            expressions.push(self.parse_expression(bp::COMMA_AND_SEMICOLON)?);

            match &self.last_token.value {
                Token::Symbol(Symbol::Semicolon) => self.next_token()?,
                Token::EndOfInput => return Ok(expressions),
                _ => return Err(ion::Error::Parser(Error::UnexpectedToken)),
            }
        }
    }

    pub fn parse_expression(&mut self, min_bp: u8) -> Result<WithSpan<Expression>, ion::Error> {
        let start = self.last_token.start;

        let mut left_side = match &self.last_token.value {
            Token::Number(number) => {
                let number = WithSpan {
                    value: Expression::Number(*number),
                    start,
                    end: self.last_token.end,
                };
                self.next_token()?;
                number
            }
            Token::String(string) => {
                let string = WithSpan {
                    value: Expression::String(string.clone()),
                    start,
                    end: self.last_token.end,
                };
                self.next_token()?;
                string
            }
            _ => return Err(ion::Error::Parser(Error::UnexpectedToken)),
        };

        macro_rules! op {
            ($e: expr, $bp: expr) => {{
                if $bp.0 < min_bp {
                    break;
                }

                self.next_token()?;

                WithSpan {
                    value: $e(Box::new(left_side), Box::new(self.parse_expression($bp.1)?)),
                    start,
                    end: self.last_token.end,
                }
            }};
        }

        loop {
            left_side = match &self.last_token.value {
                Token::Symbol(Symbol::Plus) => op!(Expression::Addition, bp::ADDITIVE),
                Token::Symbol(Symbol::Minus) => op!(Expression::Subtraction, bp::ADDITIVE),
                Token::Symbol(Symbol::Star) => op!(Expression::Multiplication, bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Slash) => op!(Expression::Division, bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::Percent) => op!(Expression::Remainder, bp::MULTIPLICATIVE),
                Token::Symbol(Symbol::StarStar) => op!(Expression::Exponentiation, bp::EXPONENTIAL),
                Token::Symbol(Symbol::Semicolon) | Token::Symbol(Symbol::Comma) | Token::EndOfInput => break,
                token => todo!("{:?}", token),
            }
        }

        Ok(left_side)
    }
}