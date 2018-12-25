use std::cell::RefCell;
use std::rc::Rc;

use super::ast::*;
use tablam_core::types as TT;
use tablam_core::types::CompareOp as CP;
use tablam_core::types::BinOp as BP;
use tablam_core::operations::*;

impl Program {
    pub fn new() -> Self {
        Program { env: Rc::new(RefCell::new(Env::empty())) }
    }

    fn eval_block(&self, expr: &ExprList) -> Expr {
        let mut last = Expr::Pass.into();

        for line in expr {
            last = self.eval_expr(line);
        }
        last
    }

    fn _decode_bool(&self, expr: &BoolExpr) -> bool {
        match expr {
            BoolExpr::Const(code)=> *code,
            BoolExpr::Cmp(code) => self.eval_cmp(code).into(),
        }
    }


    fn decode_value(&self, expr: &Value) -> RScalar {
        match expr {
            Value::Value(x)=> x.clone(),
            Value::Var(x) => {
                let (_, value) = self.get_var(x);
                self.decode_value(&value)
            },
            Value::SideEffect(x) =>{
                let code = x;
                let x = &self.eval_expr(code);
                let value = get_value(x).unwrap();
                self.decode_value( value)
            }
        }
    }

    fn eval_if(&self, expr: &BoolExpr, if_ok:&Expr, if_false:&Expr) -> Expr {
        if self._decode_bool(expr) {
            self.eval_expr(if_ok)
        } else {
            self.eval_expr(if_false)
        }

    }

    fn eval_cmp(&self, expr: &CmOp) -> bool {
        let lhs = self.decode_value(&expr.lhs);
        let rhs = self.decode_value(&expr.rhs);

        match expr.op {
            CP::Eq          => lhs == rhs,
            CP::NotEq       => lhs != rhs,
            CP::Greater     => lhs >  rhs,
            CP::GreaterEq   => lhs >= rhs,
            CP::Less        => lhs <  rhs,
            CP::LessEq      => lhs <= rhs,
        }
    }

    fn eval_bin_op(&self, expr: &BinOp) -> Expr {
        let lhs = self.decode_value(&expr.lhs);
        let rhs = self.decode_value(&expr.rhs);

        let result =
            match expr.op {
                BP::Add     => math_add(&lhs , &rhs),
                BP::Minus   => math_minus(&lhs , &rhs),
                BP::Mul     => math_mul(&lhs , &rhs),
                BP::Div     => math_div(&lhs , &rhs),
            };
        result.into()
    }

    fn eval_while(&self, test: &BoolExpr, code:&ExprList) -> Expr {
        while self._decode_bool(test) {
            for line in code {
                if line.is_loop_control() {
                    if line.is_break() {
                        break
                    } else {
                        continue
                    }
                }

                self.eval_expr(line);
            }
        }

        Expr::Pass
    }

    fn eval_for_range(&self, name:&String, range:&TT::Range, code:&ExprList) -> Expr {
        for i in (range.start..range.end).step_by(range.step) {
            self.set_var(&LetKind::Imm, &name, i.into());
            for line in code {
                if line.is_loop_control() {
                    if line.is_break() {
                        break
                    } else {
                        continue
                    }
                }

                self.eval_expr(line);
            }
        }
        Expr::Pass
    }

    pub fn set_var(&self, kind:&LetKind, name:&String, value:Value) -> Expr {
        self.env.borrow_mut().add(kind.clone(), name.clone(), value);
        Expr::Pass
    }

    pub fn get_var(&self, name:&String) -> (LetKind, Value) {
        match self.env.borrow().find(name) {
            Some(x) => x.clone(),
            None => unimplemented!()
        }
    }

    pub fn eval_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Break =>
                unreachable!(),
            Expr::Continue =>
                unreachable!(),
            Expr::Pass =>
                Expr::Pass,
            Expr::Value(_) =>
                expr.clone(),
            Expr::Block(code) =>
                self.eval_block(code),
            Expr::While(test, code) =>
                self.eval_while(test, code),
            Expr::ForI(name, range, code) =>
                self.eval_for_range(name, range, code),
            Expr::If(code, if_ok, if_false) =>
                self.eval_if(code, if_ok, if_false),
            Expr::BinOp(code) =>
                self.eval_bin_op(code),
            Expr::CmpOp(code) =>
                self.eval_cmp(code).into(),
            Expr::Let(kind, name, value)  =>
                self.set_var(kind, name, value.clone()),
            Expr::Var(name) => {
                let (_, value) = self.get_var(name);
                Expr::Value(value)
            },
        }
    }

    pub fn eval(&self, expr: ExprList) -> Expr {
        self.eval_block(&expr)
    }
}