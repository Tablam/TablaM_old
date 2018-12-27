use std::rc::Rc;

use super::ast::*;
use tablam_core::types as TT;
use tablam_core::types::CompareOp as CP;
use tablam_core::types::BinOp as BP;
use tablam_core::operations::*;

impl Program {
    pub fn new() -> Self {
        Program {

        }
    }

//    fn register_function_native(&self,` expr: &FunDef) {
//
//    }

    fn register_function(&self, env:&mut Env, expr: FunDef) {
        env.add_fun(expr);
    }

    fn eval_block(&mut self, env:&mut Env, expr: &ExprList) -> Expr {
        let mut last = Expr::Pass.into();

        for line in expr {
            last = self.eval_expr(env, line);
        }
        last
    }

    fn _decode_bool(&mut self, env:&mut Env, expr: &BoolExpr) -> bool {
        match expr {
            BoolExpr::Const(code)=> *code,
            BoolExpr::Cmp(code) => self.eval_cmp(env, code).into(),
        }
    }

    fn decode_value(&mut self, env:&mut Env, expr: &Value) -> RScalar {
        match expr {
            Value::Value(x)=> x.clone(),
            Value::Var(x) => {
                let (_, value) = self.get_var(env,x);
                self.decode_value(env,&value)
            },
            Value::SideEffect(x) =>{
                let code = x;
                let x = &self.eval_expr(env,code);
                let value = get_value(x).unwrap();
                self.decode_value( env,value)
            }
        }
    }

    fn eval_if(&mut self, env:&mut Env, expr: &BoolExpr, if_ok:&Expr, if_false:&Expr) -> Expr {
        if self._decode_bool(env, expr) {
            self.eval_expr(env,if_ok)
        } else {
            self.eval_expr(env,if_false)
        }
    }

    fn eval_cmp(&mut self, env:&mut Env, expr: &CmOp) -> bool {
        let lhs = self.decode_value(env, &expr.lhs);
        let rhs = self.decode_value(env, &expr.rhs);

        match expr.op {
            CP::Eq          => lhs == rhs,
            CP::NotEq       => lhs != rhs,
            CP::Greater     => lhs >  rhs,
            CP::GreaterEq   => lhs >= rhs,
            CP::Less        => lhs <  rhs,
            CP::LessEq      => lhs <= rhs,
        }
    }

    fn eval_bin_op(&mut self, env:&mut Env, expr: &BinOp) -> Expr {
        let lhs = self.decode_value(env, &expr.lhs);
        let rhs = self.decode_value(env, &expr.rhs);

        let result =
            match expr.op {
                BP::Add     => math_add(&lhs , &rhs),
                BP::Minus   => math_minus(&lhs , &rhs),
                BP::Mul     => math_mul(&lhs , &rhs),
                BP::Div     => math_div(&lhs , &rhs),
            };
        result.into()
    }

    fn eval_while(&mut self, env:&mut Env, test: &BoolExpr, code:&ExprList) -> Expr {
        while self._decode_bool(env,test) {
            for line in code {
                if line.is_loop_control() {
                    if line.is_break() {
                        break
                    } else {
                        continue
                    }
                }

                self.eval_expr(env, line);
            }
        }

        Expr::Pass
    }

    fn eval_for_range(&mut self, env:&mut Env, name:&String, range:&TT::Range, code:&ExprList) -> Expr {
        for i in (range.start..range.end).step_by(range.step) {
            self.set_var(env, LetKind::Imm, &name, i.into());
            for line in code {
                if line.is_loop_control() {
                    if line.is_break() {
                        break
                    } else {
                        continue
                    }
                }

                self.eval_expr(env,line);
            }
        }
        Expr::Pass
    }

    pub fn set_var(&self, env:&mut Env, kind:LetKind, name:&String, value:Value) {
        env.add_var(kind, name.clone(), value);
    }

    pub fn get_var(&self, env:&mut Env, name:&String) -> (LetKind, Value) {
        match env.find_var(name) {
            Some(x) => x.clone(),
            None => unimplemented!(),
        }
    }

    pub fn eval_call_simple(&mut self, env:&mut Env, _fun:&FunDef, params:&FunCall, expr:&Expr) -> Expr {
        for (name, param) in &params.params {
            let value = get_value(param).unwrap();
            self.set_var(env, LetKind::Imm, name, value.clone());
        }
        self.eval_expr(env, expr)
    }

    pub fn eval_call(&mut self, parent:&mut Env, expr:&FunCall) -> Expr {
        let mut env = Env::child(parent.clone());

        let result =
            match parent.find_fun(&expr.name) {
                Some(f) => {
                    match &f.body {
                        Some(code) => {
                            self.eval_call_simple(&mut env, &f, expr, code)
                        },
                        None => unreachable!()
                    }
                },
                None => unimplemented!(),
        };
        result
    }

    pub fn eval_fun(&self, env:&mut Env, expr:&FunDef) -> Expr {
        match &expr.body {
            Some(_) => {
                self.register_function(env, expr.clone());
            },
            None => unimplemented!()
        }
        Expr::Pass
    }

    pub fn eval_expr(&mut self, env:&mut Env, expr: &Expr) -> Expr {
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
                self.eval_block(env,code),
            Expr::While(test, code) =>
                self.eval_while(env, test, code),
            Expr::ForI(name, range, code) =>
                self.eval_for_range(env, name, range, code),
            Expr::If(code, if_ok, if_false) =>
                self.eval_if(env,code, if_ok, if_false),
            Expr::BinOp(code) =>
                self.eval_bin_op(env,code),
            Expr::CmpOp(code) =>
                self.eval_cmp(env,code).into(),
            Expr::Let(kind, name, value)  => {
                self.set_var(env, kind.clone(), name, value.clone());
                Expr::Pass
            }
            Expr::Var(name) => {
                let (_, value) = self.get_var(env, name);
                Expr::Value(value)
            },
            Expr::Fun(code) =>
                self.eval_fun(env, code),
            Expr::Call(code) =>
                self.eval_call(env, code),
        }
    }

    pub fn eval(&mut self, env:&mut Env, expr: ExprList) -> Expr {
        self.eval_block(env, &expr)
    }
}