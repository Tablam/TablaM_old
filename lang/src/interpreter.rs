use super::ast::*;
use tablam_core::stdlib::math::*;
use tablam_core::types as TT;
use tablam_core::types::BinOp as BP;
use tablam_core::types::CompareOp as CP;

impl Program {
    pub fn new() -> Self {
        Program {}
    }

    //    fn register_function_native(&self,` expr: &FunDef) {
    //
    //    }

    fn register_function(&self, env: &mut Env, expr: FunDef) {
        env.add_fun(expr);
    }

    fn eval_block(&mut self, env: &mut Env, expr: ExprSlice) -> Return {
        let mut last = Expr::Pass;

        for line in expr {
            last = self.eval_expr(env, line)?;
        }
        Ok(last)
    }

    fn _decode_bool(&mut self, env: &mut Env, expr: &BoolExpr) -> ReturnBool {
        match expr {
            BoolExpr::Const(code) => Ok(*code),
            BoolExpr::Cmp(code) => self.eval_cmp(env, code),
        }
    }

    fn decode_value(&mut self, env: &mut Env, expr: &Value) -> ReturnScalar {
        match expr {
            Value::Value(x) => Ok(x.clone()),
            Value::Var(x) => {
                let (_, value) = self.get_var(env, x);
                self.decode_value(env, &value)
            }
            Value::SideEffect(x) => {
                let code = x;
                let x = &self.eval_expr(env, code)?;
                let value = get_value(x).unwrap();
                self.decode_value(env, value)
            }
        }
    }

    fn eval_if(&mut self, env: &mut Env, expr: &BoolExpr, if_ok: &Expr, if_false: &Expr) -> Return {
        if self._decode_bool(env, expr)? {
            self.eval_expr(env, if_ok)
        } else {
            self.eval_expr(env, if_false)
        }
    }

    fn eval_cmp(&mut self, env: &mut Env, expr: &CmOp) -> ReturnBool {
        let lhs = self.decode_value(env, &expr.lhs)?;
        let rhs = self.decode_value(env, &expr.rhs)?;

        let result = match expr.op {
            CP::Eq => lhs == rhs,
            CP::NotEq => lhs != rhs,
            CP::Greater => lhs > rhs,
            CP::GreaterEq => lhs >= rhs,
            CP::Less => lhs < rhs,
            CP::LessEq => lhs <= rhs,
        };
        Ok(result)
    }

    fn eval_bin_op(&mut self, env: &mut Env, expr: &BinOp) -> Return {
        let lhs = self.decode_value(env, &expr.lhs)?;
        let rhs = self.decode_value(env, &expr.rhs)?;

        let result = match expr.op {
            BP::Add => math_add(&lhs, &rhs),
            BP::Minus => math_minus(&lhs, &rhs),
            BP::Mul => math_mul(&lhs, &rhs),
            BP::Div => math_div(&lhs, &rhs),
        };
        Ok(result.into())
    }

    fn eval_while(&mut self, env: &mut Env, test: &BoolExpr, code: ExprSlice) -> Return {
        for line in code {
            if self._decode_bool(env, test)? && line.is_loop_control() {
                if line.is_break() {
                    break;
                } else {
                    continue;
                }
            }
            self.eval_expr(env, line);
        }

        Ok(Expr::Pass)
    }

    fn eval_for_range(
        &mut self,
        env: &mut Env,
        name: &str,
        range: &TT::Range,
        code: ExprSlice,
    ) -> Return {
        for i in (range.start..range.end).step_by(range.step) {
            self.set_var(env, LetKind::Imm, name, (i as isize).into());
            for line in code {
                if line.is_loop_control() {
                    if line.is_break() {
                        break;
                    } else {
                        continue;
                    }
                }

                self.eval_expr(env, line);
            }
        }
        Ok(Expr::Pass)
    }

    pub fn set_var(&self, env: &mut Env, kind: LetKind, name: &str, value: Value) {
        env.add_var(kind, name, value);
    }

    pub fn get_var(&self, env: &mut Env, name: &str) -> (LetKind, Value) {
        match env.find_var(name) {
            Some(x) => x.clone(),
            None => unimplemented!(),
        }
    }

    pub fn eval_call_simple(
        &mut self,
        env: &mut Env,
        _fun: &FunDef,
        params: &FunCall,
        expr: &Expr,
    ) -> Return {
        for (name, param) in &params.params {
            let value = get_value(param).unwrap();
            self.set_var(env, LetKind::Imm, name, value.clone());
        }
        self.eval_expr(env, expr)
    }

    pub fn eval_call(&mut self, parent: &mut Env, expr: &FunCall) -> Return {
        let mut env = Env::child(parent.clone());

        match parent.find_fun(&expr.name) {
            Some(f) => match &f.body {
                Some(code) => self.eval_call_simple(&mut env, &f, expr, code),
                None => unreachable!(),
            },
            None => unimplemented!(),
        }
    }

    pub fn eval_fun(&self, env: &mut Env, expr: &FunDef) -> Return {
        match &expr.body {
            Some(_) => {
                self.register_function(env, expr.clone());
            }
            None => unimplemented!(),
        }
        Ok(Expr::Pass)
    }

    pub fn eval_fail(&mut self, expr: &Failed) -> Return {
        Err(expr.clone())
    }

    pub fn eval_expr(&mut self, env: &mut Env, expr: &Expr) -> Return {
        match expr {
            Expr::Break => unreachable!(),
            Expr::Continue => unreachable!(),
            Expr::Pass => Ok(Expr::Pass),
            Expr::Value(_) => Ok(expr.clone()),
            Expr::Block(code) => self.eval_block(env, code),
            Expr::While(test, code) => self.eval_while(env, test, code),
            Expr::ForI(name, range, code) => self.eval_for_range(env, name, range, code),
            Expr::If(code, if_ok, if_false) => self.eval_if(env, code, if_ok, if_false),
            Expr::BinOp(code) => self.eval_bin_op(env, code),
            Expr::CmpOp(code) => {
                let result = self.eval_cmp(env, code)?;
                Ok(result.into())
            }
            Expr::Let(kind, name, value) => {
                self.set_var(env, kind.clone(), name, value.clone());
                Ok(Expr::Pass)
            }
            Expr::Var(name) => {
                let (_, value) = self.get_var(env, name);
                Ok(Expr::Value(value))
            }
            Expr::Fun(code) => self.eval_fun(env, code),
            Expr::Call(code) => self.eval_call(env, code),
            Expr::Fail(code) => self.eval_fail(code),
        }
    }

    pub fn eval(&mut self, env: &mut Env, expr: ExprSlice) -> Return {
        self.eval_block(env, expr)
    }
}
