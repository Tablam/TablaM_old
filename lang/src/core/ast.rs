pub use core::types::*;
pub use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    IfElse(Rc<Exp>, Rc<Exp>, Rc<Exp>),
    If(Rc<Exp>, Rc<Exp>),
    While(Rc<Exp>, Rc<Exp>),
    Let(LetKind, String, Rc<Exp>),
    Exp(Exp),
}

#[derive(Debug, PartialEq, Clone)]
pub enum LetKind {
    Imm, Mut,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Exp {
    Unit,
    Scalar(Scalar),
    Column(ColumnExp),
    Row(RowExp),
    Range(RangeExp),
    Name(String),
    BinOp(BinOp, Rc<Exp>, Rc<Exp>),
    Apply(Rc<Exp>, Vec<Exp>),
    Block(Vec<Stmt>, Rc<Exp>),
}
/* BEGIN easy From for testing */
impl From<bool> for Exp {
    fn from(b: bool) -> Self {
        match b {
            true => Exp::Name("true".into()),
            false => Exp::Name("false".into()),
        }
    }
}

impl From<i32> for Exp {
    fn from(i: i32) -> Self {
        Exp::Scalar(i.into())
    }
}

impl From<i64> for Exp {
    fn from(i: i64) -> Self {
        Exp::Scalar(i.into())
    }
}

impl From<Exp> for Stmt {
    fn from(e: Exp) -> Self {
        Stmt::Exp(e)
    }
}
/* END easy From for testing */

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnExp {
    pub name: Option<String>,
    pub ty: Option<String>,
    pub es: Vec<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RowExp {
    pub names: Option<Vec<String>>,
    pub es: Vec<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RangeExp {
    pub start: Rc<Exp>,
    pub end: Rc<Exp>,
}

//    let e = ExpAt {
//        line: 5,
//        exp: Exp::BinOp(BinOp::Plus,
//                        &ExpAt {
//                            line: 5,
//                            exp: Exp::Scalar(Scalar::I32(3)),
//                        },
//                        &ExpAt {
//                            line: 5,
//                            exp: Exp::Scalar(Scalar::I64(4)),
//                        })
//    };
// #[derive(Debug)]
// pub enum Exp<T> {
//     Scalar(Scalar),
//     BinOp(BinOp, T, T),
// }
// 
// #[derive(Debug)]
// pub struct ExpAt<'a> {
//     pub line: i32,
//     pub exp: Exp<&'a ExpAt<'a>>,
// }
