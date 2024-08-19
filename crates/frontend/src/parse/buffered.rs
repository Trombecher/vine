use bytes::Span;
use crate::lex::{Error, Token, TokenIterator};

/// Wraps a [TokenIterator] and buffers tokens.
///
/// It allows to peek into the next token (via [Buffered::peek])
/// or even into the token after that (via [Buffered::peek_after])
/// **without advancing the iterator**.
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

    // #[inline]
    // pub fn warnings(&self) -> &[Span<Warning>] {
    //     self.iter.warnings()
    // }
    
    // #[inline]
    // pub fn warnings_mut(&mut self) -> &mut Vec<Span<Warning>> {
    //     self.iter.warnings_mut()
    // }
    
    // #[inline]
    // pub fn consume_warnings(self) -> Vec<Span<Warning>> {
    //     self.iter.consume_warnings()
    // }
    
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

    /// Returns a shared reference to the next token (via [Self::peek]) or,
    /// if that token is a line break, to the token after that (via [Self::peek_after]).
    ///
    /// If a line break was skipped, the second member of the returned tuple is `true`;
    /// otherwise `false`.
    #[inline]
    pub fn peek_non_lb<'b>(&'b mut self) -> Result<(&'b Span<Token<'a>>, bool), Error> {
        Ok(match self.peek().value {
            Token::LineBreak => (self.peek_after()?, true),
            _ => (self.peek(), false) // TODO: the borrow checker is wrong on this one. The line below should be accepted!
            // token => (token, false)
        })
    }
    
    // /// Returns the next token.
    // #[inline]
    // pub fn next(&mut self) -> Result<Span<'a, Token<'a>>, Error> {
    //     Ok(replace(&mut self.next_token, self.iter.next_token()?))
    // }

    /// Skips a potential line break.
    /// Returns `Ok(true)` if a line break was skipped or a [Token::EndOfInput] was encountered;
    /// `Ok(false)` otherwise.
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