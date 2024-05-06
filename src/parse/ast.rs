use crate::lex::token::Span;

/// A binary operation.
#[derive(Debug)]
pub enum Operation {
    PA(PAOperation),
    Comp(ComparativeOperation),
}

/// An operation that can be assigned, like `+=`.
#[derive(Debug)]
#[repr(u8)]
pub enum PAOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Remainder,
    Exponentiation,
    BitwiseOr,
    BitwiseAnd,
    BitwiseExclusiveOr,
    LogicalOr,
    LogicalAnd,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ComparativeOperation {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual
}

#[derive(Debug)]
pub enum Expression<'s> {
    Operation {
        left: Box<Span<Expression<'s>>>,
        operation: Operation,
        right: Box<Span<Expression<'s>>>,
    },
    Assignment {
        target: Box<Span<AssignmentTarget<'s>>>,
        operation: Option<PAOperation>,
        value: Box<Span<Expression<'s>>>
    },
    Not(Box<Span<Expression<'s>>>),
    Return(Box<Span<Expression<'s>>>),
    Await(Box<Span<Expression<'s>>>),
    Continue,
    Break,
    Function(Function<'s>),
    Number(f64),
    String(&'s str),
    Scope(Vec<Span<StatementOrExpression<'s>>>),
    Markup(MarkupElement<'s>),
    Identifier(&'s str),
    False,
    True,
    This,
    Nil,
    Use(Use<'s>),
    Access(Access<'s>),
    OptionalAccess(Access<'s>),
    Call {
        target: Box<Span<Expression<'s>>>,
        arguments: Vec<Span<Expression<'s>>>
    },
    If {
        base: If<'s>,
        else_ifs: Vec<If<'s>>,
        else_body: Option<Span<Vec<Span<StatementOrExpression<'s>>>>>,
    },
    While {
        condition: Box<Span<Expression<'s>>>,
        body: Span<Vec<Span<StatementOrExpression<'s>>>>
    },
}

#[derive(Debug)]
pub struct If<'s> {
    pub condition: Box<Span<Expression<'s>>>,
    pub body: Span<Vec<Span<StatementOrExpression<'s>>>>,
}

#[derive(Debug)]
pub enum StatementKind<'s> {
    Enum(Vec<&'s str>),
    Declaration(Declaration<'s>),
    Struct {
        id: &'s str,
        // TODO: struct fields
    },
    TypeAlias {
        alias: &'s str,
        ty: Type<'s>
    },
    Use(Use<'s>),
    Module(Module<'s>)
}

#[derive(Debug)]
pub struct Annotation<'s> {
    pub path: ItemPath<'s>,
    pub arguments: Vec<Span<Expression<'s>>>
}

#[derive(Debug)]
pub struct Statement<'s> {
    pub annotations: Vec<Annotation<'s>>,
    pub statement_kind: StatementKind<'s>
}

#[derive(Debug)]
pub enum StatementOrExpression<'s> {
    Statement(Statement<'s>),
    Expression(Expression<'s>)
}

#[derive(Debug)]
pub struct Module<'s> {
    pub id: &'s str,
    pub items: Option<Vec<TopLevelItem<'s>>>
}

#[derive(Debug)]
pub struct TopLevelItem<'s> {
    pub is_public: bool,
    pub statement: Span<Statement<'s>>
}

#[derive(Debug)]
pub struct Use<'s> {
    pub id: &'s str,
    pub child: Option<UseChild<'s>>
}

#[derive(Debug)]
pub enum UseChild<'s> {
    Single(Box<Use<'s>>),
    Multiple(Vec<Use<'s>>),
    All,
}

#[derive(Debug)]
pub struct Declaration<'s> {
    pub is_mutable: bool,
    pub ty: Option<Type<'s>>,
    pub identifier: &'s str,
    pub value: Option<Box<Span<Expression<'s>>>>,
}

#[derive(Debug)]
pub struct Function<'s> {
    pub signature: FunctionSignature<'s>,
    pub body: Box<Span<Expression<'s>>>,
}

#[derive(Debug)]
pub enum ConstParameter<'s> {
    Generic(&'s str),
}

#[derive(Debug)]
pub enum AssignmentTarget<'s> {
    Identifier(&'s str),
    Access(Access<'s>)
}

impl<'s> TryFrom<Expression<'s>> for AssignmentTarget<'s> {
    type Error = ();

    fn try_from(value: Expression<'s>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct Access<'s> {
    pub target: Box<Span<Expression<'s>>>,
    pub property: &'s str,
}

#[derive(Debug)]
pub struct MarkupElement<'s> {
    pub identifier: &'s str,
    pub attributes: Vec<(&'s str, Expression<'s>)>,
    pub children: Vec<MarkupChild<'s>>,
}

#[derive(Debug)]
pub enum MarkupChild<'s> {
    Element(MarkupElement<'s>),
    Text(&'s str),
    Insert(Expression<'s>),
}

#[derive(Debug)]
pub struct Parameter<'s> {
    pub identifier: &'s str,
    pub is_mutable: bool,
    pub ty: Option<Type<'s>>,
}

#[derive(Debug)]
pub enum Type<'s> {
    Never,
    Nil,
    Number,
    Boolean,
    String,
    Function(Box<FunctionSignature<'s>>),
    ItemPath {
        generics: Vec<ItemPath<'s>>,
        path: ItemPath<'s>,
    },
}

#[derive(Debug)]
pub struct FunctionSignature<'s> {
    pub return_type: Option<Type<'s>>,
    pub is_async: bool,
    pub parameters: Vec<Parameter<'s>>,
    pub has_this_parameter: bool,
    pub const_parameters: Vec<ConstParameter<'s>>,
}

#[derive(Debug)]
pub struct ItemPath<'s>(pub Vec<&'s str>);