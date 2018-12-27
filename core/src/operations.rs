#![allow(dead_code)]
use std::ops::*;

use super::ndarray::*;
use super::types::*;
use super::dsl::*;

fn bin_op<T, Op>(op: Op, x:T, y:T) -> Scalar
    where
        Op: FnOnce(T, T) -> T,
        T: From<Scalar>, Scalar: From<T>,
{
    op(x, y).into()
}

fn bin_op_by<T, Op>(op: Op, x:Scalar, y:Scalar) -> Scalar
    where
        Op: FnOnce(T, T) -> T,
        T: From<Scalar>, Scalar: From<T>,
{
    bin_op(op, x.into(), y.into())
}

//macro_rules! bin_op {
//    ($kind:ident, $op:ident, $bound:path) => (
//        fn $kind(x:Scalar, y:Scalar) -> Scalar {
//            match (x, y) {
//                ($path(a), $path(b)) => bin_op::<i32, _>( Add::add, a, b),
//                (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>( Add::add, a, b),
//                (a, b) => panic!("Argument {:?} <> {:?}", a, b )
//            }
//        }
//    )
//}

pub fn math_add(x:&Scalar, y:&Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>( Add::add, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>( Add::add, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>( Add::add, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b )
    }
}

pub fn math_minus(x:&Scalar, y:&Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>( Sub::sub, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>( Sub::sub, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>( Sub::sub, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b )
    }
}

pub fn math_mul(x:&Scalar, y:&Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>( Mul::mul, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>( Mul::mul, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>( Mul::mul, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b )
    }
}

pub fn math_div(x:&Scalar, y:&Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>( Div::div, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>( Div::div, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>( Div::div, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b )
    }
}

pub fn zip_scalar(x:&Data, y:&Data, op:&BinExpr) -> Data {
    let a = x.data.col(0);
    let b = y.data.col(0);

    let result:Col = a.into_iter().zip(b.into_iter())
        .map(|(lhs, rhs)| op(lhs, rhs)).collect();

    let kind = x.schema.columns[0].kind;
    array_t(kind, &result)
}