//! This module contains the AST for name resolution.

pub mod ast;

use std::collections::HashMap;
use std::io;
use parse_tools::bytes::Cursor;
use crate::lex::Lexer;
use crate::parse::{ast as parse_ast, ParseContext};
use crate::parse::ast::ModuleContent;
use crate::resolve::ast::Module;

pub fn process<'t>(root_path: &str) -> Result<Module, Vec<crate::Error>> {
    let mut modules = HashMap::<String, (Box<[u8]>, ModuleContent)>::new();
    
    let data = Box::new(*b"let x = 20;");
    let mut parser = ParseContext::new(Lexer::new(Cursor::new(data.as_slice()))).unwrap();
    let module_content = parser.parse_module().unwrap();
    
    modules.insert("".to_string(), (data, module_content));
    
    todo!()
}