use parse_tools::bytes::Cursor;

pub struct Lexer<'a>{
    cursor: Cursor<'a>
}

impl<'a> Lexer<'a> {
    pub fn next(&mut self) -> Result<Token<'a>, Error> {

    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.peek() {
            None => None,

        }
    }
}