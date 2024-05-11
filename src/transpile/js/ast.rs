use std::num::NonZeroU8;

pub type Reference = NonZeroU8;

pub enum Statement {
    Return(Box<Expression>),
    Break(Option<Reference>),
    Throw(Box<Expression>),
    While {
        condition: Box<Expression>,
        body: Vec<DeclarationOrStatement>,
    },
    For {
        init: Option<Box<Expression>>,
        condition: Option<Box<Expression>>,
        body: Vec<DeclarationOrStatement>
    },
    Block(Vec<DeclarationOrStatement>),
    Empty,
    Expression(Expression)
}

pub enum DeclarationOrStatement {
    Let {
        r: Reference,
        value: Box<Expression>,
    },
    Const {
        r: Reference,
        value: Box<Expression>,
    },
    Function {
        r: Reference,
        body: Box<Vec<Expression>>,
    },
    Generator {
        r: Reference,
        body: Box<Vec<Expression>>,
    },
    AsyncFunction {
        r: Reference,
        body: Box<Vec<Expression>>,
    },
    AsyncGenerator {
        r: Reference,
        body: Box<Vec<Expression>>,
    },
    Class {
        r: Reference,
    }
}

pub enum Expression {
    
}