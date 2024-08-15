use std::ops::Range;
use lex::{Index, Span};

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
        left: Box<Span<Expression<'a>>>,
        operation: Operation,
        right: Box<Span<Expression<'a>>>,
    },
    Assignment {
        target: Box<Span<AssignmentTarget<'a>>>,
        operation: Option<PAOperation>,
        value: Box<Span<Expression<'a>>>
    },
    
    // Unary Expressions
    Not(Box<Span<Expression<'a>>>),
    Return(Box<Span<Expression<'a>>>),
    
    // Control Flow
    Continue,
    Break,
    If {
        base: If<'a>,
        else_ifs: Vec<If<'a>>,
        else_body: Option<Span<Vec<Span<StatementOrExpression<'a>>>>>,
    },
    While {
        condition: Box<Span<Expression<'a>>>,
        body: Span<Vec<Span<StatementOrExpression<'a>>>>
    },
    For {
        is_mutable: bool,
        variable: &'a str,
        iter: Box<Expression<'a>>,
    },
    Block(Vec<Span<StatementOrExpression<'a>>>),
    
    // Objects And Paths
    Instance(Vec<InstanceFieldInit<'a>>),
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
        body: Box<Span<Expression<'a>>>,
    },
    Call {
        target: Box<Span<Expression<'a>>>,
        arguments: CallArguments<'a>
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstanceFieldInit<'a> {
    pub is_mutable: bool,
    pub id: &'a str,
    pub ty: Option<Span<Type<'a>>>,
    pub init: Span<Expression<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'a> {
    Unnamed(Vec<Span<Expression<'a>>>),
    Named(Vec<(Span<&'a str>, Span<Expression<'a>>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'a> {
    pub condition: Box<Span<Expression<'a>>>,
    pub body: Span<Vec<Span<StatementOrExpression<'a>>>>,
}

pub type TypeParameters<'a> = Vec<Span<TypeParameter<'a>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'a> {
    TypeParameterAlias {
        tps: TypeParameters<'a>,
        content: ModuleContent<'a>
    },
    Enum {
        doc_comments: Vec<&'a str>,
        id: &'a str,
        tps: TypeParameters<'a>,
        variants: Vec<(&'a str, Option<Span<Expression<'a>>>)>,
    },
    Declaration {
        doc_comments: Vec<&'a str>,
        is_mutable: bool,
        ty: Option<Span<Type<'a>>>,
        id: &'a str,
        value: Option<Box<Span<Expression<'a>>>>,
    },
    Struct {
        doc_comments: Vec<&'a str>,
        id: &'a str,
        tps: TypeParameters<'a>,
        fields: Vec<Span<StructField<'a>>>
    },
    TypeAlias {
        doc_comments: Vec<&'a str>,
        id: &'a str,
        tps: TypeParameters<'a>,
        ty: Type<'a>,
    },
    Use(Use<'a>),
    RootUse(UseChild<'a>),
    Module {
        doc_comments: Vec<&'a str>,
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
    pub ty: Span<Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'a> {
    pub path: Span<ItemPath<'a>>,
    pub arguments: Vec<Span<Expression<'a>>>
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
    pub statement: Span<Statement<'a>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'a> {
    pub id: &'a str,
    pub child: Option<Span<UseChild<'a>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'a> {
    Single(Box<Use<'a>>),
    Multiple(Vec<Span<Use<'a>>>),
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
    pub target: Box<Span<Expression<'a>>>,
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
    pub ty: Span<Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    Never,
    Union {
        first: RawType<'a>, // Ensures the union has at least one RawType
        remaining: Vec<RawType<'a>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawType<'a> {
    Function(Span<Box<FunctionSignature<'a>>>),
    Item(ItemRef<'a>)
}

impl<'a> RawType<'a> {
    #[inline]
    pub fn source_span(&self) -> Range<Index> {
        match self {
            RawType::Function(Span { source, ..}) => source.clone(),
            RawType::Item(item_ref) => item_ref.source_span()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemRef<'a> {
    pub path: Span<ItemPath<'a>>,
    pub tps: Span<Vec<Span<Type<'a>>>>,
}

impl<'a> ItemRef<'a> {
    #[inline]
    pub const fn source_span(&self) -> Range<Index> {
        self.path.source.start..self.tps.source.end
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'a> {
    pub return_type: Option<Span<Type<'a>>>,
    pub parameters: Vec<Parameter<'a>>,
    pub has_this_parameter: bool,
    pub tps: TypeParameters<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'a> {
    pub parents: Vec<&'a str>,
    pub id: &'a str
}