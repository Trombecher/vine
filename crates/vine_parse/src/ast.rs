use parser_tools::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'source> {
    Number(u64),

    /// An identifier.
    Identifier(&'source str),

    /// A parenthesized expression
    Parenthesized(Box<Span<Expression<'source>>>),

    Structure {
        /// The fields of the structure. They should be of the form
        /// `identifier = expression`. This will be validated when
        /// lowering.
        fields: Vec<Span<Expression<'source>>>,
    },

    /// A unary Expression<'source>:
    ///
    /// ```plain
    /// <UOP> <EXPR>
    /// ```
    Unary {
        operation: UnaryOperation,
        inner: Box<Span<Expression<'source>>>,
    },

    /// A binary Expression<'source>:
    ///
    /// ```plain
    /// <EXPR> <BOP> <EXPR>
    /// ```
    Binary {
        left: Box<Span<Expression<'source>>>,
        operation: BinaryOperation,
        right: Box<Span<Expression<'source>>>,
    },

    /// An if-Expression<'source>:
    ///
    /// ```plain
    /// if <EXPRESSION>
    ///     then <EXPRESSION>
    ///     [else <EXPRESSION>]
    /// ```
    If {
        /// The condition.
        condition: Box<Span<Expression<'source>>>,

        /// The `then` branch.
        then: Box<Span<Expression<'source>>>,

        /// The `else` branch.
        otherwise: Option<Box<Span<Expression<'source>>>>,
    },

    /// A match Expression<'source>:
    ///
    /// ```plain
    /// match <EXPRESSION>
    ///     <MATCH_CASE>
    ///     [<MATCH_CASE>]*
    /// ```
    Match {
        on: Box<Span<Expression<'source>>>,
        first_case: Span<MatchCase<'source>>,
        other_cases: Vec<Span<MatchCase<'source>>>,
    },

    /// A call expression:
    ///
    /// ```plain
    /// <EXPRESSION> <EXPRESSION>
    /// ```
    Call {
        function: Box<Span<Expression<'source>>>,
        argument: Box<Span<Expression<'source>>>,
    },

    /// A function expression:
    ///
    /// ```plain
    /// function <EXPRESSION> is|in <EXPRESSION> => <EXPRESSION>
    /// ```
    Function {
        parameter_pattern: Box<Span<Expression<'source>>>,
        parameter_domain: Box<Span<Expression<'source>>>,
        body: Box<Span<Expression<'source>>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation {
    /// `-`
    Negate,

    /// `!`
    Not,
}

/// An operation that is used as an infix between
/// two expression.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperation {
    /// `+`
    Add,

    /// `-`
    Subtract,

    /// `*`
    Multiply,

    /// `/`
    Divide,

    /// `=`
    Definition,

    /// `.`
    Access,

    /// `==`
    Equal,

    /// `!=`
    NotEqual,

    /// `<`
    LessThan,

    /// `<=`
    LessThanOrEqual,

    /// `>`
    GreaterThan,

    /// `>=`
    GreaterThanOrEqual,

    /// `..`
    RangeExclusive,

    /// `..=`
    RangeInclusive,

    /// `or`
    Or,

    /// `and`
    And,
}

/// A match case:
///
/// ```plain
/// case <EXPRESSION> [is|in <EXPRESSION>] => <EXPRESSION>
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase<'source> {
    /// The pattern to match against. (This is really an
    /// expression but will be checked if it is a pattern
    /// in the next source tree.)
    pub pattern: Box<Span<Expression<'source>>>,

    /// Optionally, a set to denote the domain of the pattern.
    pub domain: Option<Box<Span<Expression<'source>>>>,

    /// `=>`
    pub maps_to: Box<Span<Expression<'source>>>,
}
