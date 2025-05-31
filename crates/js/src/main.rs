use oxc_allocator::{Allocator, Box, Vec};
use oxc_ast::ast::*;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use std::cell::Cell;

fn main() {
    let arena = Allocator::default();

    let program = Program {
        span: Default::default(),
        source_type: Default::default(),
        hashbang: None,
        directives: Vec::new_in(&arena),
        body: {
            let mut vec = Vec::new_in(&arena);

            vec.push(Statement::VariableDeclaration(Box::new_in(VariableDeclaration {
                span: Default::default(),
                kind: VariableDeclarationKind::Let,
                declarations: {
                    let mut vec = Vec::new_in(&arena);

                    vec.push(VariableDeclarator {
                        span: Default::default(),
                        kind: VariableDeclarationKind::Let,
                        id: BindingPattern {
                            kind: BindingPatternKind::BindingIdentifier(Box::new_in(BindingIdentifier {
                                span: Default::default(),
                                name: Atom::from("test_yo"),
                                symbol_id: Cell::new(None),
                            }, &arena)),
                            type_annotation: None,
                            optional: false,
                        },
                        init: Some(Expression::NumericLiteral(Box::new_in(NumericLiteral {
                            span: Default::default(),
                            value: 42.0,
                            raw: "42",
                            base: NumberBase::Decimal,
                        }, &arena))),
                        definite: false,
                    });

                    vec
                },
                declare: false,
            }, &arena)));

            vec
        },
        scope_id: Cell::new(None),
    };

    let text = CodeGenerator::new()
        .with_options(CodegenOptions {
            single_quote: false,
            minify: true,
        })
        .build(&program)
        .source_text;

    println!("{text}");
}