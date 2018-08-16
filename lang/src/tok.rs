extern crate radix_trie;
use self::radix_trie::Trie;

pub type Spanned<T> = (usize, T, usize);

#[derive(Debug, PartialEq, Eq)]
pub struct NoCloneTok(pub Tok);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub location: usize,
    pub code: ErrorCode,
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

enum CommentState {
    OutOfComment,
    InComment,
    SeekingSlash,
}

// simplest and stupidest possible tokenizer
pub fn tokenize(s: &str) -> Vec<Spanned<Tok>> {
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
                },
                _ if c.is_alphanumeric() && c.is_uppercase() => {
                    let typename = |c:char| c.is_alphanumeric() || c == '_';
                    let (tmp, next) = take_while(c, &mut chars, typename);
                    lookahead = next;
                    let newi = i + tmp.len();
                    if tmp.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') {
                        tokens.push((i, Tok::CONSTANT(tmp), newi));
                    }
                    else {
                        tokens.push((i, Tok::TYPENAME(tmp), newi));
                    }
                    i = newi;
                    continue;
                },
                _ if c.is_alphanumeric() && c.is_lowercase() => {
                    let name = |c:char| c.is_alphanumeric() || c == '_';
                    let (tmp, next) = take_while(c, &mut chars, name);
                    lookahead = next;
                    let newi = i + tmp.len();
                    if let Some(v) = TRIE.get(tmp.as_str()) {
                        // It's a keyword
                        tokens.push((i, v.clone(), newi));
                    }
                    else {
                        // It's a variable
                        tokens.push((i, Tok::NAME(tmp), newi));
                    }
                    i = newi;
                    continue;
                },
                '"' => {
                    /* STRING LITERAL */
                    let (mut tmp, next) = take_while(c, &mut chars, |c| c != '"');
                    lookahead = chars.next(); // skip the last quote
                    tmp.push('"');
                    let newi = i + tmp.len();
                    tokens.push((i, Tok::STRINGLITERAL(tmp), newi));
                    i = newi;
                    continue;
                },
                '/' => {
                    /* COMMENT */
                    use self::CommentState::*;
                    let mut state = OutOfComment;

                    loop {
                        lookahead = chars.next();
                        state = match (lookahead, state) {
                            (Some('*'), OutOfComment) => InComment,
                            (Some('*'), InComment) => SeekingSlash,
                            (Some('/'), SeekingSlash) => { break; },
                            (Some(_), SeekingSlash) | (Some(_), InComment) =>
                                InComment,
                            (Some(_), OutOfComment) =>
                                panic!("Should not happen"),
                            (None, _) =>
                                panic!("Premature EOF in comment"),
                        }
                    }
                }
                _ => {
                    let (tmp, next) =
                        take_while_buf(c, &mut chars, |s| TRIE.get_raw_descendant(&s).is_some());

                    if tmp.len() == 0 || !TRIE.get(tmp.as_str()).is_some() {
                        panic!("invalid token: {:?}, seen tokens: {:?}", tmp, tokens);
                    }

                    lookahead = next;
                    let tok = TRIE.get(tmp.as_str()).unwrap().clone();
                    let newi = i + tmp.len();
                    tokens.push((i, tok, newi));
                    i = newi;
                    continue;
                },

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
