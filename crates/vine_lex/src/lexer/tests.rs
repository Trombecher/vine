#![allow(non_snake_case)]

mod Lexer {
    use super::super::*;

    #[test]
    fn next() {
        // TODO: add more

        let mut lexer = Lexer::new("+-*/^@&");

        assert_eq!(lexer.next(), Some(Token::Plus));
        assert_eq!(lexer.next(), Some(Token::Hypen));
        assert_eq!(lexer.next(), Some(Token::Star));
        assert_eq!(lexer.next(), Some(Token::Slash));
        assert_eq!(lexer.next(), Some(Token::Caret));
        assert_eq!(lexer.next(), Some(Token::At));
        assert_eq!(lexer.next(), Some(Token::Ampersand));
        assert_eq!(lexer.next(), None);
        assert_eq!(lexer.next(), None);
    }
}
