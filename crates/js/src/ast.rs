pub type Reference = u32;

pub enum Expression {

}

/// All JS [Statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#statements_and_declarations_by_category)
pub enum Statement {
    Empty,
    /// The [`debugger`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/debugger) Statement
    Debugger,
    Return(Expression),
    Break,
    Continue,
    Throw(Expression),
    TryCatch {
        try_block: Vec<()>,
        catch: String,

    }
}

// A-Za-z_$        54
// A-Za-z_$0-9     64

pub enum Declaration {
    Let(Vec<(Reference, )>)
}