#![cfg(test)]

use crate::lex::{Symbol, Token};
use crate::parse::ast::Expression;
use crate::parse::{ast, ParseContext};
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec;
use errors::Error;
use fallible_iterator::{FallibleIterator, IteratorExt};
use labuf::LookaheadBuffer;
use span::{Index, Span};

fn context_from_tokens<const N: usize>(
    tokens: [Token; N],
) -> ParseContext<impl FallibleIterator<Item = Span<Token>, Error = Error>, Global> {
    let mut index: Index = 0;

    ParseContext::new(
        LookaheadBuffer::new_in(
            tokens
                .into_iter()
                .map(move |token| {
                    let len = token.estimated_length();
                    let source = index..index + len;
                    index += len;

                    Ok(Span {
                        value: token,
                        source,
                    })
                })
                .transpose_into_fallible(),
            Global,
        ),
        Global,
    )
}

#[test]
fn parse_use_no_child() {
    let mut parser = context_from_tokens([Token::Identifier("import"), Token::LineBreak]);

    let import_range = parser.iter.next().unwrap().unwrap().source;

    assert_eq!(
        parser.parse_use(Span {
            value: "import",
            source: import_range.clone(),
        }),
        Ok(ast::Use {
            id: Span {
                value: "import",
                source: import_range,
            },
            child: None,
        })
    )
}

#[test]
fn parse_use_with_child() {
    let mut parser = context_from_tokens([
        Token::Identifier("import"),
        Token::LineBreak,
        Token::Symbol(Symbol::Dot),
        Token::LineBreak,
        Token::Identifier("yo"),
    ]);

    let import_range = parser.iter.next().unwrap().unwrap().source;

    assert_eq!(
        parser.parse_use(Span {
            value: "import",
            source: import_range.clone(),
        }),
        Ok(ast::Use {
            id: Span {
                value: "import",
                source: import_range,
            },
            child: Some(Span {
                value: ast::UseChild::Single(Box::new(ast::Use {
                    id: Span {
                        value: "yo",
                        source: 9..11,
                    },
                    child: None,
                })),
                source: 9..11
            }),
        })
    )
}

#[test]
fn parse_use_child_multiple() {
    let mut parser = context_from_tokens([
        Token::Symbol(Symbol::Dot),
        Token::Symbol(Symbol::LeftParenthesis),
        Token::Identifier("a"),
        Token::LineBreak,
        Token::Identifier("b"),
        Token::Symbol(Symbol::RightParenthesis),
        Token::LineBreak,
    ]);

    assert_eq!(
        parser.parse_use_child(),
        Ok(Span {
            value: ast::UseChild::Multiple(vec![
                ast::Use {
                    id: Span {
                        value: "a",
                        source: 2..3,
                    },
                    child: None,
                },
                ast::Use {
                    id: Span {
                        value: "b",
                        source: 4..5,
                    },
                    child: None,
                }
            ]),
            source: 1..6,
        })
    );
}

#[test]
fn parse_object_literal() {
    let mut parser = context_from_tokens([
        Token::Symbol(Symbol::LeftParenthesis),
        Token::Identifier("key"),
        Token::Symbol(Symbol::Equals),
        Token::Identifier("value"),
        Token::Symbol(Symbol::RightParenthesis),
    ]);

    parser.iter.advance().unwrap();

    assert_eq!(
        parser.parse_object_literal(),
        Ok((
            vec![ast::ObjectField {
                id: Span {
                    value: "key",
                    source: 1..4,
                },
                value: Span {
                    value: Expression::Identifier("value"),
                    source: 5..10,
                },
            }],
            11
        ))
    );
}
