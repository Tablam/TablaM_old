extern crate logos;

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
enum Token {
    // Logos requires that we define two default variants,
    // one for end of input source,
    #[end]
    End,

    // ...and one for errors. Those can be named anything
    // you wish as long as the attributes are there.
    #[error]
    Error,

    #[token = "("]
    LeftParen,

    #[token = ")"]
    RightParen,

    #[token = "{"]
    LeftBrace,

    #[token = "}"]
    RightBrace,

    #[token = ","]
    Comma,

    #[token = "."]
    Dot,

    #[token = "-"]
    Minus,

    #[token = "+"]
    Plus,

    #[token = ";"]
    Semicolon,

    #[token = "/"]
    Slash,

    #[token = "*"]
    Star,

    // Or regular expressions.
    #[regex = "[a-zA-Z]+"]
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_characters() {
        let mut lexer = Token::lexer("(0) {1,2} 4+5=9-2=7/7=1 and semicolon ; isn't necessary.");

        assert_eq!(lexer.token, Token::LeftParen);
        assert_eq!(lexer.slice(), "(");
        assert_eq!(lexer.range(), 0..1);

        lexer.advance();

        assert_eq!(lexer.token, Token::RightParen);
        assert_eq!(lexer.slice(), ")");
        assert_eq!(lexer.range(), 2..3);

        lexer.advance();

        assert_eq!(lexer.token, Token::LeftBrace);
        assert_eq!(lexer.slice(), "{");
        assert_eq!(lexer.range(), 4..5);

        lexer.advance();

        assert_eq!(lexer.token, Token::Comma);
        assert_eq!(lexer.slice(), ",");
        assert_eq!(lexer.range(), 7..8);

        lexer.advance();

        assert_eq!(lexer.token, Token::RightBrace);
        assert_eq!(lexer.slice(), "}");
        assert_eq!(lexer.range(), 9..10);

        lexer.advance();
    }

    #[test]
    fn it_works() {
        let mut lexer = Token::lexer("Create 0 ridiculously fast Lexers.");

        assert_eq!(lexer.token, Token::Text);
        assert_eq!(lexer.slice(), "Create");
        assert_eq!(lexer.range(), 0..6);

        lexer.advance();

        assert_eq!(lexer.token, Token::Text);
        assert_eq!(lexer.slice(), "ridiculously");
        assert_eq!(lexer.range(), 7..19);

        lexer.advance();

        assert_eq!(lexer.token, Token::Text);
        assert_eq!(lexer.slice(), "fast");
        assert_eq!(lexer.range(), 20..24);

        lexer.advance();

        assert_eq!(lexer.token, Token::Text);
        assert_eq!(lexer.slice(), "Lexers");
        assert_eq!(lexer.range(), 25..31);

        lexer.advance();

        assert_eq!(lexer.token, Token::Dot);
        assert_eq!(lexer.slice(), ".");
        assert_eq!(lexer.range(), 31..32);

        lexer.advance();

        assert_eq!(lexer.token, Token::End);
    }
}
