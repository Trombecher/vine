#![cfg(test)]

use bytes::Span;
use errors::Error;
use crate::buffered::Buffered;
use crate::lex::{Symbol, Token, TokenIterator};

struct TestIter<I: Iterator<Item = Span<Token<'static>>>> {
    inner: I,
}

impl<I: Iterator<Item = Span<Token<'static>>>> TokenIterator<'static> for TestIter<I>  {
    fn next_token(&mut self) -> Result<Span<Token<'static>>, Error> {
        Ok(self.inner.next().unwrap_or_else(|| Span {
            value: Token::EndOfInput,
            source: Default::default(),
        }))
    }
}

static DATA: [Span<Token<'static>>; 5] = [
    Span {
        value: Token::Identifier("a"),
        source: 0..1,
    },
    Span {
        value: Token::Symbol(Symbol::PlusEquals),
        source: 2..4,
    },
    Span {
        value: Token::Number(20.0),
        source: 5..7,
    },
    Span {
        value: Token::Symbol(Symbol::Minus),
        source: 9..10,
    },
    Span {
        value: Token::Number(2.0),
        source: 11..12,
    }
];

/// Returns a buffered iterator over the tokens of `a += 20 - 2`.
fn test_iter() -> Buffered<'static, impl TokenIterator<'static>> {
    Buffered::new(TestIter {
        inner: DATA.iter().cloned(),
    }).unwrap()
}

fn peek_and_advance() {
    let mut buffered = test_iter();
    
    assert_eq!(buffered.peek(), &Span {
        value: Token::Identifier("a"),
        source: 0..1,
    });
    assert_eq!(buffered.peek(), &Span {
        value: Token::Identifier("a"),
        source: 0..1,
    });

    assert_eq!(buffered.advance(), Ok(()));
    
    assert_eq!(buffered.peek(), &Span {
        value: Token::Symbol(Symbol::PlusEquals),
        source: 2..4,
    });
}