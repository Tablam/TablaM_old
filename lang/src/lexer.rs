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

    #[token = "/"]
    Slash,

    #[token = "*"]
    Star,

    // Or regular expressions.
    #[regex = "[a-zA-Z]+"]
    Text,

    //For integer only
    #[regex = r"[0-9]+"]
    IntegerNumber,

    //For Float64
    #[regex = r"[0-9]+\.[0-9]+f"]
    FloatNumber,

    //For Dec64
    #[regex = r"[0-9]+\.[0-9]+"]
    DecimalNumber,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_characters() {
        let mut lexer = Token::lexer("(0) {1,2} 4+5 9-2 7/7 8*8 .");

        assert_eq!(lexer.token, Token::LeftParen);
        assert_eq!(lexer.slice(), "(");
        assert_eq!(lexer.range(), 0..1);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "0");
        assert_eq!(lexer.range(), 1..2);

        lexer.advance();

        assert_eq!(lexer.token, Token::RightParen);
        assert_eq!(lexer.slice(), ")");
        assert_eq!(lexer.range(), 2..3);

        lexer.advance();

        assert_eq!(lexer.token, Token::LeftBrace);
        assert_eq!(lexer.slice(), "{");
        assert_eq!(lexer.range(), 4..5);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "1");
        assert_eq!(lexer.range(), 5..6);

        lexer.advance();

        assert_eq!(lexer.token, Token::Comma);
        assert_eq!(lexer.slice(), ",");
        assert_eq!(lexer.range(), 6..7);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "2");
        assert_eq!(lexer.range(), 7..8);

        lexer.advance();

        assert_eq!(lexer.token, Token::RightBrace);
        assert_eq!(lexer.slice(), "}");
        assert_eq!(lexer.range(), 8..9);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "4");
        assert_eq!(lexer.range(), 10..11);

        lexer.advance();

        assert_eq!(lexer.token, Token::Plus);
        assert_eq!(lexer.slice(), "+");
        assert_eq!(lexer.range(), 11..12);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "5");
        assert_eq!(lexer.range(), 12..13);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "9");
        assert_eq!(lexer.range(), 14..15);

        lexer.advance();

        assert_eq!(lexer.token, Token::Minus);
        assert_eq!(lexer.slice(), "-");
        assert_eq!(lexer.range(), 15..16);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "2");
        assert_eq!(lexer.range(), 16..17);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "7");
        assert_eq!(lexer.range(), 18..19);

        lexer.advance();

        assert_eq!(lexer.token, Token::Slash);
        assert_eq!(lexer.slice(), "/");
        assert_eq!(lexer.range(), 19..20);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "7");
        assert_eq!(lexer.range(), 20..21);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "8");
        assert_eq!(lexer.range(), 22..23);

        lexer.advance();

        assert_eq!(lexer.token, Token::Star);
        assert_eq!(lexer.slice(), "*");
        assert_eq!(lexer.range(), 23..24);

        lexer.advance();

        assert_eq!(lexer.token, Token::IntegerNumber);
        assert_eq!(lexer.slice(), "8");
        assert_eq!(lexer.range(), 24..25);

        lexer.advance();

        assert_eq!(lexer.token, Token::Dot);
        assert_eq!(lexer.slice(), ".");
        assert_eq!(lexer.range(), 26..27);

        lexer.advance();

        assert_eq!(lexer.token, Token::End);
    }

    //(0) {1,2} 4+5 9-2 7/7 8*8 .
    #[test]
    fn it_works() {
        let mut lexer = Token::lexer("Create ridiculously fast Lexers.");

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
