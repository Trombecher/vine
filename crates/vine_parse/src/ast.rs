use parser_tools::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'source> {
    Number(u64),

    /// An identifier.
    Identifier(&'source str),

    /// A unary Expression<'source>:
    ///
    /// ```plain
    /// <UOP> <EXPR>
    /// ```
    Unary {
        operation: UnaryOperation,
        inner: Box<Expression<'source>>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation {
    /// `-`
    Negate,

    /// `!`
    Not,
}

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
}

/// A pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<'source> {
    /// An identifier.
    Identifier(&'source str),

    /// An application:
    ///
    /// ```plain
    /// <EXPRESSION> <PATTERN>
    /// ```
    Application {
        function: Box<Span<Expression<'source>>>,
        argument: Box<Span<Pattern<'source>>>,
    },
}

/// A match case:
///
/// ```plain
/// case <PATTERN> [is|in <EXPRESSION>] => <EXPRESSION>
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase<'source> {
    /// The pattern to match against.
    pub pattern: Span<Pattern<'source>>,

    /// Optionally, a set to denote the domain of the pattern.
    pub in_set: Option<Box<Span<Expression<'source>>>>,

    /// `=>`
    pub maps_to: Box<Span<Expression<'source>>>,
}
