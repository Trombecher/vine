#![feature(allocator_api)]

//! Experimental AST-level JS backend for Vine.

mod transform;

use std::alloc::Allocator;
use oxc_ast::ast::{Program, SourceType};
use frontend::parse::ast;

pub fn transform(module: &ast::ModuleContent<impl Allocator>) -> Program {
    
}