#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BindingPrecedence {
    #[default]
    Lowest,
    LineBreakLeft,
    LineBreakRight,
    AssignmentRight,
    AssignmentLeft,
    AdditiveLeft,
    AdditiveRight,
    MultiplicativeLeft,
    MultiplicativeRight,
    AccessLeft,
    AccessRight,
    Call,
}
