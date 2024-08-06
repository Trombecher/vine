use lex::Span;
use lex::token::Keyword;

/// A binary operation.
#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    PA(PAOperation),
    Comp(ComparativeOperation),
}

/// An operation that can be assigned, like `+=`.
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
#[repr(u8)]
pub enum ComparativeOperation {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'a> {
    // Binary Expressions
    Operation {
        left: Box<Span<'a, Expression<'a>>>,
        operation: Operation,
        right: Box<Span<'a, Expression<'a>>>,
    },
    Assignment {
        target: Box<Span<'a, AssignmentTarget<'a>>>,
        operation: Option<PAOperation>,
        value: Box<Span<'a, Expression<'a>>>
    },
    
    // Unary Expressions
    Not(Box<Span<'a, Expression<'a>>>),
    Return(Box<Span<'a, Expression<'a>>>),
    
    // Control Flow
    Continue,
    Break,
    If {
        base: If<'a>,
        else_ifs: Vec<If<'a>>,
        else_body: Option<Span<'a, Vec<Span<'a, StatementOrExpression<'a>>>>>,
    },
    While {
        condition: Box<Span<'a, Expression<'a>>>,
        body: Span<'a, Vec<Span<'a, StatementOrExpression<'a>>>>
    },
    For {
        is_mutable: bool,
        variable: &'a str,
        iter: Box<Expression<'a>>,
    },
    Block(Vec<Span<'a, StatementOrExpression<'a>>>),
    
    // Objects And Paths
    Object(Vec<(&'a str, Expression<'a>)>),
    Access(Access<'a>),
    OptionalAccess(Access<'a>),
    Array(Vec<Expression<'a>>),

    // Primitives
    Number(f64),
    String(String),
    Identifier(&'a str),
    False,
    True,
    This,
    Markup(MarkupElement<'a>),
    
    // Functions
    Function {
        signature: FunctionSignature<'a>,
        body: Box<Span<'a, Expression<'a>>>,
    },
    Call {
        target: Box<Span<'a, Expression<'a>>>,
        arguments: CallArguments<'a>
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'a> {
    Unnamed(Vec<Span<'a, Expression<'a>>>),
    Named(Vec<(Span<'a, ()>, Span<'a, Expression<'a>>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'a> {
    pub condition: Box<Span<'a, Expression<'a>>>,
    pub body: Span<'a, Vec<Span<'a, StatementOrExpression<'a>>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'a> {
    Enum {
        id: &'a str,
        tps: Vec<TypeParameter<'a>>,
        variants: Vec<(&'a str, Option<Span<'a, Expression<'a>>>)>,
    },
    Declaration {
        is_mutable: bool,
        ty: Option<Span<'a, Type<'a>>>,
        id: &'a str,
        value: Option<Box<Span<'a, Expression<'a>>>>,
    },
    Struct {
        id: &'a str,
        tps: Vec<TypeParameter<'a>>,
        fields: Vec<Span<'a, StructField<'a>>>
    },
    TypeAlias {
        id: &'a str,
        tps: Vec<TypeParameter<'a>>,
        ty: Type<'a>,
    },
    Use(Use<'a>),
    RootUse(UseChild<'a>),
    Module {
        id: &'a str,
        content: Option<ModuleContent<'a>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter<'a> {
    pub id: &'a str,
    pub traits: Vec<ItemPath<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField<'a> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'a str,
    pub ty: Span<'a, Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'a> {
    pub path: ItemPath<'a>,
    pub arguments: Vec<Span<'a, Expression<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'a> {
    pub annotations: Vec<Annotation<'a>>,
    pub statement_kind: StatementKind<'a>
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'a> {
    Statement(Statement<'a>),
    Expression(Expression<'a>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContent<'a>(pub Vec<TopLevelItem<'a>>);

#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'a> {
    pub is_public: bool,
    pub statement: Span<'a, Statement<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'a> {
    pub id: &'a str,
    pub child: Option<UseChild<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'a> {
    Single(Box<Use<'a>>),
    Multiple(Vec<Use<'a>>),
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'a> {
    Identifier(&'a str),
    Access(Access<'a>)
}

impl<'a> TryFrom<Expression<'a>> for AssignmentTarget<'a> {
    type Error = ();

    fn try_from(value: Expression<'a>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access<'a> {
    pub target: Box<Span<'a, Expression<'a>>>,
    pub property: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkupElement<'a> {
    pub identifier: &'a str,
    pub attributes: Vec<(&'a str, Expression<'a>)>,
    pub children: Vec<MarkupChild<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkupChild<'a> {
    Element(MarkupElement<'a>),
    Text(&'a str),
    Insert(Expression<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'a> {
    pub id: &'a str,
    pub is_mutable: bool,
    pub ty: Span<'a, Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    Never,
    Number,
    Boolean,
    Char,
    Object,
    String,
    Any,
    Nil,
    Function(Box<FunctionSignature<'a>>),
    ItemPath {
        generics: Vec<ItemPath<'a>>,
        path: ItemPath<'a>,
    },
}

impl<'a> TryFrom<Keyword> for Type<'a> {
    type Error = ();

    fn try_from(value: Keyword) -> Result<Self, Self::Error> {
        match value {
            Keyword::Num => Ok(Self::Number),
            Keyword::Str => Ok(Self::String),
            Keyword::Bool => Ok(Self::Boolean),
            Keyword::Char => Ok(Self::Char),
            Keyword::Obj => Ok(Self::Object),
            Keyword::Any => Ok(Self::Any),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'a> {
    pub return_type: Span<'a, Type<'a>>,
    pub parameters: Vec<Parameter<'a>>,
    pub has_this_parameter: bool,
    pub tps: Vec<TypeParameter<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'a>(pub Vec<&'a str>);