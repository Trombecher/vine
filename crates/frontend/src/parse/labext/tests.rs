#![cfg(test)]

use crate::lex::Token;
use crate::parse::labext::LabExt;
use alloc::alloc::Global;
use errors::Error;
use fallible_iterator::IteratorExt;
use labuf::LookaheadBuffer;
use span::Span;

#[test]
fn skip_lb() {
    let mut buf = LookaheadBuffer::new_in(
        [
            Token::String("Hi!".into()),
            Token::LineBreak,
            Token::Number(0.),
        ]
        .into_iter()
        .map::<Result<Span<Token<'static>>, Error>, _>(|x| {
            Ok(Span {
                value: x,
                source: 0..0,
            })
        })
        .transpose_into_fallible(),
        Global,
    );

    assert_eq!(
        buf.peek(),
        Ok(Some(&Span {
            value: Token::String("Hi!".into()),
            source: 0..0
        }))
    );

    assert_eq!(buf.advance(), Ok(()));

    assert_eq!(buf.skip_lb(), Ok(true),);

    assert_eq!(
        buf.peek(),
        Ok(Some(&Span {
            value: Token::Number(0.),
            source: 0..0
        }))
    );

    assert_eq!(buf.skip_lb(), Ok(false),);

    assert_eq!(buf.advance(), Ok(()));

    assert_eq!(buf.peek(), Ok(None));
}
