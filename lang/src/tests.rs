use tablam_core::types as TT;
use super::ast::*;
use super::interpreter::*;


fn _eval_expr(input:&Expr, output:&Expr) {
    let mut program = Program::new();
    let result = &program.eval_expr(input);

    assert_eq!(result, output);
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
    let one:TT::Scalar = 1.into();
    let two:TT::Scalar = 2.into();

    let cmp1 = eq( one.clone().into(), two.clone().into());
    let cmp2 = not_eq( one.into(), two.into());

    _eval_expr(&cmp1, &false.into());
    _eval_expr(&cmp2, &true.into());
}

#[test]
fn eval_if()
{
    let one:Expr = 1.into();
    let two:Expr = 2.into();

    let if1 = eif(false, one.clone().into(), two.clone().into());

    _eval_expr(&if1, &two.clone())
}
