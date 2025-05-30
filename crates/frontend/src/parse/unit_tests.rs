#![cfg(test)]

use crate::lex::{Symbol, Token};
use crate::parse::{ast, ParseContext};
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec;
use errors::Error;
use fallible_iterator::{FallibleIterator, IteratorExt};
use labuf::LookaheadBuffer;
use span::Span;

fn context_from_tokens<const N: usize>(
    tokens: [Span<Token>; N],
) -> ParseContext<impl FallibleIterator<Item = Span<Token>, Error = Error>, Global> {
    ParseContext::new(
        LookaheadBuffer::new_in(tokens.into_iter().map(Ok).transpose_into_fallible(), Global),
        Global,
    )
}

#[test]
fn parse_use_no_child() {
    let mut parser = context_from_tokens([Span {
        value: Token::LineBreak,
        source: 8..9,
    }]);

    assert_eq!(
        parser.parse_use(Span {
            value: "import",
            source: 2..8,
        }),
        Ok(ast::Use {
            id: Span {
                value: "import",
                source: 2..8,
            },
            child: None,
        })
    )
}

#[test]
fn parse_use_with_child() {
    let mut parser = context_from_tokens([
        Span {
            value: Token::LineBreak,
            source: 6..7,
        },
        Span {
            value: Token::Symbol(Symbol::Dot),
            source: 7..8,
        },
        Span {
            value: Token::LineBreak,
            source: 8..9,
        },
        Span {
            value: Token::Identifier("yo"),
            source: 9..11,
        },
    ]);

    assert_eq!(
        parser.parse_use(Span {
            value: "import",
            source: 0..6,
        }),
        Ok(ast::Use {
            id: Span {
                value: "import",
                source: 0..6,
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
        Span {
            value: Token::Symbol(Symbol::Dot),
            source: 0..1,
        },
        Span {
            value: Token::Symbol(Symbol::LeftParenthesis),
            source: 1..2,
        },
        Span {
            value: Token::Identifier("a"),
            source: 2..3,
        },
        Span {
            value: Token::LineBreak,
            source: 3..4,
        },
        Span {
            value: Token::Identifier("b"),
            source: 4..5,
        },
        Span {
            value: Token::Symbol(Symbol::RightParenthesis),
            source: 5..6,
        },
        Span {
            value: Token::LineBreak,
            source: 6..7,
        },
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
