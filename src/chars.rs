use std::str::Chars;
use crate::ion;
use crate::lexer::Error;

pub struct CharsIterator<'a> {
    chars: Chars<'a>,
    force_take: Option<char>,
    index: usize,
}

impl<'a> CharsIterator<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            force_take: None,
            index: 0,
        }
    }
    
    pub fn unescape_char(&mut self, char: char) -> Result<char, ion::Error> {
        // TODO: unicode escapes (use of self)
        match char {
            '0' => Ok('\0'), // Null character
            '\\' => Ok('\\'),
            'f' => Ok('\u{0c}'), // Form feed
            't' => Ok('\t'),     // Horizontal tab
            'r' => Ok('\r'),     // Carriage return
            'n' => Ok('\n'),     // Line feed / new line
            'b' => Ok('\u{07}'), // Bell
            'v' => Ok('\u{0b}'), // Vertical tab
            '"' => Ok('"'),
            '\'' => Ok('\''),
            '[' => Ok('\u{1B}'), // Escape
            _ => Err(ion::Error::Lexer(Error::UnknownEscapeCharacter)),
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self.index += 1;
        self.force_take.take().or_else(|| self.chars.next())
    }

    pub fn rollback(&mut self, option: Option<char>) {
        self.index -= 1;
        self.force_take = option;
    }

    pub fn skip_white_space(&mut self) {
        loop {
            match self.next() {
                Some(char) if char.is_whitespace() => {}
                option => {
                    self.rollback(option);
                    break;
                }
            }
        }
    }
    
    pub fn index(&self) -> usize {
        self.index
    }
}