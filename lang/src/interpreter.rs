use std::cell::RefCell;
use std::rc::Rc;

use super::ast::*;
use tablam_core::types as TT;
use tablam_core::types::CompareOp as CP;

#[derive(Debug, Clone)]
pub struct Program {
    env: Rc<RefCell<Env>>
}

impl Program {
    pub fn new() -> Self {
        Program { env: Rc::new(RefCell::new(Env::empty())) }
    }

    fn eval_block(&mut self, expr: ExprList) -> Expr {
        let mut last = Expr::Pass;

        for line in expr {
            last = self.eval_expr(line);
        }
        last
    }

    fn eval_if(&mut self, expr: BoolExpr) -> Expr {
        match expr {
            BoolExpr::Const(code)=> Expr::Pass,
            BoolExpr::Cmp(code) => self.eval_cmp(code),
        }
    }

    fn eval_cmp(&mut self, expr: CmOp) -> Expr {
        match expr.op {
            CP::Not=> Expr::Pass,
            CP::Eq=> Expr::Pass,
            CP::NotEq=> Expr::Pass,
            CP::Greater=> Expr::Pass,
            CP::GreaterEq=> Expr::Pass,
            CP::Less=> Expr::Pass,
            CP::LessEq=> Expr::Pass,
        }
    }

    pub fn eval_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Pass          => Expr::Pass,
            Expr::Value(code)   => Expr::Value(code),
            Expr::Block(code)   => self.eval_block(code),
            Expr::If(code, if_ok, if_false) => self.eval_if(code),
            Expr::CmpOp(code)   => self.eval_cmp(code),
        }
    }

    pub fn eval(&mut self, expr: ExprList) -> Expr {
        self.eval_block(expr)
    }
}