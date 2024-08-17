use std::str::from_raw_parts;
use bytes::{Cursor, Index, Span};
use crate::Error;
use crate::token::{Token, TokenIterator, INSTRUCTION_MAP, KEYWORD_MAP};

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    start: *const u8,
}

impl<'a> Lexer<'a> {
    pub const fn new(slice: &[u8]) -> Self {
        Self {
            cursor: Cursor::new(slice),
            start: slice.as_ptr(),
        }
    }
    
    /// Calculates the index of the next byte.
    pub fn index(&self) -> Index {
        unsafe { self.cursor.cursor().sub_ptr(self.start) as Index }
    }
}

impl<'a> TokenIterator<'a> for Lexer<'a> {
    fn next_token(&mut self) -> Result<Span<Token<'a>>, Error> {
        self.cursor.skip_ascii_whitespace();
        
        let start = self.index();
        
        let token = match self.cursor.peek() {
            Some(b'$') => todo!("id not impl"),
            Some(_) => {
                let start = self.cursor.cursor();
                
                loop {
                    match self.cursor.peek() {
                        None | Some(b'{') => break,
                        Some(x) if x.is_ascii_whitespace() => break,
                        _ => unsafe { self.cursor.advance_unchecked() }
                    }
                }
                
                let capture = unsafe {
                    from_raw_parts(start, self.cursor.cursor().sub_ptr(start))
                };
                
                if let Some(instr) = INSTRUCTION_MAP.get(capture) {
                    Token::Instruction(*instr)
                } else if let Some(kw) = KEYWORD_MAP.get(capture) {
                    Token::Keyword(*kw)
                } else {
                    todo!("some error here")
                }
            }
            None => Token::EndOfFile,
        };
        
        Ok(Span {
            value: token,
            source: start..self.index(),
        })
    }
}