use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::fmt::Debug;
use core::ops::Range;
use derive_where::derive_where;
use bytes::{Index, Span};
use crate::lex::BoxStr;

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

#[derive_where(Clone, PartialEq, Debug)]
pub enum Expression<'source, A: Allocator + Copy> {
    /// `left operation right`
    Operation {
        left: Box<Span<Expression<'source, A>>, A>,
        operation: Operation,
        right: Box<Span<Expression<'source, A>>, A>,
    },

    /// `target operation value`
    Assignment {
        target: Box<Span<AssignmentTarget<'source, A>>, A>,
        operation: Option<PAOperation>,
        value: Box<Span<Expression<'source, A>>, A>
    },
    
    /// `!expr`
    Not(Box<Span<Expression<'source, A>>, A>),

    /// `return expr`
    Return(Box<Span<Expression<'source, A>>, A>),

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
        base: If<'source, A>,
        else_ifs: Vec<If<'source, A>, A>,
        else_body: Option<Span<Vec<Span<StatementOrExpression<'source, A>>, A>>>,
    },

    /// `while condition { body... }`
    While {
        condition: Box<Span<Expression<'source, A>>, A>,
        body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>
    },

    /// `for variable in iter { body... }`
    For {
        is_mutable: bool,
        variable: &'source str,
        iter: Box<Expression<'source, A>, A>,
        body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>
    },

    /// `{ ... }`
    Block(Vec<Span<StatementOrExpression<'source, A>>, A>),
    
    /// `(p0 = v0, p1 = v1, ...)`
    Instance(Vec<InstanceFieldInit<'source, A>, A>),

    /// `target.property`
    Access(Access<'source, A>),

    /// `target?.property`
    OptionalAccess(Access<'source, A>),

    /// `target[index]`
    ArrayAccess {
        target: Box<Expression<'source, A>, A>,
        index: Box<Expression<'source, A>, A>
    },

    /// `[v0, v1, ...]`
    Array(Vec<Span<Expression<'source, A>>, A>),

    /// `123457657_234234234.11321`
    Number(f64),

    /// `"Hello, World!"`
    String(BoxStr<A>),

    /// `identifier`
    Identifier(&'source str),

    /// `false`
    False,

    /// `true`
    True,

    /// `this`
    This,

    /// `<element args...> children... </element>`
    Markup(MarkupElement<'source, A>),
    
    /// `fn(params...) body`
    Function {
        signature: FunctionSignature<'source, A>,
        body: Box<Span<Expression<'source, A>>, A>,
    },

    /// `target(args...)`
    Call {
        target: Box<Span<Expression<'source, A>>, A>,
        arguments: CallArguments<'source, A>,
    },

    /// `target.<const_args...>(args...)`
    CallWithConstParameters {
        target: Box<Span<ConstParametersCallTarget<'source, A>>, A>,
        arguments: CallArguments<'source, A>,
        const_arguments: Vec<Span<ConstArgument<'source, A>>, A>
    },
    
    /// `expression as ty`
    As {
        expression: Box<Expression<'source, A>, A>,
        ty: Span<Type<'source, A>>
    }
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum ConstParametersCallTarget<'source, A: Allocator + Copy> {
    Identifier(&'source str),
    Access(Access<'source, A>),
    OptionalAccess(Access<'source, A>),
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum ConstArgument<'source, A: Allocator + Copy> {
    Type(Type<'source, A>),
    Expression(Expression<'source, A>)
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct InstanceFieldInit<'source, A: Allocator + Copy> {
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Option<Span<Type<'source, A>>>,
    pub init: Span<Expression<'source, A>>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum CallArguments<'source, A: Allocator + Copy> {
    Single(Box<Span<Expression<'source, A>>, A>),
    Named(Vec<(Span<&'source str>, Span<Expression<'source, A>>), A>),
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct If<'source, A: Allocator + Copy> {
    pub condition: Box<Span<Expression<'source, A>>, A>,
    pub body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>,
}

pub type ConstParameters<'source, A> = Vec<Span<ConstParameter<'source, A>>, A>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThisParameter {
    This,
    ThisMut
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum StatementKind<'source, A: Allocator + Copy> {
    Enum {
        doc_comments: Vec<&'source str, A>,
        id: &'source str,
        const_parameters: ConstParameters<'source, A>,
        variants: Vec<(&'source str, Option<Span<Expression<'source, A>>>), A>,
    },
    Declaration {
        doc_comments: Vec<&'source str, A>,
        is_mutable: bool,
        ty: Option<Span<Type<'source, A>>>,
        id: &'source str,
        value: Option<Box<Span<Expression<'source, A>>, A>>,
    },
    Function {
        signature: FunctionSignature<'source, A>,
        id: &'source str,
        this_parameter: Option<ThisParameter>,
        body: Box<Span<Expression<'source, A>>, A>,
    },
    Struct {
        doc_comments: Vec<&'source str, A>,
        id: &'source str,
        const_parameters: ConstParameters<'source, A>,
        fields: Vec<Span<StructField<'source, A>>, A>
    },
    TypeAlias {
        doc_comments: Vec<&'source str, A>,
        id: &'source str,
        const_parameters: ConstParameters<'source, A>,
        ty: Type<'source, A>,
    },
    Use(Use<'source, A>),
    RootUse(UseChild<'source, A>),
    Module {
        doc_comments: Vec<&'source str, A>,
        id: &'source str,
        content: Option<ModuleContent<'source, A>>
    },
    Break,
    Continue
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum ConstParameter<'source, A: Allocator + Copy> {
    Type {
        id: &'source str,
        trait_bounds: Vec<RawType<'source, A>, A>
    },
    Let {
        id: &'source str,
        ty: Type<'source, A>
    }
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct StructField<'source, A: Allocator + Copy> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Span<Type<'source, A>>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct Annotation<'source, A: Allocator + Copy> {
    pub path: Span<ItemPath<'source, A>>,
    pub arguments: Vec<Span<Expression<'source, A>>, A>
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct Statement<'source, A: Allocator + Copy> {
    pub annotations: Vec<Annotation<'source, A>, A>,
    pub statement_kind: StatementKind<'source, A>
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'source, A: Allocator + Copy> {
    Statement(Statement<'source, A>),
    Expression(Expression<'source, A>)
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct ModuleContent<'source, A: Allocator + Copy>(pub Vec<TopLevelItem<'source, A>, A>);

#[derive_where(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'source, A: Allocator + Copy> {
    pub is_public: bool,
    pub statement: Span<Statement<'source, A>>
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct Use<'source, A: Allocator + Copy> {
    pub id: &'source str,
    pub child: Option<Span<UseChild<'source, A>>>
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum UseChild<'source, A: Allocator + Copy> {
    Single(Box<Use<'source, A>, A>),
    Multiple(Vec<Span<Use<'source, A>>, A>),
    All,
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'source, A: Allocator + Copy> {
    Identifier(&'source str),
    Access(Access<'source, A>)
}

impl<'source, A: Allocator + Copy> TryFrom<Expression<'source, A>> for AssignmentTarget<'source, A> {
    type Error = ();

    fn try_from(value: Expression<'source, A>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(())
        }
    }
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct Access<'source, A: Allocator + Copy> {
    pub target: Box<Span<Expression<'source, A>>, A>,
    pub property: &'source str,
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct MarkupElement<'source, A: Allocator + Copy> {
    pub identifier: &'source str,
    pub attributes: Vec<(&'source str, Expression<'source, A>), A>,
    pub children: Vec<MarkupChild<'source, A>, A>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum MarkupChild<'source, A: Allocator + Copy> {
    Element(MarkupElement<'source, A>),
    Text(&'source str),
    Insert(Expression<'source, A>),
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct Parameter<'source, A: Allocator + Copy> {
    pub id: &'source str,
    pub is_mutable: bool,
    pub ty: Span<Type<'source, A>>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum Type<'source, A: Allocator + Copy> {
    Never,
    Union {
        first: RawType<'source, A>, // Ensures the union has at least one RawType
        remaining: Vec<RawType<'source, A>, A>
    }
}

#[derive_where(Debug, PartialEq, Clone)]
pub enum RawType<'source, A: Allocator + Copy> {
    Function(Span<Box<FunctionSignature<'source, A>, A>>),
    Item(ItemRef<'source, A>)
}

impl<'source, A: Allocator + Copy> RawType<'source, A> {
    #[inline]
    pub fn source_span(&self) -> Range<Index> {
        match self {
            RawType::Function(Span { source, ..}) => source.clone(),
            RawType::Item(item_ref) => item_ref.source_span()
        }
    }
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct ItemRef<'source, A: Allocator + Copy> {
    pub path: Span<ItemPath<'source, A>>,
    pub const_parameters: Span<Vec<Span<Type<'source, A>>, A>>,
}

impl<'source, A: Allocator + Copy> ItemRef<'source, A> {
    #[inline]
    pub const fn source_span(&self) -> Range<Index> {
        self.path.source.start..self.const_parameters.source.end
    }
}

/// A struct containing information about type parameters,
/// parameters and the return-type of a function.
#[derive_where(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'source, A: Allocator + Copy> {
    pub const_parameters: ConstParameters<'source, A>,
    pub parameters: Vec<Parameter<'source, A>, A>,
    pub return_type: Option<Span<Type<'source, A>>>,
}

#[derive_where(Debug, PartialEq, Clone)]
pub struct ItemPath<'source, A: Allocator + Copy> {
    pub parents: Vec<&'source str, A>,
    pub id: &'source str
}