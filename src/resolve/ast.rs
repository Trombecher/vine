use std::collections::HashMap;
use crate::parse::ast::Operation;
use crate::Span;

#[derive(Debug)]
pub enum Type {
    Never,
    Nil,
    Number,
    Boolean,
    String,
    // Function,
    // Item
}

#[derive(Debug)]
pub enum Expression<'t> {
    Operation {
        left: Box<Span<Expression<'t>>>,
        operation: Operation,
        right: Box<Span<Expression<'t>>>,
    },
    // Assignment {}
    Not(Box<Span<Expression<'t>>>),
    Return(Box<Span<Expression<'t>>>),
    Continue,
    Break,
    // Function {}
    Number(f64),
    String(String),
    // Scope(Vec<Span<StatementOrExpression>>),
    // Markup(MarkupElement<'s>)
    Identifier(&'t Item<'t>),
    False,
    True,
    This,
    Nil,
    // Access,
    // OptionalAccess,
    Call {
        target: Span<&'t Item<'t>>,
        arguments: Vec<Span<Expression<'t>>>
    },
    If {
    }
}

#[derive(Debug)]
pub enum StatementOrExpression {
    
}

#[derive(Debug)]
pub struct If<'t> {
    pub condition: Box<Span<Expression<'t>>>,
    pub body: Span<Vec<Span<Expression<'t>>>>,
}

#[derive(Debug)]
pub struct Module<'t>(pub HashMap<&'t str, Item<'t>>);

#[derive(Debug)]
pub enum Item<'t> {
    Module(Module<'t>),
    Declaration {
        is_mutable: bool,
        value: Expression<'t>
    }
}