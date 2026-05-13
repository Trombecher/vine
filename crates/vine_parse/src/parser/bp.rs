#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BindingPrecedence {
    #[default]
    Lowest,
    AssignmentRight,
    AssignmentLeft,
    OrLeft,
    OrRight,
    AndLeft,
    AndRight,
    EqualityLeft,
    EqualityRight,
    ComparisonLeft,
    ComparisonRight,
    AdditiveLeft,
    AdditiveRight,
    MultiplicativeLeft,
    MultiplicativeRight,
    CallLeft,
    CallRight,
    Unary,
    AccessLeft,
    AccessRight,
}
