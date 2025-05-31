/// Extension trait for [LookaheadBuffer].
mod units_tests;

use crate::lex::Token;
use core::alloc::Allocator;
use errors::Error;
use fallible_iterator::FallibleIterator;
use labuf::LookaheadBuffer;
use span::Span;

pub trait LabExt<'source> {
    fn skip_lb(&mut self) -> Result<bool, Error>;
    fn peek_n_non_lb(&mut self, n: usize) -> Result<(Option<&Span<Token<'source>>>, bool), Error>;
}

impl<'source, I: FallibleIterator<Item = Span<Token<'source>>, Error = Error>, A: Allocator>
    LabExt<'source> for LookaheadBuffer<I, A>
{
    /// Advances the iterator if the next token is a line break.
    /// Returns `true` if a line break was skipped.
    fn skip_lb(&mut self) -> Result<bool, Error> {
        Ok(match self.peek()?.map(|x| &x.value) {
            Some(Token::LineBreak) => {
                self.advance()?;
                true
            }
            None => true,
            _ => false,
        })
    }

    /// Allows peeking into the nth non-line break token.
    /// The second field is `true` if a line break was skipped.
    fn peek_n_non_lb(
        &mut self,
        mut n: usize,
    ) -> Result<(Option<&Span<Token<'source>>>, bool), Error> {
        let mut real_index = 0;

        loop {
            if real_index >= self.queue().len()
                && let Some(x) = self.iter_mut().next()?
            {
                self.queue_mut().push_back(x);
            }

            match self.queue().get(real_index) {
                Some(Span {
                    value: Token::LineBreak,
                    ..
                }) => {}
                _ if n > 0 => n -= 1,
                // Why do I have to borrow again???
                _ => return Ok((self.queue().get(real_index), real_index != n)),
            }

            real_index += 1;
        }
    }
}
