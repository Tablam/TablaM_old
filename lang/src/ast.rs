use std::rc::Rc;
use std::cell::RefCell;
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
pub type RExpr = Rc<Expr>;
pub type BExpr = Box<Expr>;
pub type RScalar = Rc<TT::Scalar>;

#[derive(Debug, Clone)]
pub struct Env {
    pub vars: HashMap<String, (LetKind, Value)>,
    pub up: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Env {
        Env { vars: HashMap::new(), up: None }
    }

    pub fn add(&mut self, kind:LetKind, k: String, v: Value) {
        self.vars.insert(k, (kind,v));
    }

    pub fn find(&self, k: &String) -> Option<&(LetKind, Value)> {
        // TODO: make this use recursive envs
        self.vars.get(k)
    }

    pub fn add_many(&self, k: Vec<String>, v: Vec<(LetKind, Value)>) -> Env {
        let mut copy = self.clone();
        for (k, v) in k.into_iter().zip(v.iter()) {
            copy.vars.insert(k, v.clone());
        }
        copy
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub env: Rc<RefCell<Env>>
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
    pub lhs: Value,
    pub rhs: Value
}

pub type ExprList = Vec<BExpr>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Var(String),
    Value(RScalar),
    //Fun(RScalar),
    SideEffect(BExpr)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    //Values
    Break,
    Continue,
    Pass,
    Value(Value),
    Block(ExprList),
    //Control flow
    If(BoolExpr, BExpr, BExpr),
    While(BoolExpr, BExpr),
//    For(Expr, BExpr),
//    Match(Expr, BExpr),
    //Operators
//    RelOp(TT::RelOp, BExpr),
//    IndexOp(TT::IndexOp, BExpr),
//    BinOp(TT::BinOp, BExpr, BExpr),
    CmpOp(CmOp),
    //Vars
    Var(String),
    Let(LetKind, String, Value),
}

impl <T> From<T> for Value
    where T: From<TT::Scalar>, TT::Scalar: From<T>
{
    fn from(of: T) -> Self {
        Value::Value(Rc::new(of.into()))
    }
}

impl <T> From<T> for Expr
    where T: From<TT::Scalar>, TT::Scalar: From<T>
{
    fn from(of: T) -> Self {
        Expr::Value(of.into())
    }
}

impl From<Expr> for Value
{
    fn from(of: Expr) -> Self {
        Value::SideEffect(Box::new(of))
    }
}

impl From<CmOp> for Expr
{
    fn from(of: CmOp) -> Self {
        Expr::CmpOp(of)
    }
}

pub fn pass() -> Expr {
    Expr::Pass
}

pub fn lines(of:Vec<Expr>) -> Expr {
    Expr::Block(of.into_iter().map(|x| x.into()).collect())
}

pub fn get_value(of: &Expr) -> Option<&Value> {
    match of {
        Expr::Value(value) => Some(value),
        _   => None
    }
}

pub fn var(name:&str) -> Value {
    Value::Var(name.to_string())
}

pub fn set_var_imm(name:&str, of:Value) -> Expr {
    Expr::Let(LetKind::Imm, name.to_string(), of)
}

pub fn set_var_mut(name:&str, of:Value) -> Expr {
    Expr::Let(LetKind::Mut, name.to_string(), of)
}

pub fn cmp(op:TT::CompareOp, lhs:Value, rhs:Value) -> CmOp {
    CmOp{
        op, lhs, rhs
    }
}

pub fn eq(lhs:Value, rhs:Value) -> CmOp {
    cmp(TT::CompareOp::Eq, lhs, rhs)
}

pub fn not_eq(lhs:Value, rhs:Value) -> CmOp {
    cmp(TT::CompareOp::NotEq, lhs, rhs)
}

pub fn less(lhs:Value, rhs:Value) -> CmOp {
    cmp(TT::CompareOp::Less, lhs, rhs)
}

pub fn eif(of:bool, if_true:BExpr, if_false:BExpr) -> Expr {
    Expr::If(BoolExpr::Const(of), if_true, if_false)
}

pub fn eif_cmp(of:CmOp, if_true:BExpr, if_false:BExpr) -> Expr {
    Expr::If(BoolExpr::Cmp(of), if_true, if_false)
}

pub fn ewhile(of:bool, body:BExpr) -> Expr {
    Expr::While(BoolExpr::Const(of), body)
}

pub fn ewhile_cmp(of:CmOp, body:BExpr) -> Expr {
    Expr::While(BoolExpr::Cmp(of), body)
}