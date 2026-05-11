#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BindingPrecedence {
    #[default]
    Lowest,
    AdditiveLeft,
    AdditiveRight,
    MultiplicativeLeft,
    MultiplicativeRight,
    Call,
}
