use crate::token::WithSpan;

#[derive(Debug)]
pub enum Expression {
    Addition(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    Subtraction(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    Multiplication(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    Division(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    Remainder(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    Exponentiation(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseOr(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseAnd(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseExclusiveOr(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LogicalOr(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LogicalAnd(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),

    AdditionAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    SubtractionAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    MultiplicationAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    DivisionAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    RemainderAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    ExponentiationAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseOrAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseAndAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    BitwiseExclusiveOrAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LogicalOrAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LogicalAndAssignment(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    
    Function {
        is_async: bool,
        parameters: Vec<Parameter>,
        body: Vec<WithSpan<Expression>>
    },
    Number(f64),
    String(String),
    Block(Vec<WithSpan<Expression>>)
}

#[derive(Debug)]
pub struct Parameter {
    identifier: String,
    is_mutable: bool,
    ty: Type,
}

#[derive(Debug)]
pub struct Type {
    generics: Vec<Item>,
    item: Item,
}

#[derive(Debug)]
pub struct Item {
    path: Vec<String>
}