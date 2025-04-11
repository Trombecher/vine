#![cfg(test)]

use alloc::boxed::Box;
use alloc::vec;
use crate::parse::{parse_module, ast::*};

#[test]
fn t1() {
    assert_eq!(
        parse_module(b"fn x() { }"),
        Ok((
            ModuleContent(vec![
                TopLevelItem {
                    is_public: false,
                    statement: Span {
                        value: Statement {
                            annotations: vec![],
                            statement_kind: StatementKind::Function {
                                signature: FunctionSignature {
                                    const_parameters: vec![],
                                    parameters: vec![],
                                    return_type: None,
                                },
                                id: "x",
                                this_parameter: None,
                                body: Box::new(Span {
                                    value: Expression::Block(vec![]),
                                    source: 7..10,
                                }),
                            },
                        },
                        source: 0..10,
                    },
                }
            ]),
            vec![]
        ))
    );
}