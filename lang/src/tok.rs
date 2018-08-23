extern crate radix_trie;
use self::radix_trie::Trie;

use std::iter::Iterator;
use std::str::Chars;


#[derive(Debug, PartialEq, Eq)]
pub struct NoCloneTok(pub Tok);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub location: usize,
    pub code: ErrorCode,
}

use std::fmt;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {:?} at position {}", self.code, self.location)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    UnrecognizedToken,
    UnterminatedEscape,
    UnterminatedStringLiteral,
    UnterminatedCharacterLiteral,
    UnterminatedAttribute,
    UnterminatedCode,
    ExpectedStringLiteral,
}

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
    TICK,
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
    KTYPE,
    KOF,

    DIGITS(String),
    I32SUFFIX,
    I64SUFFIX,

    CONSTANT(String),
    STRINGLITERAL(String),

    NAME(String),
    TYPENAME(String),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TablamTokenizer<'input> {
    chars: Chars<'input>,
    lookahead: Option<char>,
    i: usize,
}
enum CommentState {
    OutOfComment,
    InComment,
    SeekingSlash,
}

impl<'input> TablamTokenizer<'input> {
    pub fn new(s: &'input str) -> TablamTokenizer<'input> {
        TablamTokenizer { chars: s.chars(), lookahead: None, i: 0 }
    }

    fn skip_ws(&mut self) -> () {
        if Some(true) == self.lookahead.map(|c| !c.is_whitespace()) {
            return;
        }

        loop {
            self.lookahead = self.chars.next();
            match self.lookahead {
                None => { break; }
                Some(c) if !c.is_whitespace() => { break; }
                _ => { continue; }
            }
        }
    }
}

pub type Spanned<T> = (usize, T, usize);

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
        kws.insert("'", Tok::TICK);
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
        kws.insert("type", Tok::KTYPE);
        kws.insert("of", Tok::KOF);
        kws.insert("i32", Tok::I32SUFFIX);
        kws.insert("i64", Tok::I64SUFFIX);
        kws
    };
}

impl<'input> Iterator for TablamTokenizer<'input> {
    type Item = Spanned<Tok>;
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();
        if self.lookahead.is_none() { return None; }
        let c = self.lookahead.unwrap();

        let name = |c:char| c.is_alphanumeric() || c == '_';
        match c {
            _ if c.is_digit(10) => {
                let (tmp, next) = take_while(c, &mut self.chars, |c| c.is_digit(10));
                self.lookahead = next;
                let newi = self.i + tmp.len();
                let t = (self.i, Tok::DIGITS(tmp), newi);
                self.i = newi;
                Some(t)
            },
            _ if c.is_alphanumeric() && c.is_uppercase() => {
                let (tmp, next) = take_while(c, &mut self.chars, name);
                self.lookahead = next;
                let newi = self.i + tmp.len();
                let constlike = |c:char| c.is_uppercase() || c.is_numeric() || c == '_';
                let t = if tmp.chars().all(constlike) {
                    (self.i, Tok::CONSTANT(tmp), newi)
                }
                else {
                    (self.i, Tok::TYPENAME(tmp), newi)
                };
                self.i = newi;
                Some(t)
            },
            _ if c.is_alphanumeric() && c.is_lowercase() => {
                let (tmp, next) = take_while(c, &mut self.chars, name);
                self.lookahead = next;
                let newi = self.i + tmp.len();
                let t = match TRIE.get(tmp.as_str()) {
                    // It's a keyword
                    Some(v) => (self.i, v.clone(), newi),
                    // It's a variable
                    _ => (self.i, Tok::NAME(tmp), newi),
                };
                self.i = newi;
                Some(t)
            },
            '"' => {
                /* STRING LITERAL */
                let (mut tmp, next) = take_while(c, &mut self.chars, |c| c != '"');
                self.lookahead = self.chars.next(); // skip the last quote
                tmp.push('"');
                let newi = self.i + tmp.len();
                let t = (self.i, Tok::STRINGLITERAL(tmp), newi);
                self.i = newi;
                Some(t)
            },
            '/' => {
                /* COMMENT */
                use self::CommentState::*;
                let mut state = OutOfComment;

                loop {
                    self.lookahead = self.chars.next();
                    state = match (self.lookahead, state) {
                        (Some('*'), OutOfComment) => InComment,
                        (Some('*'), InComment) => SeekingSlash,
                        (Some('/'), SeekingSlash) => { break; },
                        (Some(_), SeekingSlash) | (Some(_), InComment) =>
                            InComment,
                        (Some(c), OutOfComment) => { break; },
                        (None, _) => panic!("Premature EOF in comment"),
                    }
                }

                self.next()
            },
            _ => {
                let (tmp, next) =
                    take_while_buf(c, &mut self.chars, |s| TRIE.get_raw_descendant(&s).is_some());

                if tmp.len() == 0 || !TRIE.get(tmp.as_str()).is_some() {
                    panic!("invalid token: {:?}", tmp);
                }

                self.lookahead = next;
                let tok = TRIE.get(tmp.as_str()).unwrap().clone();
                let newi = self.i + tmp.len();
                let t = (self.i, tok, newi);
                self.i = newi;
                Some(t)
            },
        }
    }
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
