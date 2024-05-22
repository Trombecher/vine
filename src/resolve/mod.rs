//! This module contains the AST for name resolution.

pub mod ast;
pub mod error;
mod deps;

use std::collections::HashMap;
use crate::parse::ast as parse_ast;

pub fn get_dependencies(module: parse_ast::Module) {
    
}

pub fn resolve(module: parse_ast::Module) -> HashMap<&str, ()> {
    let item_map = HashMap::new();
    
    item_map
}