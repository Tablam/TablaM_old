use std::ops;

use super::types::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR must be same length

// TODO: The operators follow this patterns:
// maps: ColumnExp & ColumnExp = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp = Column (+[1, 2] = 3)

/// Comparing 2 columns
#[inline]
fn _compare_both<'r, 's, T, F>(left:&'r [T], right:&'s [T], mut apply:F) -> Column
    where T: PartialEq,
          F: FnMut(&'r T, &'s T) -> bool
{
    let x:Vec<bool> = left.into_iter()
        .zip(right.into_iter())
        .map( |(x, y)| apply(x, y))
        .collect();
    Column::from(x)
}

/// Comparing a column with a scalar
fn _compare_col_scalar<T, F>(left:&[T], right:&T, mut apply:F) -> Column
    where T:PartialEq,
          F: FnMut(&T, &T) -> bool
{
    let x:Vec<bool> = left.into_iter()
        .map( | x | apply(x, right))
        .collect();
    Column::from(x)
}

/// Comparing 2 scalars
fn _compare_scalar_scalar<T, F>(left:&T, right:&T, mut apply:F) -> Column
    where T:PartialEq,
          F: FnMut(&T, &T) -> bool
{
    let x:Vec<bool> = vec!(apply(left, right));
    Column::from(x)
}

pub fn decode_both(left:&Column, right:&Column, op:Operator) -> Column {
    match (left, right) {
        (Column::I64(lhs), Column::I64(rhs))  => {
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::UTF8(lhs), Column::UTF8(rhs)) => {
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::BOOL(lhs), Column::BOOL(rhs))  =>{
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::ROW(lhs), Column::ROW(rhs)) =>{
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (x , y) => panic!(" Incompatible {:?} and {:?}", x, y)
    }
}

pub fn equal_both(left:&Column, right:&Column) -> Column {
    decode_both(left, right, Operator::Eq)
//    match (left, right) {
//        (Column::I64(lhs), Column::I64(rhs))  => {
//            _compare_both(lhs.as_slice(), rhs.as_slice(), PartialEq::eq)
//        }
//        (Column::UTF8(lhs), Column::UTF8(rhs)) => {
//            _compare_both(lhs.as_slice(), rhs.as_slice(), PartialEq::eq)
//        }
//        (Column::BOOL(lhs), Column::BOOL(rhs))  =>{
//            _compare_both(lhs.as_slice(), rhs.as_slice(), PartialEq::eq)
//        }
//        (Column::ROW(lhs), Column::ROW(rhs)) =>{
//            _compare_both(lhs.as_slice(), rhs.as_slice(), PartialEq::eq)
//        }
//        (x , y) => panic!(" Incompatible {:?} and {:?}", x, y)
//    }
}

pub fn not_equal_both(left:&Column, right:&Column) -> Column {
    decode_both(left, right, Operator::NotEq)
}

pub fn less_both(left:&Column, right:&Column) -> Column
{
    decode_both(left, right, Operator::Less)
}

// Select column
fn select_name(name:&str, of:&RelationRow) -> Column {
    of.col_named(name)
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn math() {
//
//    }
//
//    #[test]
//    fn compare() {
//
//    }
//
//}
