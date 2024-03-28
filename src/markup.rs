use crate::token::{Token, WithSpan};

#[derive(Debug)]
pub enum Child {
    Element(Element),
    Text(String),
    Insert(Vec<Token>)
}

#[derive(Debug)]
pub struct Element {
    pub children: Vec<WithSpan<Child>>,
    pub identifier: WithSpan<String>,
    pub attributes: Vec<(WithSpan<String>, WithSpan<Vec<Token>>)>,
}