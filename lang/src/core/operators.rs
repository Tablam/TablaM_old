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

//select a single column
fn select(col_pos: usize) -> Apply {
    Box::new(move |frame: &Frame| -> Column { frame.data[col_pos].clone() })
}
