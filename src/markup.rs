use std::collections::HashMap;
use crate::token::Token;

#[derive(Debug)]
pub enum Child {
    Element(Element),
    Text(String),
    Tokens(Vec<Token>)
}

#[derive(Debug)]
pub struct Element {
    children: Vec<Child>,
    identifier: String,
    attributes: HashMap<String, Vec<Token>>,
}