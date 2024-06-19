//! This module contains the AST for name resolution.

pub mod ast;

use std::collections::HashMap;
use std::{io, slice};
use parse_tools::bytes::Cursor;
use crate::lex::Lexer;
use crate::parse::{ast as parse_ast, ParseContext};
use crate::parse::ast::{ModuleContent, StatementKind, TopLevelItem};
use crate::resolve::ast::Module;

pub fn process<'t>(root_path: &str) -> Result<Module, Vec<crate::Error>> {
    let mut modules = HashMap::<String, (Box<[u8]>, ModuleContent)>::new();

    let data = Box::<[u8]>::from(b"let x = 20;".as_slice());
    let mut parser = ParseContext::new(Lexer::new(Cursor::new(&*data))).unwrap();
    let module_content = parser.parse_module().unwrap();

    let threads = Vec::new();
    
    for x in module_content.0.iter() {
        
    }
    
    modules.insert("".to_string(), (data, module_content));
    
    
    todo!()
}