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
pub struct Prog {
    pub blocks: Vec<ProgBlock>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProgBlock {
    Function(FunDef),
    Constant(String, Ty, Rc<Exp>),
    TypeDeclaration(Ty, Vec<(String, Vec<Atype>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunDef {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret_ty: Ty,
    pub body: Exp,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ty {
    Tycon(String, Vec<String>),
    Arrow(Rc<Ty>, Rc<Option<Ty>>),
    TyApp(Rc<Ty>, Rc<Atype>),
    Atype(Rc<Atype>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Atype {
    // Gtycon(Gtycon),
    Conid(String),
    Tyvar(String),
    Tuple(Vec<Ty>),
    List(Ty),
    Paren(Ty),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Gtycon {
    Conid(String), Unit, List, Function,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    IfElse(Rc<Exp>, Rc<Exp>, Rc<Exp>),
    If(Rc<Exp>, Rc<Exp>),
    While(Rc<Exp>, Rc<Stmt>),
    For(String, Rc<Exp>, Rc<Stmt>),
    Let(LetKind, String, Option<Ty>, Rc<Exp>),
    Block(Vec<Stmt>),
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
    Container(Ty, ColumnExp),
    ColumnSelect(Rc<Exp>, ColumnSelector),
    Column(ColumnExp),
    Row(RowExp),
    Relation(RelationExp),
    Range(RangeExp),
    Name(String),
    Constant(String),
    BinOp(BinOp, Rc<Exp>, Rc<Exp>),
    Apply(Rc<Exp>, Vec<Exp>),
    Block(Vec<Stmt>, Rc<Exp>),
    QueryFilter(Rc<Exp>, Vec<FilterExp>),
    QuerySelect(Rc<Exp>, Rc<Exp>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnSelector {
    Name(String),
    Num(u32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnExp {
    pub name: Option<String>,
    pub ty: Option<Ty>,
    pub es: Vec<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RowExp {
    pub names: Option<Vec<String>>,
    pub types: Vec<Option<Ty>>,
    pub es: Vec<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RelationExp {
    pub rel_type: RelType,
    pub names: Vec<String>,
    pub data: Vec<Vec<Exp>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelType {
    Row, Col
}

#[derive(Debug, PartialEq, Clone)]
pub struct RangeExp {
    pub start: Rc<Exp>,
    pub end: Rc<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelOp {
    Equals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    NotEquals,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExp {
    RelOp(RelOp, String, Rc<Exp>),
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

impl<'a> From<&'a str> for ColumnSelector {
    fn from(s: &'a str) -> Self {
        ColumnSelector::Name(s.to_owned())
    }
}

impl From<u32> for ColumnSelector {
    fn from(s: u32) -> Self {
        ColumnSelector::Num(s)
    }
}

impl From<Exp> for Stmt {
    fn from(e: Exp) -> Self {
        Stmt::Exp(e)
    }
}
/* END easy From for testing */


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