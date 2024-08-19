use std::ops::Range;
use bytes::{Index, Span};
use crate::parse::{Box, Vec};

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
pub enum Expression<'sf, 'arena> {
    // Binary Expressions
    Operation {
        left: Box<'arena, Span<Expression<'sf, 'arena>>>,
        operation: Operation,
        right: Box<'arena, Span<Expression<'sf, 'arena>>>,
    },
    Assignment {
        target: Box<'arena, Span<AssignmentTarget<'sf, 'arena>>>,
        operation: Option<PAOperation>,
        value: Box<'arena, Span<Expression<'sf, 'arena>>>
    },
    
    // Unary Expressions
    Not(Box<'arena, Span<Expression<'sf, 'arena>>>),
    Return(Box<'arena, Span<Expression<'sf, 'arena>>>),
    
    // Control Flow
    Continue,
    Break,
    If {
        base: If<'sf, 'arena>,
        else_ifs: Vec<'arena, If<'sf, 'arena>>,
        else_body: Option<Span<Vec<'arena, Span<StatementOrExpression<'sf, 'arena>>>>>,
    },
    While {
        condition: Box<'arena, Span<Expression<'sf, 'arena>>>,
        body: Span<Vec<'arena, Span<StatementOrExpression<'sf, 'arena>>>>
    },
    For {
        is_mutable: bool,
        variable: &'sf str,
        iter: Box<'arena, Expression<'sf, 'arena>>,
    },
    Block(Vec<'arena, Span<StatementOrExpression<'sf, 'arena>>>),
    
    // Objects And Paths
    Instance(Vec<'arena, InstanceFieldInit<'sf, 'arena>>),
    Access(Access<'sf, 'arena>),
    OptionalAccess(Access<'sf, 'arena>),
    Array(Vec<'arena, Expression<'sf, 'arena>>),

    // Primitives
    Number(f64),
    String(String),
    Identifier(&'sf str),
    False,
    True,
    This,
    Markup(MarkupElement<'sf, 'arena>),
    
    // Functions
    Function {
        signature: FunctionSignature<'sf, 'arena>,
        body: Box<'arena, Span<Expression<'sf, 'arena>>>,
    },
    Call {
        target: Box<'arena, Span<Expression<'sf, 'arena>>>,
        arguments: CallArguments<'sf, 'arena>
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstanceFieldInit<'sf, 'arena> {
    pub is_mutable: bool,
    pub id: &'sf str,
    pub ty: Option<Span<Type<'sf, 'arena>>>,
    pub init: Span<Expression<'sf, 'arena>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'sf, 'arena> {
    Unnamed(Vec<'arena, Span<Expression<'sf, 'arena>>>),
    Named(Vec<'arena, (Span<&'sf str>, Span<Expression<'sf, 'arena>>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'sf, 'arena> {
    pub condition: Box<'arena, Span<Expression<'sf, 'arena>>>,
    pub body: Span<Vec<'arena, Span<StatementOrExpression<'sf, 'arena>>>>,
}

pub type TypeParameters<'sf, 'arena> = Vec<'arena, Span<TypeParameter<'sf, 'arena>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'sf, 'arena> {
    TypeParameterAlias {
        tps: TypeParameters<'sf, 'arena>,
        content: ModuleContent<'sf, 'arena>
    },
    Enum {
        doc_comments: Vec<'arena, &'sf str>,
        id: &'sf str,
        tps: TypeParameters<'sf, 'arena>,
        variants: Vec<'arena, (&'sf str, Option<Span<Expression<'sf, 'arena>>>)>,
    },
    Declaration {
        doc_comments: Vec<'arena, &'sf str>,
        is_mutable: bool,
        ty: Option<Span<Type<'sf, 'arena>>>,
        id: &'sf str,
        value: Option<Box<'arena, Span<Expression<'sf, 'arena>>>>,
    },
    Struct {
        doc_comments: Vec<'arena, &'sf str>,
        id: &'sf str,
        tps: TypeParameters<'sf, 'arena>,
        fields: Vec<'arena, Span<StructField<'sf, 'arena>>>
    },
    TypeAlias {
        doc_comments: Vec<'arena, &'sf str>,
        id: &'sf str,
        tps: TypeParameters<'sf, 'arena>,
        ty: Type<'sf, 'arena>,
    },
    Use(Use<'sf, 'arena>),
    RootUse(UseChild<'sf, 'arena>),
    Module {
        doc_comments: Vec<'arena, &'sf str>,
        id: &'sf str,
        content: Option<ModuleContent<'sf, 'arena>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter<'sf, 'arena> {
    pub id: &'sf str,
    pub traits: Vec<'arena, ItemPath<'sf, 'arena>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField<'sf, 'arena> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'sf str,
    pub ty: Span<Type<'sf, 'arena>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'sf, 'arena> {
    pub path: Span<ItemPath<'sf, 'arena>>,
    pub arguments: Vec<'arena, Span<Expression<'sf, 'arena>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'sf, 'arena> {
    pub annotations: Vec<'arena, Annotation<'sf, 'arena>>,
    pub statement_kind: StatementKind<'sf, 'arena>
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'sf, 'arena> {
    Statement(Statement<'sf, 'arena>),
    Expression(Expression<'sf, 'arena>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContent<'sf, 'arena>(pub Vec<'arena, TopLevelItem<'sf, 'arena>>);

#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'sf, 'arena> {
    pub is_public: bool,
    pub statement: Span<Statement<'sf, 'arena>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'sf, 'arena> {
    pub id: &'sf str,
    pub child: Option<Span<UseChild<'sf, 'arena>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'sf, 'arena> {
    Single(Box<'arena, Use<'sf, 'arena>>),
    Multiple(Vec<'arena, Span<Use<'sf, 'arena>>>),
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'sf, 'arena> {
    Identifier(&'sf str),
    Access(Access<'sf, 'arena>)
}

impl<'sf, 'arena> TryFrom<Expression<'sf, 'arena>> for AssignmentTarget<'sf, 'arena> {
    type Error = ();

    fn try_from(value: Expression<'sf, 'arena>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access<'sf, 'arena> {
    pub target: Box<'arena, Span<Expression<'sf, 'arena>>>,
    pub property: &'sf str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkupElement<'sf, 'arena> {
    pub identifier: &'sf str,
    pub attributes: Vec<'arena, (&'sf str, Expression<'sf, 'arena>)>,
    pub children: Vec<'arena, MarkupChild<'sf, 'arena>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkupChild<'sf, 'arena> {
    Element(MarkupElement<'sf, 'arena>),
    Text(&'sf str),
    Insert(Expression<'sf, 'arena>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'sf, 'arena> {
    pub id: &'sf str,
    pub is_mutable: bool,
    pub ty: Span<Type<'sf, 'arena>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'sf, 'arena> {
    Never,
    Union {
        first: RawType<'sf, 'arena>, // Ensures the union has at least one RawType
        remaining: Vec<'arena, RawType<'sf, 'arena>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawType<'sf, 'arena> {
    Function(Span<Box<'arena, FunctionSignature<'sf, 'arena>>>),
    Item(ItemRef<'sf, 'arena>)
}

impl<'sf, 'arena> RawType<'sf, 'arena> {
    #[inline]
    pub fn source_span(&self) -> Range<Index> {
        match self {
            RawType::Function(Span { source, ..}) => source.clone(),
            RawType::Item(item_ref) => item_ref.source_span()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemRef<'sf, 'arena> {
    pub path: Span<ItemPath<'sf, 'arena>>,
    pub tps: Span<Vec<'arena, Span<Type<'sf, 'arena>>>>,
}

impl<'sf, 'arena> ItemRef<'sf, 'arena> {
    #[inline]
    pub const fn source_span(&self) -> Range<Index> {
        self.path.source.start..self.tps.source.end
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'sf, 'arena> {
    pub return_type: Option<Span<Type<'sf, 'arena>>>,
    pub parameters: Vec<'arena, Parameter<'sf, 'arena>>,
    pub has_this_parameter: bool,
    pub tps: TypeParameters<'sf, 'arena>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'sf, 'arena> {
    pub parents: Vec<'arena, &'sf str>,
    pub id: &'sf str
}