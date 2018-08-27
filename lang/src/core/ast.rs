pub use core::types::*;
pub use std::rc::Rc;
use std::collections::HashMap;
use std::fmt;

type Return = Result<Frame, String>;
type ReturnUnit = Result<(), String>;

#[derive(Clone)]
pub struct Env {
    pub vars: HashMap<String, Frame>,
    pub up: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Env {
        Env { vars: HashMap::new(), up: None }
    }

    pub fn add(&mut self, k: String, v: Frame) {
        self.vars.insert(k, v);
    }

    pub fn find(&self, k: String) -> Option<&Frame> {
        // TODO: make this use recursive envs
        self.vars.get(&k)
    }

    pub fn add_many(&self, k: Vec<String>, v: Vec<Frame>) -> Env {
        let mut copy = self.clone();
        for (k, v) in k.into_iter().zip(v.iter()) {
            copy.vars.insert(k, v.clone());
        }
        copy
    }
}

// pub struct Env<'a> {
//     pub vars: HashMap<&'a String, Frame>,
//     pub up: Option<Rc<Env<'a>>>,
// }
// 
// impl<'a> Env<'a> {
//     pub fn empty() -> Env<'a> {
//         Env { vars: HashMap::new(), up: None }
//     }
// 
//     pub fn add(&mut self, k: String, v: Frame) {
//         self.vars.insert(k, v);
//     }
// 
//     pub fn add_many(&self, k: Vec<&'a String>, v: Vec<Frame>) -> Env<'a> {
//         let mut copy = self.clone();
//         for (k, v) in k.into_iter().zip(v.iter()) {
//             copy.vars.insert(k, v.clone());
//         }
//         copy
//     }
// }

pub trait Runnable<T: fmt::Debug> {
    fn run(&self, args: Vec<Exp<T>>, env: &Env) -> Return;
}

pub trait RunnableMut<T: fmt::Debug> {
    fn run_mut(&self, args: Vec<Exp<T>>, env: &mut Env) -> ReturnUnit;
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Prog<T> {
    pub blocks: Vec<ProgBlock<T>>,
}

impl<T> Prog<T> {
    fn find_main(&self) -> Result<&FunDef<T>, String> {
        let main : Vec<&FunDef<T>> = self.blocks.iter()
            .flat_map(|b| b.extract_function())
            .filter(|f| f.name == "main")
            .collect();
        if main.len() > 1 {
            Err("Too many main functions".into())
        }
        else if main.len() < 1 {
            Err("Did not find a a main function".into())
        }
        else {
            Ok(main[0])
        }
    }
}

impl<T: fmt::Debug> Runnable<T> for Prog<T> {
    fn run(&self, args: Vec<Exp<T>>, env: &Env) -> Return {
        self.find_main().and_then(|main| main.run(args, env))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProgBlock<T> {
    Function(FunDef<T>),
    Constant(String, Ty, Rc<Exp<T>>),
    TypeDeclaration(Ty, Vec<(String, Vec<Atype>)>),
}

impl<T> ProgBlock<T> {
    pub fn extract_function(&self) -> Option<&FunDef<T>> {
        match self {
            ProgBlock::Function(f) => Some(f),
            _ => None,
        }
    }
}

// impl<T: fmt::Debug> RunnableMut<T> for ProgBlock<T> {
//     fn run_mut(&self, args: Vec<Exp<T>>, env: &Env) -> ReturnUnit {
//         match self {
//             ProgBlock::Function(f) => (f.run(args, env), env),
//             _ => Err("Cannot run a non-function".into()),
//         }
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct FunDef<T> {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret_ty: Ty,
    pub body: Exp<T>,
}

impl<T: fmt::Debug> Runnable<T> for FunDef<T> {
    fn run(&self, args: Vec<Exp<T>>, env: &Env) -> Return {
        let mut arg_values_results =
            args.iter().map(|e| e.run(vec!(), env));
        if arg_values_results.any(|x| x.is_err()) {
            return arg_values_results.find(|x| x.is_err()).unwrap()
        }
        let arg_values = arg_values_results.flat_map(|x| x.ok()).collect();
        let fn_env = env.add_many(
            self.params.iter().map(|(n, t)| n.to_string()).collect(),
            arg_values
        );
        self.body.run(vec!(), &fn_env)
    }
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
pub enum Stmt<T> {
    IfElse(Rc<Exp<T>>, Rc<Exp<T>>, Rc<Exp<T>>),
    If(Rc<Exp<T>>, Rc<Exp<T>>),
    While(Rc<Exp<T>>, Rc<Stmt<T>>),
    For(String, Rc<Exp<T>>, Rc<Stmt<T>>),
    Let(LetKind, String, Option<Ty>, Rc<Exp<T>>),
    Block(Vec<Stmt<T>>),
    Exp(Exp<T>),
}

impl<T: fmt::Debug> RunnableMut<T> for Stmt<T> {
    fn run_mut(&self, args: Vec<Exp<T>>, env: &mut Env) -> ReturnUnit {
        match self {
            Stmt::Let(LetKind::Imm, n, _, e) => {
                let v = e.run(vec!(), env)?;
                env.add(n.to_owned(), v);
                Ok(())
            },
            x => Err(format!("Not implemented yet: {:?}", x)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LetKind {
    Imm, Mut,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Exp<T> {
    Thing(T),
    Unit,
    Scalar(Scalar),
    Container(Ty, ColumnExp<T>),
    ColumnSelect(Rc<Exp<T>>, ColumnSelector),
    Column(ColumnExp<T>),
    Row(RowExp<T>),
    Relation(RelationExp<T>),
    Range(RangeExp<T>),
    Name(String),
    Constant(String),
    BinOp(BinOp, Rc<Exp<T>>, Rc<Exp<T>>),
    Apply(Rc<Exp<T>>, Vec<Exp<T>>),
    Block(Vec<Stmt<T>>, Rc<Exp<T>>),
    QueryFilter(Rc<Exp<T>>, Vec<FilterExp<T>>),
    QuerySelect(Rc<Exp<T>>, Rc<Exp<T>>),
}

impl<T: fmt::Debug> Runnable<T> for Exp<T> {
    fn run(&self, args: Vec<Exp<T>>, env: &Env) -> Return {
        match self {
            Exp::Scalar(s) => Ok(s.clone().into()),
            Exp::Name(n) if env.vars.get(n).is_some() =>
                Ok(env.find(n.to_string()).unwrap().clone()),
            Exp::Name(n) =>
                Err(format!("Could not find variable {:?} in env", n)),
            Exp::BinOp(op, e1, e2) => {
                let v1 = e1.run(vec!(), &env)?;
                let v2 = e2.run(vec!(), &env)?;
                match op {
                    BinOp::Plus => Ok(v1 + v2),
                    // BinOp::Minus => v1 - v2,
                    // BinOp::Times => v1 * v2,
                    // BinOp::Divide => v1 / v2,
                    o => Err(format!("Not implemented yet: {:?}", o))
                }
            },
            Exp::Block(stmts, e) => {
                let mut block_env = env.clone();
                for stmt in stmts {
                    stmt.run_mut(vec!(), &mut block_env)?;
                }
                e.run(vec!(), &block_env)
            },
            x => Err(format!("Not implemented yet: {:?}", x)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnSelector {
    Name(String),
    Num(u32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnExp<T> {
    pub name: Option<String>,
    pub ty: Option<Ty>,
    pub es: Vec<Exp<T>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RowExp<T> {
    pub names: Option<Vec<String>>,
    pub types: Vec<Option<Ty>>,
    pub es: Vec<Exp<T>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RelationExp<T> {
    pub rel_type: RelType,
    pub names: Vec<String>,
    pub data: Vec<Vec<Exp<T>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelType {
    Row, Col
}

#[derive(Debug, PartialEq, Clone)]
pub struct RangeExp<T> {
    pub start: Rc<Exp<T>>,
    pub end: Rc<Exp<T>>,
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
pub enum FilterExp<T> {
    RelOp(RelOp, String, Rc<Exp<T>>),
}


/* BEGIN easy From for testing */
impl<T> From<bool> for Exp<T> {
    fn from(b: bool) -> Self {
        match b {
            true => Exp::Name("true".into()),
            false => Exp::Name("false".into()),
        }
    }
}

impl<T> From<i32> for Exp<T> {
    fn from(i: i32) -> Self {
        Exp::Scalar(i.into())
    }
}

impl<T> From<i64> for Exp<T> {
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

impl<T> From<Exp<T>> for Stmt<T> {
    fn from(e: Exp<T>) -> Self {
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
