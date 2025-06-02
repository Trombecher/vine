use byte_reader::Cursor;

pub trait CursorExt {
    fn peek_non_whitespace(&self, n: usize) -> Option<u8>;
}

impl<'source> CursorExt for Cursor<'source> {
    fn peek_non_whitespace(&self, mut n: usize) -> Option<u8> {
        let mut index = 0;
        
        loop {
            match self.peek_n(index) {
                None => break None,
                Some(x) if x.is_ascii_whitespace() => index += 1,
                Some(x) if n == 0 => break Some(x),
                Some(_) => {
                    n -= 1;
                    index += 1;
                }
            }
        }
    }
}