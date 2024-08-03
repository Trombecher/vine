use std::mem::replace;
use error::Error;
use lex::Span;
use lex::token::{Token, TokenIterator};
use warning::Warning;

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
    pub fn warnings(&self) -> &[Span<'a, Warning>] {
        self.iter.warnings()
    }
    
    #[inline]
    pub fn warnings_mut(&mut self) -> &mut Vec<Span<'a, Warning>> {
        self.iter.warnings_mut()
    }
    
    #[inline]
    pub fn consume_warnings(self) -> Vec<Span<'a, Warning>> {
        self.iter.consume_warnings()
    }
    
    #[inline]
    pub fn peek<'b>(&'b self) -> &'b Span<'a, Token<'a>> {
        &self.next_token
    }
    
    // /// Returns the next token.
    // #[inline]
    // pub fn next(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
    //     Ok(replace(&mut self.next_token, self.iter.next_token()?))
    // }

    /// Skips a potential line break. Returns `Ok(true)` if a line break was skipped, `Ok(false)` otherwise.
    #[inline]
    pub fn skip_lb(&mut self) -> Result<bool, Error> {
        Ok(if let Token::LineBreak = self.peek().value {
            self.advance()?;
            true
        } else {
            false
        })
    }

    #[inline]
    pub fn advance(&mut self) -> Result<(), Error> {
        let _ = replace(&mut self.next_token, self.iter.next_token()?);
        Ok(())
    }
    
    /// Advances the iterator one token.
    /// If the token is a [Token::LineBreak], it advances another token.
    #[inline]
    pub fn advance_skip_lb(&mut self) -> Result<bool, Error> {
        self.advance()?;
        self.skip_lb()
    }
}