use std::mem::replace;
use error::Error;
use lex::Span;
use lex::token::{Token, TokenIterator};

/// Wraps a [TokenIterator] and buffers tokens.
pub struct Buffered<'a, T: TokenIterator<'a>> {
    iter: T,
    next_token: Span<'a, Token<'a>>
}

impl<'a, T: TokenIterator<'a>> Buffered<'a, T> {
    #[inline]
    pub fn new(mut iter: T) -> Result<Buffered<'a, T>, Error> {
        Ok(Self {
            next_token: iter.next_token()?,
            iter,
        })
    }

    #[inline]
    pub fn peek<'b>(&'b self) -> &'b Span<'a, Token<'a>> {
        &self.next_token
    }
    
    /// Returns the next token.
    #[inline]
    pub fn next(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
        Ok(replace(&mut self.next_token, self.iter.next_token()?))
    }

    /// Returns the next non-whitespace token.
    #[inline]
    pub fn skip_ws(&mut self) -> Result<(), Error> {
        loop {
            match self.peek() {
                Span { value: Token::LineBreak, .. } => self.advance()?,
                _ => break Ok(())
            }
        }
    }

    #[inline]
    pub fn advance(&mut self) -> Result<(), Error> {
        let _ = replace(&mut self.next_token, self.iter.next_token()?);
        Ok(())
    }
}