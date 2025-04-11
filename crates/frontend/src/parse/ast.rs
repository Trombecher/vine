use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::Range;
use ecow::EcoString;
use span::{Index, Span};

/// A binary operation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    PA(PAOperation),
    Comp(ComparativeOperation),
}

/// An operation that can be assigned, like `+=`.
#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
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
pub enum Expression<'source> {
    /// `left operation right`
    Operation {
        left: Box<Span<Expression<'source>>>,
        operation: Operation,
        right: Box<Span<Expression<'source>>>,
    },

    /// `target operation value`
    Assignment {
        target: Box<Span<AssignmentTarget<'source>>>,
        operation: Option<PAOperation>,
        value: Box<Span<Expression<'source>>>
    },
    
    /// `!expr`
    Not(Box<Span<Expression<'source>>>),

    /// `return expr`
    Return(Box<Span<Expression<'source>>>),

    /// `continue`
    Continue,

    /// `break`
    Break,

    /// ```plaintext
    /// if condition { ... }
    /// if condition { ... } else { ...}
    /// if condition { ... } else if c1 { ... }
    /// if condition { ... } else if c1 { ... } else { ...}
    /// ...
    /// ```
    If {
        base: If<'source>,
        else_ifs: Vec<If<'source>>,
        else_body: Option<Span<Vec<Span<StatementOrExpression<'source>>>>>,
    },

    /// `while condition { body... }`
    While {
        condition: Box<Span<Expression<'source>>>,
        body: Span<Vec<Span<StatementOrExpression<'source>>>>
    },

    /// `for variable in iter { body... }`
    For {
        is_mutable: bool,
        variable: &'source str,
        iter: Box<Expression<'source>>,
        body: Span<Vec<Span<StatementOrExpression<'source>>>>
    },

    /// `{ ... }`
    Block(Vec<Span<StatementOrExpression<'source>>>),
    
    /// `(p0 = v0, p1 = v1, ...)`
    Instance(Vec<InstanceFieldInit<'source>>),

    /// `target.property`
    Access(Access<'source>),

    /// `target?.property`
    OptionalAccess(Access<'source>),

    /// `target[index]`
    ArrayAccess {
        target: Box<Expression<'source>>,
        index: Box<Expression<'source>>
    },

    /// `[v0, v1, ...]`
    Array(Vec<Span<Expression<'source>>>),

    /// `123457657_234234234.11321`
    Number(f64),

    /// `"Hello, World!"`
    String(EcoString),

    /// `identifier`
    Identifier(&'source str),

    /// `false`
    False,

    /// `true`
    True,

    /// `this`
    This,

    /// `<element args...> children... </element>`
    Markup(MarkupElement<'source>),
    
    /// `fn(params...) body`
    Function {
        signature: FunctionSignature<'source>,
        body: Box<Span<Expression<'source>>>,
    },

    /// `target(args...)`
    Call {
        target: Box<Span<Expression<'source>>>,
        arguments: CallArguments<'source>,
    },

    /// `target.<const_args...>(args...)`
    CallWithConstParameters {
        target: Box<Span<ConstParametersCallTarget<'source>>>,
        arguments: CallArguments<'source>,
        const_arguments: Vec<Span<ConstArgument<'source>>>
    },
    
    /// `expression as ty`
    As {
        expression: Box<Expression<'source>>,
        ty: Span<Type<'source>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstParametersCallTarget<'source> {
    Identifier(&'source str),
    Access(Access<'source>),
    OptionalAccess(Access<'source>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstArgument<'source> {
    Type(Type<'source>),
    Expression(Expression<'source>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstanceFieldInit<'source> {
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Option<Span<Type<'source>>>,
    pub init: Span<Expression<'source>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'source> {
    Single(Box<Span<Expression<'source>>>),
    Named(Vec<(Span<&'source str>, Span<Expression<'source>>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'source> {
    pub condition: Box<Span<Expression<'source>>>,
    pub body: Span<Vec<Span<StatementOrExpression<'source>>>>,
}

pub type ConstParameters<'source> = Vec<Span<ConstParameter<'source>>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThisParameter {
    This,
    ThisMut
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'source> {
    Enum {
        doc_comments: Vec<&'source str>,
        id: &'source str,
        const_parameters: ConstParameters<'source>,
        variants: Vec<(&'source str, Option<Span<Expression<'source>>>)>,
    },
    Declaration {
        doc_comments: Vec<&'source str>,
        is_mutable: bool,
        ty: Option<Span<Type<'source>>>,
        id: &'source str,
        value: Option<Box<Span<Expression<'source>>>>,
    },
    Function {
        signature: FunctionSignature<'source>,
        id: &'source str,
        this_parameter: Option<ThisParameter>,
        body: Box<Span<Expression<'source>>>,
    },
    Struct {
        doc_comments: Vec<&'source str>,
        id: &'source str,
        const_parameters: ConstParameters<'source>,
        fields: Vec<Span<StructField<'source>>>
    },
    TypeAlias {
        doc_comments: Vec<&'source str>,
        id: &'source str,
        const_parameters: ConstParameters<'source>,
        ty: Type<'source>,
    },
    Use(Use<'source>),
    RootUse(UseChild<'source>),
    Module {
        doc_comments: Vec<&'source str>,
        id: &'source str,
        content: Option<ModuleContent<'source>>
    },
    Break,
    Continue
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstParameter<'source> {
    Type {
        id: &'source str,
        trait_bounds: Vec<RawType<'source>>
    },
    Let {
        id: &'source str,
        ty: Type<'source>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField<'source> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Span<Type<'source>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'source> {
    pub path: Span<ItemPath<'source>>,
    pub arguments: Vec<Span<Expression<'source>>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'source> {
    pub annotations: Vec<Annotation<'source>>,
    pub statement_kind: StatementKind<'source>
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'source> {
    Statement(Statement<'source>),
    Expression(Expression<'source>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContent<'source>(pub Vec<TopLevelItem<'source>>);

#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'source> {
    pub is_public: bool,
    pub statement: Span<Statement<'source>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'source> {
    pub id: &'source str,
    pub child: Option<Span<UseChild<'source>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'source> {
    Single(Box<Use<'source>>),
    Multiple(Vec<Span<Use<'source>>>),
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'source> {
    Identifier(&'source str),
    Access(Access<'source>)
}

impl<'source> TryFrom<Expression<'source>> for AssignmentTarget<'source> {
    type Error = ();

    fn try_from(value: Expression<'source>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access<'source> {
    pub target: Box<Span<Expression<'source>>>,
    pub property: &'source str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkupElement<'source> {
    pub identifier: &'source str,
    pub attributes: Vec<(&'source str, Expression<'source>)>,
    pub children: Vec<MarkupChild<'source>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkupChild<'source> {
    Element(MarkupElement<'source>),
    Text(&'source str),
    Insert(Expression<'source>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'source> {
    pub id: &'source str,
    pub is_mutable: bool,
    pub ty: Span<Type<'source>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'source> {
    Never,
    Union {
        first: RawType<'source>, // Ensures the union has at least one RawType
        remaining: Vec<RawType<'source>>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawType<'source> {
    Function(Span<Box<FunctionSignature<'source>>>),
    Item(ItemRef<'source>)
}

impl<'source> RawType<'source> {
    #[inline]
    pub fn source_span(&self) -> Range<Index> {
        match self {
            RawType::Function(Span { source, ..}) => source.clone(),
            RawType::Item(item_ref) => item_ref.source_span()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemRef<'source> {
    pub path: Span<ItemPath<'source>>,
    pub const_parameters: Span<Vec<Span<Type<'source>>>>,
}

impl<'source> ItemRef<'source> {
    #[inline]
    pub const fn source_span(&self) -> Range<Index> {
        self.path.source.start..self.const_parameters.source.end
    }
}

/// A struct containing information about type parameters,
/// parameters and the return-type of a function.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'source> {
    pub const_parameters: ConstParameters<'source>,
    pub parameters: Vec<Parameter<'source>>,
    pub return_type: Option<Span<Type<'source>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'source> {
    pub parents: Vec<&'source str>,
    pub id: &'source str
}