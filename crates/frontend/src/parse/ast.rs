use core::alloc::Allocator;
use core::fmt::Debug;
use core::ops::Range;
use derive_where::derive_where;
use ecow::EcoString;
use span::{Index, Span};

type Box<T, A> = alloc::boxed::Box<T, A>;
type Vec<T, A> = alloc::vec::Vec<T, A>;

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

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
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
        condition: Box<Span<Expression<'source, A>>, A>,
        then_expr: Box<Span<Expression<'source, A>>, A>,
        else_expr: Option<Box<Span<Expression<'source, A>>, A>>
    },

    /// `while condition { body... }`
    While {
        condition: Box<Span<Expression<'source, A>>, A>,
        body: Box<Span<Expression<'source, A>>, A>,
    },

    /// `for variable in iter { ... }`
    For {
        is_mutable: bool,
        variable: &'source str,
        iter: Box<Expression<'source, A>, A>,
        body: Span<Vec<StatementOrExpression<'source, A>, A>>,
    },

    /// `{ ... }`
    Block(Vec<StatementOrExpression<'source, A>, A>),

    /// `(p0 = v0, p1 = v1, ...)`
    Object(Vec<ObjectField<'source, A>, A>),

    /// `target.property`
    Access(Access<'source, A>),

    /// `target?.property`
    OptionalAccess(Access<'source, A>),

    /// `target[index]` TODO
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

    /// `This`
    CapitalThis,

    /// `<element args...> children... </element>`
    Markup(MarkupElement<'source, A>),

    /// `fn(params...) body`
    Function {
        pattern: Pattern<'source, A>,
        return_type: Span<Type<'source, A>>,
        body: Box<Span<Expression<'source, A>>, A>,
    },

    /// `callee argument`
    Call {
        callee: Box<Span<Expression<'source, A>>, A>,
        argument: Box<Span<Expression<'source, A>>, A>,
    },

    /// `target.<const_args...>`
    Refine {
        target: Box<Span<Expression<'source, A>>, A>,
        const_arguments: Vec<Span<ConstArgument<'source, A>>, A>,
    },

    /// `expression as ty`
    As {
        expression: Box<Span<Expression<'source, A>>, A>,
        ty: Span<Type<'source, A>>,
    },

    Match {
        on: Box<Span<Expression<'source, A>>, A>,
        cases: Vec<MatchCase<'source, A>, A>,
    },
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct MatchCase<'source, A: Allocator> {
    pub pattern: Pattern<'source, A>,
    pub expression: Span<Expression<'source, A>>,
}

impl<'source, A: Allocator> MatchCase<'source, A> {
    pub fn source(&self) -> Range<Index> {
        self.pattern.source().start..self.expression.source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum ConstParametersCallTarget<'source, A: Allocator> {
    Identifier(&'source str),
    Access(Access<'source, A>),
    OptionalAccess(Access<'source, A>),
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum ConstArgument<'source, A: Allocator> {
    Type(Type<'source, A>),
    Expression(Expression<'source, A>),
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ObjectField<'source, A: Allocator> {
    pub id: Span<&'source str>,
    pub value: Span<Expression<'source, A>>,
}

impl<'source, A: Allocator> ObjectField<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        self.id.source.start..self.value.source.end
    }
}

pub type ConstParameters<'source, A> = Vec<Span<ConstParameter<'source, A>>, A>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThisParameter {
    This,
    ThisMut,
}

/// In which scope an item is visible.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Visible {
    /// The item is visible to external code.
    Public,

    /// The item is visible only within the current package.
    Package,

    /// The item is visible only within the current module.
    Module,
}

/// Visibility of an item, `None` if the item is only visible to the binding type.
pub type Visibility = Option<Span<Visible>>;

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum StatementKind<'source, A: Allocator> {
    For {
        pattern: Pattern<'source, A>,
        iter: Span<Expression<'source, A>>,
        body: Span<Expression<'source, A>>
    },
    Type {
        const_parameters: ConstParameters<'source, A>,
        id: Span<&'source str>,
        ty_visibility: Visibility,
        ty_is_mutable: bool,
        ty: Span<Type<'source, A>>,
    },
    Enum {
        const_parameters: ConstParameters<'source, A>,
        id: Span<&'source str>,
        variants: Vec<(Span<&'source str>, Option<Span<Expression<'source, A>>>), A>,
    },
    Alias {
        const_parameters: ConstParameters<'source, A>,
        id: &'source str,
        ty: Type<'source, A>,
    },
    Let {
        pattern: Pattern<'source, A>,
        value: Option<Span<Expression<'source, A>>>,
    },
    Function {
        const_parameters: ConstParameters<'source, A>,
        id: Span<&'source str>,
        pattern: Pattern<'source, A>,
        output_type: Span<Type<'source, A>>,
        body: Span<Vec<StatementOrExpression<'source, A>, A>>,
    },
    Use(Use<'source, A>),
    RootUse(UseChild<'source, A>),
    Module {
        id: Span<&'source str>,
        content: Option<ModuleContent<'source, A>>,
    },
    Impl {
        const_parameters: ConstParameters<'source, A>,
        what: Span<Type<'source, A>>,
        for_type: Span<Type<'source, A>>,
        body: Span<Vec<StatementOrExpression<'source, A>, A>>,
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum ConstParameter<'source, A: Allocator> {
    Type {
        id: Span<&'source str>,
        trait_bounds: Vec<Span<Type<'source, A>>, A>,
    },
    Let {
        id: Span<&'source str>,
        ty: Span<Type<'source, A>>,
    },
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct Annotation<'source, A: Allocator> {
    pub path: ItemPath<'source, A>,
    pub arguments: Vec<Span<Expression<'source, A>>, A>,
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct Statement<'source, A: Allocator> {
    pub annotations: Vec<Span<Annotation<'source, A>>, A>,
    pub statement_kind: Span<StatementKind<'source, A>>,
}

impl<'source, A: Allocator> Statement<'source, A> {
    pub fn source(&self) -> Range<Index> {
        self.annotations
            .first()
            .map(|a| a.source.start)
            .unwrap_or(self.statement_kind.source.start)..self.statement_kind.source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum StatementOrExpression<'source, A: Allocator> {
    Statement(Statement<'source, A>),
    Expression(Span<Expression<'source, A>>),
}

impl<'source, A: Allocator> StatementOrExpression<'source, A> {
    pub fn source(&self) -> Range<Index> {
        match self {
            Self::Statement(s) => s.source(),
            Self::Expression(e) => e.source.clone(),
        }
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ModuleContent<'source, A: Allocator>(pub Vec<TopLevelItem<'source, A>, A>);

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct TopLevelItem<'source, A: Allocator> {
    pub visibility: Visibility,
    pub statement: Statement<'source, A>,
}

impl<'source, A: Allocator> TopLevelItem<'source, A> {
    pub fn source(&self) -> Range<Index> {
        let stmt_source = self.statement.source();

        self.visibility
            .as_ref()
            .map(|v| v.source.start)
            .unwrap_or(stmt_source.start)..stmt_source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct Use<'source, A: Allocator> {
    pub id: Span<&'source str>,
    pub child: Option<Span<UseChild<'source, A>>>,
}

impl<'source, A: Allocator> Use<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        if let Some(child) = &self.child {
            self.id.source.start..child.source.end
        } else {
            self.id.source.clone()
        }
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum UseChild<'source, A: Allocator> {
    Single(Box<Use<'source, A>, A>),
    Multiple(Vec<Use<'source, A>, A>),
    All,
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
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

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct Access<'source, A: Allocator> {
    pub target: Box<Span<Expression<'source, A>>, A>,
    pub property: &'source str,
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct MarkupElement<'source, A: Allocator> {
    pub identifier: &'source str,
    pub attributes: Vec<(&'source str, Expression<'source, A>), A>,
    pub children: Vec<MarkupChild<'source, A>, A>,
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum MarkupChild<'source, A: Allocator> {
    Element(MarkupElement<'source, A>),
    Text(&'source str),
    Insert(Expression<'source, A>),
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum Type<'source, A: Allocator> {
    /// A type describing a value that will never exist.
    Never,

    /// Indicates that the type will be inferred.
    Inferred,

    /// The `This` type, the associated type.
    This,

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

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ObjectTypeField<'source, A: Allocator> {
    pub visibility: Visibility,
    pub is_mutable: Option<Span<()>>,
    pub id: Span<&'source str>,
    pub ty: Span<Type<'source, A>>,
}

impl<'source, A: Allocator> ObjectTypeField<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        self.visibility
            .as_ref()
            .map(|x| x.source.start)
            .or_else(|| self.is_mutable.as_ref().map(|x| x.source.start))
            .unwrap_or_else(|| self.ty.source.start)..self.ty.source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ItemRef<'source, A: Allocator> {
    pub path: ItemPath<'source, A>,
    pub const_parameters: Span<Vec<Span<Type<'source, A>>, A>>,
}

impl<'source, A: Allocator> ItemRef<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        self.path.source().start..self.const_parameters.source.end
    }
}

/// A struct containing information about type parameters,
/// parameters and the return-type of a function.
#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct FunctionSignature<'source, A: Allocator> {
    pub const_parameters: ConstParameters<'source, A>,
    pub parameters: Span<Type<'source, A>>,
    pub return_type: Option<Span<Type<'source, A>>>,
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ItemPath<'source, A: Allocator> {
    /// The parents of id, last in Vec is first in the path.
    pub parents: Vec<Span<&'source str>, A>,
    pub id: Span<&'source str>,
}

impl<'source, A: Allocator> ItemPath<'source, A> {
    pub fn source(&self) -> Range<Index> {
        self.parents
            .last()
            .map(|parent| parent.source.start)
            .unwrap_or(self.id.source.start)..self.id.source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum PatternUnit<'source, A: Allocator> {
    Any(Span<()>),
    Identifier {
        is_mutable: Option<Span<()>>,
        id: Span<&'source str>,
    },
    Object(Span<Vec<ObjectPatternField<'source, A>, A>>),
    Array(Span<Vec<Pattern<'source, A>, A>>),
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct ObjectPatternField<'source, A: Allocator> {
    pub id: Span<&'source str>,
    pub remap: Pattern<'source, A>,
}

impl<'source, A: Allocator> ObjectPatternField<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        self.id.source.start..self.remap.source().end
    }
}

impl<'source, A: Allocator> PatternUnit<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        match self {
            PatternUnit::Any(s) => s.source.clone(),
            PatternUnit::Identifier { is_mutable, id } => {
                is_mutable
                    .as_ref()
                    .map(|t| t.source.start)
                    .unwrap_or(id.source.start)..id.source.end
            }
            PatternUnit::Object(object) => object.source.clone(),
            PatternUnit::Array(array) => array.source.clone(),
        }
    }
}

/// A pattern with a type: `<pattern_unit> <ty>`
#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub struct PatternUnitWithType<'source, A: Allocator> {
    pub unit: PatternUnit<'source, A>,
    pub ty: Span<Type<'source, A>>,
}

impl<'source, A: Allocator> PatternUnitWithType<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        self.unit.source().start..self.ty.source.end
    }
}

#[derive(Clone)]
#[derive_where(Debug, PartialEq)]
pub enum Pattern<'source, A: Allocator> {
    WithType(PatternUnitWithType<'source, A>),

    /// Infamous `x @ <pattern>` syntax.
    Attach {
        is_mutable: Option<Span<()>>,
        id: Span<&'source str>,
        pattern: Box<Pattern<'source, A>, A>,
    },
}

impl<'source, A: Allocator> Pattern<'source, A> {
    #[inline]
    pub fn source(&self) -> Range<Index> {
        match self {
            Pattern::WithType(pt) => pt.source(),
            Pattern::Attach {
                id,
                pattern,
                is_mutable,
            } => {
                is_mutable
                    .as_ref()
                    .map(|x| x.source.start)
                    .unwrap_or_else(|| id.source.start)..pattern.source().end
            }
        }
    }

    /// Splits `x A -> B` into `(x A, B)`.
    pub fn lift_function_return_type(self) -> (Self, Span<Type<'source, A>>)
    where
        A: Clone,
    {
        match self {
            Pattern::WithType(PatternUnitWithType {
                ty:
                    Span {
                        value: Type::Function { input, output },
                        ..
                    },
                unit,
            }) => (
                Pattern::WithType(PatternUnitWithType { ty: *input, unit }),
                *output,
            ),
            Pattern::Attach {
                pattern,
                id,
                is_mutable,
            } => {
                let alloc = Box::allocator(&pattern).clone();
                let (pattern, ty) = (*pattern).lift_function_return_type();

                (
                    Self::Attach {
                        is_mutable,
                        id,
                        pattern: Box::new_in(pattern, alloc),
                    },
                    ty,
                )
            }
            pattern => {
                let end = pattern.source().end;

                (
                    pattern,
                    Span {
                        source: end..end,
                        value: Type::Inferred,
                    },
                )
            }
        }
    }
}
