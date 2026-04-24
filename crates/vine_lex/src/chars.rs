use core::str;

pub struct Chars<'input> {
    chars: str::Chars<'input>,
}

impl<'input> Chars<'input> {
    #[inline]
    #[must_use]
    pub fn new(s: &'input str) -> Self {
        Self { chars: s.chars() }
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    #[inline]
    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    #[inline]
    pub fn advance(&mut self) {
        self.next();
    }
}
