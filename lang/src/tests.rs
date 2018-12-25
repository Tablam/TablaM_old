use super::ast::*;

fn _eval_expr(input:&Expr, output:&Expr) {
    let program = Program::new();
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

    _eval_expr(&if1, &two.clone())
}

#[test]
fn eval_while()
{
    let one:Expr = 1.into();
    let two:Expr = 2.into();
    let check = less(var("x"), two.clone().into());

    let v1 = set_var_mut("x", one.into());
    let body = lines(vec![set_var_mut("x", two.into())]);

    let loop1= ewhile_cmp(check, body);
    let full = block_lines(vec![v1, loop1]);

    _eval_expr(&full, &Expr::Pass)
}

#[test]
fn eval_for()
{
    let one:Expr = 1.into();
    let ten:Expr = 10isize.into();
    let body = lines(vec![pass()]);

    let loop1= efor("x", 0, 11, body);
    let full = block_lines(vec![loop1, evar("x")]);

    _eval_expr(&full, &ten)
}