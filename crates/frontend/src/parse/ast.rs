use core::alloc::Allocator;
use core::fmt::Debug;
use core::ops::Range;
use ecow::EcoString;
use span::{Index, Span};

type Box<T, A: Allocator> = alloc::boxed::Box<T, A>;
type Vec<T, A: Allocator> = alloc::vec::Vec<T, A>;

/// A binary operation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    PA(PAOperation),
    Comp(ComparativeOperation),
}

/// An operation that can be assigned like `+=`.
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
    GreaterThanOrEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<'source, A: Allocator> {
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
        value: Box<Span<Expression<'source, A>>, A>,
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
        body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>,
    },

    /// `for variable in iter { body... }`
    For {
        is_mutable: bool,
        variable: &'source str,
        iter: Box<Expression<'source, A>, A>,
        body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>,
    },

    /// `{ ... }`
    Block(Vec<Span<StatementOrExpression<'source, A>>, A>),

    /// `(p0 = v0, p1 = v1, ...)`
    Object(Vec<ObjectField<'source, A>, A>),

    /// `target.property`
    Access(Access<'source, A>),

    /// `target?.property`
    OptionalAccess(Access<'source, A>),

    /// `target[index]`
    ArrayAccess {
        target: Box<Span<Expression<'source, A>>, A>,
        index: Box<Span<Expression<'source, A>>, A>,
    },

    /// `[v0, v1, ...]`
    Array(Vec<Span<Expression<'source, A>>, A>),

    /// `123457657_234234234.11321`
    Number(f64),

    /// `"Hello, World!"`
    String(EcoString),

    /// `identifier`
    Identifier(&'source str),

    /// `this`
    This,

    /// `<element args...> children... </element>`
    Markup(MarkupElement<'source, A>),

    /// `fn(params...) body`
    Function {
        // signature: FunctionSignature<'source, A>,
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
        const_arguments: Vec<Span<ConstArgument<'source, A>>, A>,
    },

    /// `expression as ty`
    As {
        expression: Box<Expression<'source, A>, A>,
        ty: Span<Type<'source, A>>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstParametersCallTarget<'source, A: Allocator> {
    Identifier(&'source str),
    Access(Access<'source, A>),
    OptionalAccess(Access<'source, A>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstArgument<'source, A: Allocator> {
    Type(Type<'source, A>),
    Expression(Expression<'source, A>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectField<'source, A: Allocator> {
    pub field: &'source str,
    pub value: Span<Expression<'source, A>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CallArguments<'source, A: Allocator> {
    Single(Box<Span<Expression<'source, A>>, A>),
    Named(Vec<(Span<&'source str>, Span<Expression<'source, A>>), A>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct If<'source, A: Allocator> {
    pub condition: Box<Span<Expression<'source, A>>, A>,
    pub body: Span<Vec<Span<StatementOrExpression<'source, A>>, A>>,
}

pub type ConstParameters<'source, A: Allocator> = Vec<Span<ConstParameter<'source, A>>, A>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThisParameter {
    This,
    ThisMut,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind<'source, A: Allocator> {
    Type {
        const_parameters: ConstParameters<'source, A>,
        id: &'source str,
        ty: Span<Type<'source, A>>,
    },
    Enum {
        const_parameters: ConstParameters<'source, A>,
        id: &'source str,
        variants: Vec<(&'source str, Option<Span<Expression<'source, A>>>), A>,
    },
    Alias {
        const_parameters: ConstParameters<'source, A>,
        id: &'source str,
        ty: Type<'source, A>,
    },
    Let {
        is_mutable: bool,
        ty: Span<Type<'source, A>>,
        id: &'source str,
        value: Option<Box<Span<Expression<'source, A>>, A>>,
    },
    Function {
        const_parameters: ConstParameters<'source, A>,
        id: &'source str,
        pattern: (), // TODO
        this_parameter: Option<ThisParameter>,
        input_type: Span<Type<'source, A>>,
        output_type: Span<Type<'source, A>>,
        body: Box<Span<Expression<'source, A>>, A>,
    },
    Use(Use<'source, A>),
    RootUse(UseChild<'source, A>),
    Module {
        id: &'source str,
        content: Option<ModuleContent<'source, A>>,
    },
    Break,
    Continue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstParameter<'source, A: Allocator> {
    Type {
        id: &'source str,
        trait_bounds: Vec<&'source A, A>,
    },
    Let {
        id: &'source str,
        ty: Type<'source, A>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'source, A: Allocator> {
    pub path: Span<ItemPath<'source, A>>,
    pub arguments: Vec<Span<Expression<'source, A>>, A>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'source, A: Allocator> {
    pub annotations: Vec<Annotation<'source, A>, A>,
    pub statement_kind: StatementKind<'source, A>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementOrExpression<'source, A: Allocator> {
    Statement(Statement<'source, A>),
    Expression(Expression<'source, A>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleContent<'source, A: Allocator>(pub Vec<TopLevelItem<'source, A>, A>);

#[derive(Debug, PartialEq, Clone)]
pub struct TopLevelItem<'source, A: Allocator> {
    pub is_public: bool,
    pub statement: Span<Statement<'source, A>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Use<'source, A: Allocator> {
    pub id: &'source str,
    pub child: Option<Span<UseChild<'source, A>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UseChild<'source, A: Allocator> {
    Single(Box<Use<'source, A>, A>),
    Multiple(Vec<Span<Use<'source, A>>, A>),
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentTarget<'source, A: Allocator> {
    Identifier(&'source str),
    Access(Access<'source, A>),
}

impl<'source, A: Allocator> TryFrom<Expression<'source, A>> for AssignmentTarget<'source, A> {
    type Error = ();

    fn try_from(value: Expression<'source, A>) -> Result<Self, Self::Error> {
        match value {
            Expression::Access(access) => Ok(Self::Access(access)),
            Expression::Identifier(identifier) => Ok(Self::Identifier(identifier)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access<'source, A: Allocator> {
    pub target: Box<Span<Expression<'source, A>>, A>,
    pub property: &'source str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkupElement<'source, A: Allocator> {
    pub identifier: &'source str,
    pub attributes: Vec<(&'source str, Expression<'source, A>), A>,
    pub children: Vec<MarkupChild<'source, A>, A>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkupChild<'source, A: Allocator> {
    Element(MarkupElement<'source, A>),
    Text(&'source str),
    Insert(Expression<'source, A>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'source, A: Allocator> {
    pub id: &'source str,
    pub is_mutable: bool,
    pub ty: Span<Type<'source, A>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'source, A: Allocator> {
    /// A type describing a value that will never exist.
    Never,

    /// Indicates that the type will be inferred.
    Inferred,

    /// An item (path).
    Item(ItemRef<'source, A>),

    /// An object.
    Object(Vec<ObjectTypeField<'source, A>, A>),

    /// A union of two types, `Type1 | Type2`.
    Union {
        left: Box<Span<Type<'source, A>>, A>,
        right: Box<Span<Type<'source, A>>, A>,
    },

    /// A function from one type to another, `Type1 -> Type2`
    Function {
        input: Box<Span<Type<'source, A>>, A>,
        output: Box<Span<Type<'source, A>>, A>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectTypeField<'source, A: Allocator> {
    pub is_public: bool,
    pub is_mutable: bool,
    pub id: &'source str,
    pub ty: Span<Type<'source, A>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemRef<'source, A: Allocator> {
    pub path: Span<ItemPath<'source, A>>,
    pub const_parameters: Span<Vec<Span<Type<'source, A>>, A>>,
}

impl<'source, A: Allocator> ItemRef<'source, A> {
    #[inline]
    pub const fn source(&self) -> Range<Index> {
        self.path.source.start..self.const_parameters.source.end
    }
}

/// A struct containing information about type parameters,
/// parameters and the return-type of a function.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature<'source, A: Allocator> {
    pub const_parameters: ConstParameters<'source, A>,
    pub parameters: Span<Type<'source, A>>,
    pub return_type: Option<Span<Type<'source, A>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemPath<'source, A: Allocator> {
    pub parents: Vec<&'source str, A>,
    pub id: &'source str,
}

// TODO: patterns
