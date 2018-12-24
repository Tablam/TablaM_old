use super::ast::*;
use super::interpreter::*;


#[test]
fn eval_const()
{
    let mut program = Program::new();

    let one = 1.into();
    let result = program.eval_expr(one);

    assert_eq!(result, one);
}
