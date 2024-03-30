use crate::token::WithSpan;

#[derive(Debug)]
pub enum Expression {
    Equals(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    NotEquals(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LessThan(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    LessThanOrEqual(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    GreaterThan(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    GreaterThanOrEqual(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),

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
    ShiftLeft(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),
    ShiftRight(Box<WithSpan<Expression>>, Box<WithSpan<Expression>>),

    Assignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    AdditionAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    SubtractionAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    MultiplicationAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    DivisionAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    RemainderAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    ExponentiationAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    BitwiseOrAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    BitwiseAndAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    BitwiseExclusiveOrAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    LogicalOrAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    LogicalAndAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    ShiftLeftAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),
    ShiftRightAssignment(Box<WithSpan<AssignmentTarget>>, Box<WithSpan<Expression>>),

    Not(Box<WithSpan<Expression>>),
    
    Function {
        is_async: bool,
        parameters: Vec<Parameter>,
        has_this_parameter: bool,
        body: Box<WithSpan<Expression>>
    },
    Number(f64),
    String(String),
    Block(Vec<WithSpan<Expression>>),
    Markup(MarkupElement),
    Identifier(String),
    False,
    True,
    Nil,
    Declaration {
        is_mutable: bool,
        identifier: String,
        value: Option<Box<WithSpan<Expression>>>,
    },
    Access(Access),
    Call {
        target: Box<WithSpan<Expression>>,
        arguments: Vec<WithSpan<Expression>>
    }
}

#[derive(Debug)]
pub enum AssignmentTarget {
    Identifier(String),
    Access(Access)
}

impl TryFrom<Expression> for AssignmentTarget {
    type Error = ();

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct Access {
    pub target: Box<WithSpan<Expression>>,
    pub property: String,
}

#[derive(Debug)]
pub struct MarkupElement {
    pub identifier: String,
    pub attributes: Vec<(String, Expression)>,
    pub children: Vec<MarkupChild>,
}

#[derive(Debug)]
pub enum MarkupChild {
    Element(MarkupElement),
    Text(String),
    Insert(Expression),
}

#[derive(Debug)]
pub struct Parameter {
    pub identifier: String,
    pub is_mutable: bool,
    pub ty: Option<Type>,
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