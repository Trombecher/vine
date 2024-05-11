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
pub enum Item {
    Declaration {
        is_mutable: bool,
        ty: Option<Type>
    },
}

pub enum TypeDeclaration {
    Class {
        
    },
    Enum(Vec<f64>)
}

#[derive(Debug)]
pub enum Expression<'i> {
    Operation {
        left: Box<Span<Expression<'i>>>,
        operation: Operation,
        right: Box<Span<Expression<'i>>>,
    },
    // Assignment {}
    Not(Box<Span<Expression<'i>>>),
    Return(Box<Span<Expression<'i>>>),
    Continue,
    Break,
    // Function {}
    Number(f64),
    String(String),
    // Scope(Vec<Span<StatementOrExpression>>),
    // Markup(MarkupElement<'s>)
    Identifier(&'i Item),
    False,
    True,
    This,
    Nil,
    // Access,
    // OptionalAccess,
    Call {
        target: Span<&'i Item>,
        arguments: Vec<Span<Expression<'i>>>
    },
    If {
    }
}

#[derive(Debug)]
pub enum StatementOrExpression {
    
}

pub struct If<'i> {
    pub condition: Box<Span<Expression<'i>>>,
    pub body: Span<Vec<Span<Expression<'i>>>>,
}