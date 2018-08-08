pub use core::types::*;
pub use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Exp {
    Scalar(Scalar),
    Column(ColumnExp),
    Range(RangeExp),
    Name(String),
    BinOp(BinOp, Rc<Exp>, Rc<Exp>),
    LetImm(String, Rc<Exp>),
    LetMut(String, Rc<Exp>),
}

#[derive(Debug, PartialEq)]
pub struct ColumnExp {
    pub name: Option<String>,
    pub ty: Option<String>,
    pub es: Vec<Exp>,
}

#[derive(Debug, PartialEq)]
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
