use core::fmt::Debug;
use core::ops::Range;
use bytes::{Index, Span};
use crate::{Box, Vec};
use crate::lex::BoxStr;

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
pub enum Expression<'source, 'alloc: 'alloc> {
    // Binary Expressions
    Operation {
        left: Box<'alloc, Span<Expression<'source, 'alloc>>>,
        operation: Operation,
        right: Box<'alloc, Span<Expression<'source, 'alloc>>>,
    },
    Assignment {
        target: Box<'alloc, Span<AssignmentTarget<'source, 'alloc>>>,
        operation: Option<PAOperation>,
        value: Box<'alloc, Span<Expression<'source, 'alloc>>>
    },
    
    // Unary Expressions
    Not(Box<'alloc, Span<Expression<'source, 'alloc>>>),
    Return(Box<'alloc, Span<Expression<'source, 'alloc>>>),
    
    // Control Flow
    Continue,
    Break,
    If {
        base: If<'source, 'alloc>,
        else_ifs: Vec<'alloc, If<'source, 'alloc>>,
        else_body: Option<Span<Vec<'alloc, Span<StatementOrExpression<'source, 'alloc>>>>>,
    },
    While {
        condition: Box<'alloc, Span<Expression<'source, 'alloc>>>,
        body: Span<Vec<'alloc, Span<StatementOrExpression<'source, 'alloc>>>>
    },
    For {
        is_mutable: bool,
        variable: &'source str,
        iter: Box<'alloc, Expression<'source, 'alloc>>,
    },
    Block(Vec<'alloc, Span<StatementOrExpression<'source, 'alloc>>>),
    
    // Objects And Paths
    Instance(Vec<'alloc, InstanceFieldInit<'source, 'alloc>>),
    Access(Access<'source, 'alloc>),
    OptionalAccess(Access<'source, 'alloc>),
    Array(Vec<'alloc, Span<Expression<'source, 'alloc>>>),

    // Primitives
    Number(f64),
    String(BoxStr<'alloc>),
    Identifier(&'source str),
    False,
    True,
    This,
    Markup(MarkupElement<'source, 'alloc>),
    
    // Functions
    Function {
        signature: FunctionSignature<'source, 'alloc>,
        body: Box<'alloc, Span<Expression<'source, 'alloc>>>,
    },
    Call {
        target: Box<'alloc, Span<Expression<'source, 'alloc>>>,
        arguments: CallArguments<'source, 'alloc>
    },
    As {
        ty: Span<Type<'source, 'alloc>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstanceFieldInit<'source, 'alloc> {
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Option<Span<Type<'source, 'alloc>>>,
    pub init: Span<Expression<'source, 'alloc>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'source, 'alloc> {
    Unnamed(Vec<'alloc, Span<Expression<'source, 'alloc>>>),
    Named(Vec<'alloc, (Span<&'source str>, Span<Expression<'source, 'alloc>>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'source, 'alloc> {
    pub condition: Box<'alloc, Span<Expression<'source, 'alloc>>>,
    pub body: Span<Vec<'alloc, Span<StatementOrExpression<'source, 'alloc>>>>,
}

pub type TypeParameters<'source, 'alloc> = Vec<'alloc, Span<TypeParameter<'source, 'alloc>>>;

#[derive(Debug, PartialEq, Clone)]
pub enum ThisParameter {
    This,
    ThisMut
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'source, 'alloc> {
    Enum {
        doc_comments: Vec<'alloc, &'source str>,
        id: &'source str,
        tps: TypeParameters<'source, 'alloc>,
        variants: Vec<'alloc, (&'source str, Option<Span<Expression<'source, 'alloc>>>)>,
    },
    Declaration {
        doc_comments: Vec<'alloc, &'source str>,
        is_mutable: bool,
        ty: Option<Span<Type<'source, 'alloc>>>,
        id: &'source str,
        value: Option<Box<'alloc, Span<Expression<'source, 'alloc>>>>,
    },
    Function {
        signature: FunctionSignature<'source, 'alloc>,
        id: &'source str,
        this_parameter: Option<ThisParameter>,
        body: Box<'alloc, Span<Expression<'source, 'alloc>>>,
    },
    Struct {
        doc_comments: Vec<'alloc, &'source str>,
        id: &'source str,
        tps: TypeParameters<'source, 'alloc>,
        fields: Vec<'alloc, Span<StructField<'source, 'alloc>>>
    },
    TypeAlias {
        doc_comments: Vec<'alloc, &'source str>,
        id: &'source str,
        tps: TypeParameters<'source, 'alloc>,
        ty: Type<'source, 'alloc>,
    },
    Use(Use<'source, 'alloc>),
    RootUse(UseChild<'source, 'alloc>),
    Module {
        doc_comments: Vec<'alloc, &'source str>,
        id: &'source str,
        content: Option<ModuleContent<'source, 'alloc>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter<'source, 'alloc> {
    pub id: &'source str,
    pub traits: Vec<'alloc, ItemPath<'source, 'alloc>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField<'source, 'alloc> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Span<Type<'source, 'alloc>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'source, 'alloc> {
    pub path: Span<ItemPath<'source, 'alloc>>,
    pub arguments: Vec<'alloc, Span<Expression<'source, 'alloc>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'source, 'alloc> {
    pub annotations: Vec<'alloc, Annotation<'source, 'alloc>>,
    pub statement_kind: StatementKind<'source, 'alloc>
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'source, 'alloc> {
    Statement(Statement<'source, 'alloc>),
    Expression(Expression<'source, 'alloc>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContent<'source, 'alloc>(pub Vec<'alloc, TopLevelItem<'source, 'alloc>>);

#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'source, 'alloc> {
    pub is_public: bool,
    pub statement: Span<Statement<'source, 'alloc>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'source, 'alloc> {
    pub id: &'source str,
    pub child: Option<Span<UseChild<'source, 'alloc>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'source, 'alloc> {
    Single(Box<'alloc, Use<'source, 'alloc>>),
    Multiple(Vec<'alloc, Span<Use<'source, 'alloc>>>),
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'source, 'alloc> {
    Identifier(&'source str),
    Access(Access<'source, 'alloc>)
}

impl<'source, 'alloc> TryFrom<Expression<'source, 'alloc>> for AssignmentTarget<'source, 'alloc> {
    type Error = ();

    fn try_from(value: Expression<'source, 'alloc>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access<'source, 'alloc> {
    pub target: Box<'alloc, Span<Expression<'source, 'alloc>>>,
    pub property: &'source str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkupElement<'source, 'alloc> {
    pub identifier: &'source str,
    pub attributes: Vec<'alloc, (&'source str, Expression<'source, 'alloc>)>,
    pub children: Vec<'alloc, MarkupChild<'source, 'alloc>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkupChild<'source, 'alloc> {
    Element(MarkupElement<'source, 'alloc>),
    Text(&'source str),
    Insert(Expression<'source, 'alloc>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'source, 'alloc> {
    pub id: &'source str,
    pub is_mutable: bool,
    pub ty: Span<Type<'source, 'alloc>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'source, 'alloc> {
    Never,
    Union {
        first: RawType<'source, 'alloc>, // Ensures the union has at least one RawType
        remaining: Vec<'alloc, RawType<'source, 'alloc>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawType<'source, 'alloc> {
    Function(Span<Box<'alloc, FunctionSignature<'source, 'alloc>>>),
    Item(ItemRef<'source, 'alloc>)
}

impl<'source, 'alloc> RawType<'source, 'alloc> {
    #[inline]
    pub fn source_span(&self) -> Range<Index> {
        match self {
            RawType::Function(Span { source, ..}) => source.clone(),
            RawType::Item(item_ref) => item_ref.source_span()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemRef<'source, 'alloc> {
    pub path: Span<ItemPath<'source, 'alloc>>,
    pub tps: Span<Vec<'alloc, Span<Type<'source, 'alloc>>>>,
}

impl<'source, 'alloc> ItemRef<'source, 'alloc> {
    #[inline]
    pub const fn source_span(&self) -> Range<Index> {
        self.path.source.start..self.tps.source.end
    }
}

/// A struct containing information about type parameters,
/// parameters and the return-type of a function.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'source, 'alloc> {
    pub tps: TypeParameters<'source, 'alloc>,
    pub parameters: Vec<'alloc, Parameter<'source, 'alloc>>,
    pub return_type: Option<Span<Type<'source, 'alloc>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'source, 'alloc> {
    pub parents: Vec<'alloc, &'source str>,
    pub id: &'source str
}