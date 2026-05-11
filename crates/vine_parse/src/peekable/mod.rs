#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use parser_tools::Span;
use vine_lex::FilteredToken;

pub struct ArbitrarilyPeekable<Iter: Iterator> {
    /// The queue of already peeked items. On
    /// peek, items are added to the end, and on
    /// consumption, items are popped from the front.
    queue: VecDeque<Iter::Item>,

    /// The underlying iterator.
    iter: Iter,
}

impl<Iter: Iterator> ArbitrarilyPeekable<Iter> {
    pub const fn new(iter: Iter) -> Self {
        Self {
            queue: VecDeque::new(),
            iter,
        }
    }

    pub fn peek(&mut self) -> Option<&Iter::Item> {
        self.peek_n(0)
    }

    pub fn peek_n(&mut self, n: usize) -> Option<&Iter::Item> {
        while self.queue.len() <= n {
            let item = self.next()?;
            self.queue.push_back(item);
        }

        self.queue.get(n)
    }
}

impl<Iter: Iterator> Iterator for ArbitrarilyPeekable<Iter> {
    type Item = Iter::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front().or_else(|| self.iter.next())
    }
}

impl<'source, Iter: Iterator<Item = Span<FilteredToken<'source>>>> ArbitrarilyPeekable<Iter> {
    fn next_no_linebreak(&mut self) -> Option<Iter::Item> {
        match self.next() {
            Some(Span {
                value: FilteredToken::LineBreak,
                ..
            }) => self.iter.next(),
            token => token,
        }
    }

    /// Returns the nth non-line break item in the queue, filling it if needed.
    fn peek_no_linebreak_n(&mut self, mut n: usize) -> Option<&Iter::Item> {
        let mut index = 0;

        loop {
            let item = self.peek_n(index);

            if let Some(Span {
                value: FilteredToken::LineBreak,
                ..
            }) = item
            {
            } else {
                if n == 0 {
                    break;
                }

                n -= 1;
            }

            index += 1;
        }

        self.queue.get(index)
    }
}
