#![allow(non_snake_case)]

mod Lexer {
    use crate::tokens::CharacterSource;

    use super::super::*;

    #[test]
    fn next() {
        // TODO: add more

        let mut lexer =
            Lexer::new(" \n\r\t iäß_a347 123_456_789__\0'ß';@^,=<>(){}[]+-*/|.&!$%§?#~`:");

        assert_eq!(
            lexer.next(),
            Some(Token::Whitespace(unsafe {
                WhitespaceSource::new_unchecked(" \n\r\t ")
            }))
        );
        assert_eq!(lexer.next(), Some(Token::IdentifierOrKeyword("iäß_a347")));
        assert_eq!(
            lexer.next(),
            Some(Token::Whitespace(unsafe {
                WhitespaceSource::new_unchecked(" ")
            }))
        );
        assert_eq!(
            lexer.next(),
            Some(Token::Number(unsafe {
                NumberSource::new_unchecked("123_456_789__")
            }))
        );
        assert_eq!(lexer.next(), Some(Token::Invalid("\0")));
        assert_eq!(
            lexer.next(),
            Some(Token::Character(unsafe {
                CharacterSource::new_unchecked("'ß'")
            }))
        );
        // TODO: test & implement comment & string
        assert_eq!(lexer.next(), Some(Token::Semicolon));
        assert_eq!(lexer.next(), Some(Token::At));
        assert_eq!(lexer.next(), Some(Token::Caret));
        assert_eq!(lexer.next(), Some(Token::Comma));
        assert_eq!(lexer.next(), Some(Token::Equals));
        assert_eq!(lexer.next(), Some(Token::LessThan));
        assert_eq!(lexer.next(), Some(Token::GreaterThan));
        assert_eq!(lexer.next(), Some(Token::OpeningParenthesis));
        assert_eq!(lexer.next(), Some(Token::ClosingParenthesis));
        assert_eq!(lexer.next(), Some(Token::OpeningBrace));
        assert_eq!(lexer.next(), Some(Token::ClosingBrace));
        assert_eq!(lexer.next(), Some(Token::OpeningBracket));
        assert_eq!(lexer.next(), Some(Token::ClosingBracket));
        assert_eq!(lexer.next(), Some(Token::Plus));
        assert_eq!(lexer.next(), Some(Token::Hypen));
        assert_eq!(lexer.next(), Some(Token::Star));
        assert_eq!(lexer.next(), Some(Token::Slash));
        assert_eq!(lexer.next(), Some(Token::Bar));
        assert_eq!(lexer.next(), Some(Token::Period));
        assert_eq!(lexer.next(), Some(Token::Ampersand));
        assert_eq!(lexer.next(), Some(Token::ExclamationMark));
        assert_eq!(lexer.next(), Some(Token::DollarSign));
        assert_eq!(lexer.next(), Some(Token::Percent));
        assert_eq!(lexer.next(), Some(Token::Paragraph));
        assert_eq!(lexer.next(), Some(Token::QuestionMark));
        assert_eq!(lexer.next(), Some(Token::Hashtag));
        assert_eq!(lexer.next(), Some(Token::Tilde));
        assert_eq!(lexer.next(), Some(Token::Backtick));
        assert_eq!(lexer.next(), Some(Token::Colon));
        assert_eq!(lexer.next(), None);
        assert_eq!(lexer.next(), None);
    }
}
