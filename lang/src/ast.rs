use std::rc::Rc;
use std::collections::HashMap;
//use stdlib::fmt;

use tablam_core::types as TT;
use tablam_core::dsl as DD;

#[derive(Clone)]
pub struct SourceMap {
    pub row:    u32,
    pub col:    u32,
    pub module: u32,
}

pub type RExpr = Rc<Expr>;
pub type BExpr = Box<Expr>;
pub type RScalar = Rc<TT::Scalar>;
pub type ParamsCall = HashMap<String, RExpr>;
pub type Return = Result<Expr, Failed>;
pub type ReturnScalar = Result<RScalar, Failed>;
pub type ReturnBool = Result<bool, Failed>;

pub type ExprList = Vec<BExpr>;
pub type ExprSlice<'a> = &'a [BExpr];

#[derive(Debug, Clone)]
pub struct Env {
    pub vars: HashMap<String, (LetKind, Value)>,
    pub fun: HashMap<String, FunDef>,
    pub up: Option<Box<Env>>,
}

impl Env {
    fn create(parent:Option<Box<Env>>) -> Self {
        Env { vars: HashMap::new(), fun: HashMap::new(), up:parent}
    }
    pub fn empty() -> Self {
        Self::create(None)
    }
    pub fn child(of:Env) -> Self {
        Self::create(Some(of.into()))
    }

    pub fn add_var(&mut self, kind:LetKind, k: &str, v: Value) {
        self.vars.insert(k.into(), (kind,v));
    }
    pub fn add_fun(&mut self, def: FunDef) {
        self.fun.insert(def.name.clone(), def);
    }

    pub fn find_var(&self, k: &str) -> Option<&(LetKind, Value)> {
        match self.vars.get(k) {
            Some(var) => Some(var),
            None => {
                match &self.up {
                    Some(env) => env.find_var(k),
                    None => None
                }
            }
        }
    }

    pub fn find_fun(&self, k: &str) -> Option<&FunDef> {
        match self.fun.get(k) {
            Some(var) => Some(var),
            None => {
                match &self.up {
                    Some(env) => env.find_fun(k),
                    None => None
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
}

#[derive(Debug, PartialEq, Clone)]
pub enum LetKind {
    Imm, Mut,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunCall {
    pub name: String,
    pub params: ParamsCall,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunDef {
    pub name: String,
    pub params: TT::Schema,
    pub ret_ty: TT::DataType,
    pub body: Option<BExpr>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct BinOp {
    pub op:  TT::BinOp,
    pub lhs: Value,
    pub rhs: Value
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fail {
    pub msg: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Failed {
    Runtime(Fail),
}

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
    While(BoolExpr, ExprList),
    ForI(String, TT::Range, ExprList),
//    ForEach(String, TT::Range),
//    Match(Expr, BExpr),
    //Operators
//    RelOp(TT::RelOp, BExpr),
//    IndexOp(TT::IndexOp, BExpr),
    BinOp(BinOp),
    CmpOp(CmOp),
    //Vars
    Var(String),
    Let(LetKind, String, Value),
    //Functions
    Fun(FunDef),
    Call(FunCall),
    //Exceptions,
    Fail(Failed),
}

impl Expr {
    pub fn is_break(&self) -> bool {
        self == &Expr::Break
    }

    pub fn is_continue(&self) -> bool {
        self == &Expr::Continue
    }

    pub fn is_loop_control(&self) -> bool {
        self.is_break() || self.is_continue()
    }
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

pub fn lines(of:Vec<Expr>) -> ExprList {
   of.into_iter().map(|x| x.into()).collect()
}

pub fn block(of:ExprList) -> Expr {
    Expr::Block(of)
}

pub fn block_lines(of:Vec<Expr>) -> Expr {
    block(lines(of))
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

pub fn evar(name:&str) -> Expr {
    Expr::Var(name.to_string())
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

pub fn ewhile(of:bool, body:ExprList) -> Expr {
    Expr::While(BoolExpr::Const(of), body)
}

pub fn ewhile_cmp(of:CmOp, body:ExprList) -> Expr {
    Expr::While(BoolExpr::Cmp(of), body)
}

pub fn efor_step(name:&str, start:isize, end:isize, step:usize, body:ExprList) -> Expr {
    let range = TT::Range::new(start, end, step);
    Expr::ForI(name.to_string(), range, body)
}

pub fn efor(name:&str, start:isize, end:isize, body:ExprList) -> Expr {
    let range = TT::Range::new(start, end, 1);
    Expr::ForI(name.to_string(), range, body)
}

pub fn bin_op(op:TT::BinOp, lhs:Value, rhs:Value) -> Expr {
    Expr::BinOp(BinOp{op, lhs, rhs})
}

pub fn plus_op(lhs:Value, rhs:Value) -> Expr {
    bin_op(TT::BinOp::Add, lhs, rhs)
}

pub fn minus_op(lhs:Value, rhs:Value) -> Expr {
    bin_op(TT::BinOp::Minus, lhs, rhs)
}

pub fn div_op(lhs:Value, rhs:Value) -> Expr {
    bin_op(TT::BinOp::Div, lhs, rhs)
}

pub fn mul_op(lhs:Value, rhs:Value) -> Expr {
    bin_op(TT::BinOp::Mul, lhs, rhs)
}

pub fn fun_def(name:&str, pars:&[(&str, TT::DataType)], ret_ty:TT::DataType, body: BExpr) -> Expr {
    let params = DD::schema(pars);

    Expr::Fun(FunDef{
        name: name.to_string(),
        params,
        ret_ty,
        body: Some(body)
    })
}

pub fn fun_call(name:&str, pars:&[(&str, RExpr)]) -> Expr {
    let mut params = HashMap::with_capacity(pars.len());

    for (name, value) in pars {
        params.insert(name.to_string(), value.clone());
    }

    Expr::Call(FunCall{
        name: name.to_string(),
        params,
    })
}