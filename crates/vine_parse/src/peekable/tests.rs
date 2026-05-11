#[allow(non_snake_case)]
mod ArbitrarilyPeekable {
    use super::super::*;

    #[test]
    fn peek_n() {
        let mut peekable = ArbitrarilyPeekable::new([0_u8, 1, 2].into_iter());

        assert_eq!(peekable.peek_n(0).copied(), Some(0));
        assert_eq!(peekable.peek_n(0).copied(), Some(0));
        assert_eq!(peekable.peek_n(1).copied(), Some(1));
        assert_eq!(peekable.peek_n(2).copied(), Some(2));
        assert_eq!(peekable.peek_n(3).copied(), None);
        assert_eq!(peekable.peek_n(3).copied(), None);
        assert_eq!(peekable.peek_n(1000).copied(), None);

        peekable.next();

        assert_eq!(peekable.peek_n(0).copied(), Some(1));
        assert_eq!(peekable.peek_n(1).copied(), Some(2));
        assert_eq!(peekable.peek_n(2).copied(), None);

        peekable.next();
        peekable.next();
        peekable.next();

        assert_eq!(peekable.peek(), None);
        assert_eq!(peekable.peek(), None);
        assert_eq!(peekable.peek(), None);
    }

    #[test]
    fn next() {
        let mut peekable = ArbitrarilyPeekable::new([0_u8, 1, 2].into_iter());

        assert_eq!(peekable.peek_n(1).copied(), Some(1));
        assert_eq!(peekable.next(), Some(0));
        assert_eq!(peekable.next(), Some(1));

        assert_eq!(peekable.peek_n(3).copied(), None);
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), None);
        assert_eq!(peekable.next(), None);
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn next_no_linebreak() {
        let mut peekable = ArbitrarilyPeekable::new(
            [
                FilteredToken::Comma,
                FilteredToken::LineBreak,
                FilteredToken::At,
                FilteredToken::LineBreak,
            ]
            .into_iter()
            .map(|token| Span {
                value: token,
                range: 0..0,
            }),
        );

        assert_eq!(
            peekable.next_no_linebreak(),
            Some(Span {
                value: FilteredToken::Comma,
                range: 0..0
            })
        );

        assert_eq!(
            peekable.next_no_linebreak(),
            Some(Span {
                value: FilteredToken::At,
                range: 0..0
            })
        );

        assert_eq!(peekable.next_no_linebreak(), None);
        assert_eq!(peekable.next_no_linebreak(), None);
        assert_eq!(peekable.next_no_linebreak(), None);
    }

    #[test]
    fn peek_no_linebreak() {
        let mut peekable = ArbitrarilyPeekable::new(
            [
                FilteredToken::Bar,
                FilteredToken::LineBreak,
                FilteredToken::Block,
                FilteredToken::LineBreak,
            ]
            .into_iter()
            .map(|token| Span {
                value: token,
                range: 0..0,
            }),
        );

        assert_eq!(
            peekable.peek_no_linebreak_n(0),
            Some(&Span {
                value: FilteredToken::Bar,
                range: 0..0
            })
        );

        assert_eq!(
            peekable.peek_no_linebreak_n(1),
            Some(&Span {
                value: FilteredToken::Block,
                range: 0..0
            })
        );

        assert_eq!(peekable.peek_no_linebreak_n(2), None);
        assert_eq!(peekable.peek_no_linebreak_n(3), None);

        peekable.next_no_linebreak();

        assert_eq!(
            peekable.peek_no_linebreak_n(0),
            Some(&Span {
                value: FilteredToken::Block,
                range: 0..0
            })
        );

        assert_eq!(peekable.peek_no_linebreak_n(1), None);
        assert_eq!(peekable.peek_no_linebreak_n(2), None);
    }
}
