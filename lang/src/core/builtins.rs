/// Implement the global available functions/operators
use std::ops::{Add, BitAnd, BitOr, BitXor, Deref, Div, Mul, Neg, Rem, Shl, Shr, Sub};

pub use core::types::*;

fn add<T: Add>(x: T, y: T) -> <T as Add>::Output { x + y }
fn sub<T: Sub>(x: T, y: T) -> <T as Sub>::Output { x - y }
fn mul<T: Mul>(x: T, y: T) -> <T as Mul>::Output { x * y }
fn div<T: Div>(x: T, y: T) -> <T as Div>::Output { x / y }

pub fn bin_op(op:&BinExp, x:&Scalar, y:&Scalar) -> &Scalar {

}