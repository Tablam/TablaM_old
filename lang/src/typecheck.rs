use std::rc::Rc;
use core::ast::Exp;
use std::fmt::{self, Display};

// #[derive(Debug, PartialEq, Clone)]
// pub enum BuiltinTy {
//     I32, I64, UTF8,
// }
// 
// #[derive(Debug, PartialEq, Clone)]
// pub enum Ty {
//     Tycon(String),
// }

type Name<'a> = &'a str;

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Type,
    Arrow(Vec<Kind>, Rc<Kind>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Tyex<'a> {
    Tycon(Name<'a>),
    Conapp(Rc<Tyex<'a>>, Vec<Tyex<'a>>),
    Funty(Vec<Tyex<'a>>, Rc<Tyex<'a>>),
    Forall(Vec<Name<'a>>, Rc<Tyex<'a>>),
    Tyvar(Name<'a>),
}

fn space_sep<T: Display>(v: Vec<T>) -> String {
    let vs: Vec<String> = v.into_iter().map(|a:T| a.to_string()).collect();
    vs.join(" ")
}

impl<'a> Display for Tyex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Tyex::*;
        let printable: String = match self {
            Tycon(&ref n) => n.to_owned(),
            Tyvar(&ref a) => a.to_owned(),
            Funty(args, result) =>
                format!("({} -> {})", space_sep(args.to_vec()), result.to_string()),
            Conapp(tau, tys) if tys.len() == 0 => format!("({})", tau.to_string()),
            Conapp(tau, tys) =>
                format!("({} {})", tau.to_string(), space_sep(tys.to_vec())),
            Forall(tyvars, tau) =>
                format!("(forall ({}) {})", space_sep(tyvars.to_vec()), tau.to_string()),
        };
        write!(f, "{}", printable)
    }
}

pub const I32TYPE : Tyex = Tyex::Tycon("I32");
pub const I64TYPE : Tyex = Tyex::Tycon("I64");
pub const BOOLTYPE : Tyex = Tyex::Tycon("Bool");
pub const TYVARA : Tyex = Tyex::Tyvar("'a");
pub const LIST : Tyex = Tyex::Tycon("List");
pub fn listtype(ty: Tyex) -> Tyex { Tyex::Conapp(LIST.into(), vec!(ty)) }

// #[derive(Debug, PartialEq, Clone)]
// pub struct TypedExp {
//     pub ty: Ty,
//     pub exp: Exp<TypedExp>,
// }
