use std::cell::RefCell;
use std::rc::Rc;

use super::ast::*;
//use tablam_core::types as TT;
use tablam_core::types::CompareOp as CP;

#[derive(Debug, Clone)]
pub struct Program {
    env: Rc<RefCell<Env>>
}

impl Program {
    pub fn new() -> Self {
        Program { env: Rc::new(RefCell::new(Env::empty())) }
    }

    fn eval_block(&mut self, expr: &ExprList) -> Expr {
        let mut last = Expr::Pass.into();

        for line in expr {
            last = self.eval_expr(line);
        }
        last
    }

    fn eval_if(&mut self, expr: &BoolExpr, if_ok:&Expr, if_false:&Expr) -> Expr {

        let result =
            match expr {
                BoolExpr::Const(code)=> *code,
                BoolExpr::Cmp(code) => self.eval_cmp(code).into(),
            };

        if result {
            self.eval_expr(if_ok)
        } else {
            self.eval_expr(if_false)
        }

    }

    fn eval_cmp(&mut self, expr: &CmOp) -> bool {
        match expr.op {
            CP::Eq          => expr.lhs == expr.rhs,
            CP::NotEq       => expr.lhs != expr.rhs,
            CP::Greater     => expr.lhs >  expr.rhs,
            CP::GreaterEq   => expr.lhs >= expr.rhs,
            CP::Less        => expr.lhs <  expr.rhs,
            CP::LessEq      => expr.lhs <= expr.rhs,
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Pass => Expr::Pass.into(),
            Expr::Value(_)      => expr.clone(),
            Expr::Block(code)   => self.eval_block(code),
            Expr::If(code, if_ok, if_false) => self.eval_if(code, if_ok, if_false),
            Expr::CmpOp(code)   => self.eval_cmp(code).into(),
        }
    }

    pub fn eval(&mut self, expr: ExprList) -> Expr {
        self.eval_block(&expr)
    }
}