pub use std::rc::Rc;
use std::collections::HashMap;
//use std::fmt;

use tablam_core::types as TT;

#[derive(Clone)]
pub struct SourceMap {
    pub row:    u32,
    pub col:    u32,
    pub module: u32,
}

type Return = Result<TT::Scalar, String>;
type ReturnUnit = Result<(), String>;
type RExpr = Rc<Expr>;

#[derive(Debug, Clone)]
pub struct Env {
    pub vars: HashMap<String, TT::Scalar>,
    pub up: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Env {
        Env { vars: HashMap::new(), up: None }
    }

    pub fn add(&mut self, k: String, v: TT::Scalar) {
        self.vars.insert(k, v);
    }

    pub fn find(&self, k: String) -> Option<&TT::Scalar> {
        // TODO: make this use recursive envs
        self.vars.get(&k)
    }

    pub fn add_many(&self, k: Vec<String>, v: Vec<TT::Scalar>) -> Env {
        let mut copy = self.clone();
        for (k, v) in k.into_iter().zip(v.iter()) {
            copy.vars.insert(k, v.clone());
        }
        copy
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LetKind {
    Imm, Mut,
}

#[derive(Debug, Clone)]
pub struct FunDef {
    pub name: String,
    pub params: Vec<(String, TT::DataType)>,
    pub ret_ty: TT::DataType,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolExpr {
    Const(bool),
    Cmp(CmOp),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmOp {
    pub op:  TT::CompareOp,
    pub lhs: RExpr,
    pub rhs: Option<RExpr>
}

pub type ExprList = Vec<Expr>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    //Values
    Pass,
    Value(Rc<TT::Scalar>),
    Block(ExprList),
    //Control flow
    If(BoolExpr, RExpr, Option<RExpr>),
//    While(BoolExpr, Expr),
//    For(Expr, Expr),
//    Match(Expr, Expr),
    //Operators
//    RelOp(TT::RelOp, Expr),
//    IndexOp(TT::IndexOp, Expr),
//    BinOp(TT::BinOp, Expr, Expr),
    CmpOp(CmOp),
    //Vars
//    Let(LetKind, String, Expr),
}

impl <T> From<T> for Expr
    where T: From<TT::Scalar>, TT::Scalar: From<T>
{
    fn from(of: T) -> Self {
        Expr::Value(Rc::new(of.into()))
    }
}


pub fn pass() -> Expr {
    Expr::Pass
}

pub fn cmp(op:TT::CompareOp, lhs:RExpr, rhs:Option<RExpr>) -> Expr {
    Expr::CmpOp(CmOp{
        op, lhs, rhs
    })
}

pub fn eq(lhs:RExpr, rhs:Option<RExpr>) -> Expr {
    cmp(TT::CompareOp::Eq, lhs, rhs)
}

pub fn not_eq(lhs:RExpr, rhs:Option<RExpr>) -> Expr {
    cmp(TT::CompareOp::NotEq, lhs, rhs)
}

pub fn eif(of:bool, if_true:RExpr, if_false:Option<RExpr>) -> Expr {
    Expr::If(BoolExpr::Const(of), if_true, if_false)
}

pub fn eif_cmp(of:CmOp, if_true:RExpr, if_false:Option<RExpr>) -> Expr {
    Expr::If(BoolExpr::Cmp(of), if_true, if_false)
}
