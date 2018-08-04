use std::rc::Rc;
use std::ops::Add;

use super::types::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR

//TODO: Must automate the build of operators, and apply the above rules...
fn _add(a: RVec<i64>, b: RVec<i64>) -> RVec<i64> {
    println!("Dot! {} {}", a.len(), b.len());
    if a.len() == b.len() {
        println!("Dot!");
        Rc::new(a.iter().zip(b.iter()).map(|(x, y)| x+y).collect())
    } else {
        println!("Scalar");
        Rc::new(a.iter().map(|x| x + b[0]).collect())
    }
}

//TODO: Use the Num crate for already implemented
//polymorphic math
impl Add for Column {
    type Output = Column;

    fn add(self, other: Column) -> Column {
        match (self, other) {
            //(Column::None, Column::None) => Column::None,
            (Column::I64(x), Column::I64(y)) => Column::I64(_add(x, y)),
            (x, y) => panic!("Type mismatch for + {:?} <> {:?} ", x, y)
        }
    }
}

// TODO: The operators follow this patterns:
// maps: ColumnExp & ColumnExp = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp = Column (+[1, 2] = 3)

type Apply = Box<Fn(&Frame) -> Column>;

fn idx_for_name(name:&str, frame:&Frame) -> usize {
    frame.names.iter().position(|&r| r == name).unwrap()
}

//select a single column
fn select(col_pos: usize) -> Apply {
    Box::new(move |frame: &Frame| -> Column { frame.data[col_pos].clone() })
}

fn _equal_scalar<T>(left:&[T], right:&T) -> Column
    where
        T:PartialEq
{
    let x:Vec<bool> = left.into_iter()
        .map( |x| x == right)
        .collect();
    Column::from(x)
}

pub fn equal_scalar(left:&Column, right:&Scalar) -> Column {
    match (left, right) {
        (Column::I64(lhs), Scalar::I64(rhs))  => {
            _equal_scalar(lhs.as_slice(), rhs)
        }
        (Column::UTF8(lhs), Scalar::UTF8(rhs)) => {
            _equal_scalar(lhs.as_slice(), rhs)
        }
        (Column::BOOL(lhs), Scalar::BOOL(rhs))  =>{
            _equal_scalar(lhs.as_slice(), rhs)
        }
        (x , y) => panic!("Improper cast of {:?} to {:?}", x, y)
    }
}

fn _equal_both<T>(left:&[T], right:&[T]) -> Column
    where T:PartialEq
{
    let x:Vec<bool> = left.into_iter()
        .zip(right.into_iter())
        .map( |(x, y)| x == y)
        .collect();
    Column::from(x)
}

pub fn equal_both(left:&Column, right:&Column) -> Column {
    match (left, right) {
        (Column::I64(lhs), Column::I64(rhs))  => {
            _equal_both(lhs.as_slice(), rhs.as_slice())
        }
        (Column::UTF8(lhs), Column::UTF8(rhs)) => {
            _equal_both(lhs.as_slice(), rhs.as_slice())
        }
        (Column::BOOL(lhs), Column::BOOL(rhs))  =>{
            _equal_both(lhs.as_slice(), rhs.as_slice())
        }
        (Column::ROW(lhs), Column::ROW(rhs)) =>{
            _equal_both(lhs.as_slice(), rhs.as_slice())
        }
        (x , y) => panic!("Improper cast of {:?} to {:?}", x, y)
    }
}

fn compare_eq(of:&Frame, left:ColumnExp, right:ColumnExp) -> Column {
    match (left, right) {
        //(ColumnExp::Value(lhs), ColumnExp::Value(rhs))  => Column::from_scalar(true),
        (ColumnExp::Pos(lhs), ColumnExp::Pos(rhs))  =>

            Column::from_scalar(true),
        (x , y) => panic!("Improper cast of {:?} to {:?}", x, y)
    }
}

fn compare_dot(what:Compare) -> Column {
    match what {
        Compare::Eq(lhs, rhs)    => Column::from_scalar(true),
        Compare::NotEq(lhs, rhs) => Column::from_scalar(true),
        Compare::Less(lhs, rhs)  => Column::from_scalar(true),
        Compare::Bigger(lhs, rhs)=> Column::from_scalar(true),
    }
}