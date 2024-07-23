use std::collections::VecDeque;
use error::Error;
use lex::Span;
use lex::token::{Token, TokenIterator};

/// Wraps a [TokenIterator] and buffers tokens.
pub struct Buffered<'a, T: TokenIterator<'a>> {
    iter: T,
    queue: VecDeque<Span<'a, Token<'a>>>
}

impl<'a, T: TokenIterator<'a>> Buffered<'a, T> {
    #[inline]
    pub const fn new(iter: T) -> Buffered<'a, T> {
        Self {
            iter,
            queue: VecDeque::new(),
        }
    }

    /// Equivalent to `peek(0)` (see [Buffered::peek]).
    #[inline]
    pub fn peek<'b>(&'b mut self) -> Result<&'b Span<'a, Token<'a>>, Error> {
        self.peek_n::<0>()
    }

    /// Equivalent to `peek_mut(0)` (see [Buffered::peek_mut]).
    #[inline]
    pub fn peek_mut<'b>(&'b mut self) -> Result<&'b mut Span<'a, Token<'a>>, Error> {
        self.peek_n_mut::<0>()
    }

    /// Ensures that at least n + 1 tokens are in the queue.
    #[inline]
    fn ensure_n1<const N: u8>(&mut self) -> Result<(), Error> {
        for _ in 0..(N + 1).saturating_sub(self.queue.len() as u8) {
            self.queue.push_back(self.iter.next_token()?)
        }
        Ok(())
    }

    /// Peeks into the nth token. The first token is of index 0.
    ///
    /// All tokens up to the desired one will be pre-generated.
    pub fn peek_n<'b, const N: u8>(&'b mut self) -> Result<&'b Span<'a, Token<'a>>, Error> {
        self.ensure_n1::<N>()?;
        unsafe {
            Ok(self.queue.get(N as usize).unwrap_unchecked())
        }
    }

    /// The mutable version of [Buffered::peek].
    pub fn peek_n_mut<'b, const N: u8>(&'b mut self) -> Result<&'b mut Span<'a, Token<'a>>, Error> {
        self.ensure_n1::<N>()?;
        unsafe {
            Ok(self.queue.get_mut(N as usize).unwrap_unchecked())
        }
    }

    /// Returns the next token.
    pub fn next(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        match self.queue.pop_front() {
            None => self.iter.next_token(),
            Some(x) => Ok(x)
        }
    }

    /// Pops the front element from the queue if the queue is not empty; otherwise advance the iterator.
    #[inline]
    pub fn advance(&mut self) -> Result<(), Error> {
        if let None = self.queue.pop_front() {
            let _ = self.iter.next_token()?;
        }

        Ok(())
    }
}