use bytes::Span;
use errors::Error;
use crate::lex::{Token, TokenIterator};

/// Wraps a [TokenIterator] and buffers tokens.
///
/// It allows to peek into the next token (via [Buffered::peek])
/// or even into the token after that (via [Buffered::peek_after])
/// **without advancing**.
pub struct Buffered<'a, T: TokenIterator<'a>> {
    iter: T,
    next_token: Span<Token<'a>>,
    next_next_token: Option<Span<Token<'a>>>,
}

impl<'a, T: TokenIterator<'a>> Buffered<'a, T> {
    #[inline]
    pub fn new(mut iter: T) -> Result<Buffered<'a, T>, Error> {
        Ok(Self {
            next_token: iter.next_token()?,
            iter,
            next_next_token: None,
        })
    }

    /// Creates a new [Buffered] with a specified first token.
    #[inline]
    pub fn new_init(init: Span<Token<'a>>, iter: T) -> Buffered<'a, T> {
        Self {
            iter,
            next_token: init,
            next_next_token: None,
        }
    }

    #[inline]
    pub fn peek<'b>(&'b self) -> &'b Span<Token<'a>> {
        &self.next_token
    }

    /// Returns a shared reference to the token after the token, [Self::peek] would return.
    /// In the process of generating a new token, a line break is skipped.
    #[inline]
    pub fn peek_after<'b>(&'b mut self) -> Result<&'b Span<Token<'a>>, Error> {
        if self.next_next_token.is_none() {
            self.next_next_token = Some(self.iter.next_token()?);

            // Skip a line break
            if let Token::LineBreak = unsafe { self.next_next_token.as_ref().unwrap_unchecked() }.value {
                self.next_next_token = Some(self.iter.next_token()?);
            }
        }

        Ok(unsafe {
            self.next_next_token.as_ref().unwrap_unchecked()
        })
    }


    #[inline]
    pub fn peek_non_lb<'b>(&'b mut self) -> Result<(&'b Span<Token<'a>>, bool), Error> {
        Ok(match self.peek().value {
            Token::LineBreak => (self.peek_after()?, true),
            _ => (self.peek(), false) // TODO: the borrow checker is wrong on this one. The line below should be accepted!
            // token => (token, false)
        })
    }


    #[inline]
    pub fn skip_lb(&mut self) -> Result<bool, Error> {
        Ok(match self.peek().value {
            Token::LineBreak => {
                self.advance()?;
                true
            },
            Token::EndOfInput => true,
            _ => false,
        })
    }

    #[inline]
    pub fn advance(&mut self) -> Result<(), Error> {
        self.next_token = if let Some(next) = self.next_next_token.take() {
            next
        } else {
            self.iter.next_token()?
        };
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