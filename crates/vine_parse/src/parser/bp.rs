#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BindingPrecedence {
    #[default]
    Lowest,
    AssignmentRight,
    AssignmentLeft,
    EqualityLeft,
    EqualityRight,
    ComparisonLeft,
    ComparisonRight,
    AdditiveLeft,
    AdditiveRight,
    MultiplicativeLeft,
    MultiplicativeRight,
    AccessLeft,
    AccessRight,
    Call,
    Negate,
}
