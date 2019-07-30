use tablam_core::types::DataType as DT;
use tablam_core::types::CompareOp as CP;
use super::ast::*;

fn _eval_expr(input:&Expr, output:&Expr) {
    let mut program = Program::new();
    let mut env =Env::empty();

    match &program.eval_expr(&mut env, input) {
        Ok(result) => assert_eq!(result, output),
        Err(msg) => panic!("{:?}", msg),
    }
}

fn _eval_exprs(of:&[(Expr, Expr)]) {
    for (input, output) in of {
        _eval_expr(input, output)
    }
}

#[test]
fn eval_const()
{
    let one:Expr = 1.into();
    _eval_expr(&one, &one.clone())
}

#[test]
fn eval_cmp()
{
    let one:Value = 1.into();
    let two:Value = 2.into();

    let cmp1 = eq( one.clone().into(), two.clone().into()).into();
    let cmp2 = not_eq( one.into(), two.into()).into();

    _eval_expr(&cmp1, &false.into());
    _eval_expr(&cmp2, &true.into());
}

#[test]
fn eval_if()
{
    let one:Expr = 1.into();
    let two:Expr = 2.into();

    let if1 = eif(false, one.clone().into(), two.clone().into());
    _eval_expr(&if1, &two.clone());

    let if2 = eif_cmp(cmp(CP::Less, 1.into(), 2.into()), two.clone().into(), one.clone().into());
    _eval_expr(&if2, &two.clone());

}

#[test]
fn eval_bin_op()
{
    let one:Value = 1i64.into();
    let two:Value = 2i64.into();

    _eval_exprs(&[
        (plus_op(one.clone(), two.clone()), 3i64.into()),
        (minus_op(one.clone(), two.clone()), (-1i64).into()),
        (mul_op(one.clone(), two.clone()), 2i64.into()),
        (div_op(one.clone(), two.clone()), 0i64.into()),
    ])
}

#[test]
fn eval_while()
{
    let one:Expr = 1.into();
    let two:Expr = 2.into();
    let check = less(var("x"), two.clone().into());

    let v1 = set_var_mut("x", one.into());
    let body = lines(vec![set_var_mut("x", two.clone().into())]);
    let body_break = lines(vec![set_var_imm("x", two.into()), Expr::Break]);

    let loop1= ewhile_cmp(check, body.clone());
    let loop2= ewhile(true, body_break);

    let full = block_lines(vec![v1, loop1]);

    _eval_expr(&full, &Expr::Pass);
    _eval_expr(&loop2, &Expr::Pass);
}

#[test]
fn eval_for()
{
    let ten:Expr = 10isize.into();
    let body = lines(vec![pass()]);

    let loop1= efor("x", 0, 11, body.clone());
    let loop2= efor_step("x", 0, 11, 2, body);

    let full = block_lines(vec![loop1, evar("x")]);
    _eval_expr(&full, &ten);

    let full = block_lines(vec![loop2, evar("x")]);
    _eval_expr(&full, &ten);
}


#[test]
fn eval_fun()
{
    let one:Expr = 1.into();
    let two:Expr = 2.into();

    let body = plus_op(var("x"), var("y"));
    let params = &[("x", DT::I64), ("y", DT::I64)];
    let fun1 = fun_def("sum", params, DT::I64, body.into());
    let call1 = fun_call("sum", &[("x", one.into()), ("y", two.into())]);
    let full = block_lines(vec![fun1, call1]);

    _eval_expr(&full, &3.into())
}