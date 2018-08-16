extern crate radix_trie;
use self::radix_trie::Trie;

#[derive(Debug, PartialEq, Eq)]
pub struct NoCloneTok(pub Tok);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    LBRACK,
    RBRACK,
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN,
    COLON,
    SEMICOLON,
    COMMA,
    EQ,
    ARROW,
    EQUALS,
    NOTEQUALS,
    GT,
    LT,
    PIPE,
    QMARK,
    HASH,
    PLUS,
    MINUS,
    TIMES,
    DIVIDE,
    DOTDOT,
    DOT,
    LROW,
    RROW,
    LCOL,
    RCOL,
    KFUN,
    KLET,
    KVAR,
    KIF,
    KELSE,
    KWHILE,
    KFOR,
    KDO,
    KEND,
    KIN,

    DIGITS(String),
    I32SUFFIX,
    I64SUFFIX,

    CONSTANT(String),
    STRINGLITERAL(String),

    NAME(String),
    TYPENAME(String),
}
lazy_static! {
    static ref TRIE: Trie<&'static str, Tok> = {
        let mut kws = Trie::new();
        kws.insert("[", Tok::LBRACK);
        kws.insert("]", Tok::RBRACK);
        kws.insert("{", Tok::LBRACE);
        kws.insert("}", Tok::RBRACE);
        kws.insert("(", Tok::LPAREN);
        kws.insert(")", Tok::RPAREN);
        kws.insert(":", Tok::COLON);
        kws.insert(";", Tok::SEMICOLON);
        kws.insert(",", Tok::COMMA);
        kws.insert("=", Tok::EQ);
        kws.insert("->", Tok::ARROW);
        kws.insert("==", Tok::EQUALS);
        kws.insert("!=", Tok::NOTEQUALS);
        kws.insert(">", Tok::GT);
        kws.insert("<", Tok::LT);
        kws.insert("|", Tok::PIPE);
        kws.insert("?", Tok::QMARK);
        kws.insert("#", Tok::HASH);
        kws.insert("+", Tok::PLUS);
        kws.insert("-", Tok::MINUS);
        kws.insert("*", Tok::TIMES);
        kws.insert("/", Tok::DIVIDE);
        kws.insert("..", Tok::DOTDOT);
        kws.insert(".", Tok::DOT);
        kws.insert("[<", Tok::LROW);
        kws.insert(">]", Tok::RROW);
        kws.insert("[|", Tok::LCOL);
        kws.insert("|]", Tok::RCOL);
        kws.insert("fun", Tok::KFUN);
        kws.insert("let", Tok::KLET);
        kws.insert("var", Tok::KVAR);
        kws.insert("if", Tok::KIF);
        kws.insert("else", Tok::KELSE);
        kws.insert("while", Tok::KWHILE);
        kws.insert("for", Tok::KFOR);
        kws.insert("do", Tok::KDO);
        kws.insert("end", Tok::KEND);
        kws.insert("in", Tok::KIN);
        kws.insert("i32", Tok::I32SUFFIX);
        kws.insert("i64", Tok::I64SUFFIX);
        kws
    };
}

// simplest and stupidest possible tokenizer
pub fn tokenize(s: &str) -> Vec<(usize, Tok, usize)> {
    let mut tokens = vec![];
    let mut chars = s.chars();
    let mut lookahead = chars.next();
    let mut i = 0;
    while let Some(c) = lookahead {
        // skip whitespace characters
        if !c.is_whitespace() {
            match c {
                _ if c.is_digit(10) => {
                    let (tmp, next) = take_while(c, &mut chars, |c| c.is_digit(10));
                    lookahead = next;
                    let newi = i + tmp.len();
                    tokens.push((i, Tok::DIGITS(tmp), newi));
                    i = newi;
                    continue;
                }
                // _ if c.is_upper()
                /* CONSTANT */
                /* STRINGLITERAL */
                /* NAME */
                /* TYPENAME */
                /* COMMENT */
                _ => {
                    let (tmp, next) =
                        take_while_buf(c, &mut chars, |s| TRIE.subtrie(&s).is_some());

                    if tmp.len() == 0 || !TRIE.subtrie(tmp.as_str()).is_some() {
                        panic!("invalid token: {:?}, seen tokens: {:?}", tmp, tokens);
                    }

                    lookahead = next;
                    let tok = TRIE.get(tmp.as_str()).unwrap().clone();
                    let newi = i + tmp.len();
                    tokens.push((i, tok, newi));
                    i = newi;
                    continue;
                }
            }
        }
        else {
            i += 1;
        }

        // advance to next character by default
        lookahead = chars.next();
    }

    tokens
}

fn take_while<C, F>(c0: char, chars: &mut C, f: F) -> (String, Option<char>)
where
    C: Iterator<Item = char>,
    F: Fn(char) -> bool,
{
    let mut buf = String::new();

    buf.push(c0);

    while let Some(c) = chars.next() {
        if !f(c) {
            return (buf, Some(c));
        }

        buf.push(c);
    }

    return (buf, None);
}

fn take_while_buf<C, F>(c0: char, chars: &mut C, f: F) -> (String, Option<char>)
where
    C: Iterator<Item = char>,
    F: Fn(&str) -> bool,
{
    let mut buf = String::new();

    buf.push(c0);

    while let Some(c) = chars.next() {
        if !f(&(buf.clone() + &c.to_string())) {
            return (buf.to_string(), Some(c));
        }

        buf.push(c);
    }

    return (buf, None);
}
