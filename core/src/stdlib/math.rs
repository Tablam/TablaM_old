use std::ops::*;

extern crate rust_decimal;
use rust_decimal::Decimal;

use crate::scalar::bin_op;
use crate::types::*;

pub fn math_add(x: &Scalar, y: &Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>(Add::add, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>(Add::add, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>(Add::add, *a, *b),
        (Scalar::Decimal(a), Scalar::Decimal(b)) => bin_op::<Decimal, _>(Add::add, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b),
    }
}

pub fn math_minus(x: &Scalar, y: &Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>(Sub::sub, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>(Sub::sub, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>(Sub::sub, *a, *b),
        (Scalar::Decimal(a), Scalar::Decimal(b)) => bin_op::<Decimal, _>(Sub::sub, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b),
    }
}

pub fn math_mul(x: &Scalar, y: &Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>(Mul::mul, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>(Mul::mul, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>(Mul::mul, *a, *b),
        (Scalar::Decimal(a), Scalar::Decimal(b)) => bin_op::<Decimal, _>(Mul::mul, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b),
    }
}

pub fn math_div(x: &Scalar, y: &Scalar) -> Scalar {
    match (x, y) {
        (Scalar::ISize(a), Scalar::ISize(b)) => bin_op::<isize, _>(Div::div, *a, *b),
        (Scalar::I32(a), Scalar::I32(b)) => bin_op::<i32, _>(Div::div, *a, *b),
        (Scalar::I64(a), Scalar::I64(b)) => bin_op::<i64, _>(Div::div, *a, *b),
        (Scalar::Decimal(a), Scalar::Decimal(b)) => bin_op::<Decimal, _>(Div::div, *a, *b),
        (a, b) => panic!("Argument {:?} <> {:?}", a, b),
    }
}
