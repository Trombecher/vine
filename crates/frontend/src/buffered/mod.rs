//! This module contains the wrapper [Buffered] to allow peeking into a token iterator.

// mod tests;
// mod old;

use alloc::alloc::Global;
use crate::lex::{Token, TokenIterator};
use alloc::collections::VecDeque;
use core::alloc::Allocator;
use core::mem::transmute;
use bytes::Span;
use errors::Error;

/// A variable size lookahead-buffer implementation for the parser.
pub struct LookaheadBuffer<'source_text, T: TokenIterator<'source_text>, LABAlloc: Allocator = Global> {
    iter: T,
    queue: VecDeque<Span<Token<'source_text>>, LABAlloc>,
}

impl<'source_text, T: TokenIterator<'source_text>> LookaheadBuffer<'source_text, T> {
    #[inline]
    pub const fn new(iter: T) -> Self {
        Self {
            iter,
            queue: VecDeque::new(),
        }
    }
}

impl<'source_text, T: TokenIterator<'source_text>, LABAlloc: Allocator>
LookaheadBuffer<'source_text, T, LABAlloc>
{
    #[inline]
    pub fn new_in(iter: T, allocator: LABAlloc) -> Result<Self, Error> {
        Ok(Self {
            iter,
            queue: VecDeque::new_in(allocator),
        })
    }

    /// Returns a reference to the underlying iterator.
    #[inline]
    pub const fn iter(&self) -> &T {
        &self.iter
    }

    #[inline]
    pub fn peek(&mut self) -> Result<&Span<Token<'source_text>>, Error> {
        self.peek_n(0)
    }

    #[inline]
    pub fn peek_n(&mut self, n: usize) -> Result<&Span<Token<'source_text>>, Error> {
        for _ in 0..(n as isize - self.queue.len() as isize + 1) {
            let new_token = self.iter.next_token()?;
            // println!("Lexer generated: {new_token:?}");
            self.queue.push_back(new_token);
        }

        // `unwrap()` is safe because we ensured that there is at least one element present.
        Ok(self.queue.get(n).unwrap())
    }

    /// Returns a shared reference to the nth token in the future, while skipping line breaks.
    #[inline]
    pub fn peek_n_non_lb(&mut self, mut n: usize) -> Result<(&Span<Token<'source_text>>, bool), Error> {
        let mut real_index = 0;

        loop {
            if real_index >= self.queue.len() {
                self.queue.push_back(self.iter.next_token()?);
            }
            
            match self.queue.get(real_index).unwrap() {
                Span { value: Token::LineBreak, .. } => {}
                _ if n > 0 => n -= 1,
                // Why do I have to borrow again???
                _ => return Ok((self.queue.get(real_index).unwrap(), real_index == n)),
            }

            real_index += 1;
        }
    }

    /// Returns a shared reference to the next token (via [Self::peek]) or,
    /// if that token is a line break, to the token after that (via [Self::peek_after]).
    ///
    /// If a line break was skipped, the second member of the returned tuple is `true`;
    /// otherwise `false`.
    #[inline]
    pub fn peek_non_lb(&mut self) -> Result<(&Span<Token<'source_text>>, bool), Error> {
        self.peek_n_non_lb(0)
    }

    #[inline]
    pub fn advance(&mut self) -> Result<(), Error> {
        self.next().map(|_| ())
    }

    #[inline]
    pub fn next(&mut self) -> Result<Span<Token<'source_text>>, Error> {
        match self.queue.pop_front() {
            Some(token) => Ok(token),
            None => self.iter.next_token()
        }
    }

    /// Skips a potential line break.
    /// Returns `Ok(true)` if a line break was skipped or a [Token::EndOfInput] was encountered;
    /// `Ok(false)` otherwise.
    #[inline]
    pub fn skip_lb(&mut self) -> Result<bool, Error> {
        Ok(match self.peek()?.value {
            Token::LineBreak => {
                self.advance()?;
                true
            },
            Token::EndOfInput => true,
            _ => false,
        })
    }
}
