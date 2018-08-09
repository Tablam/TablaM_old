//use std::ops;
use std::rc::Rc;

extern crate bit_vec;
use self::bit_vec::BitVec;

use super::types::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR must be same length

// TODO: The operators follow this patterns:
// maps: ColumnExp & ColumnExp = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp = Column (+[1, 2] = 3)

pub struct FilterTwo
{
    left: Column,
    right:Column,
    count:usize,
}

pub struct CompareTwo
{
    left: Column,
    right:Column,
    result:BitVec,
}

impl CompareTwo
{
    pub fn new(left:Column, right:Column, size:usize) -> Self {
        CompareTwo{
            left,
            right,
            result:BitVec::from_elem(size, false)
        }
    }

    fn scan<'a, T, F>(&self, left: &'a [T], right: &'a [T], apply: &'a F ) -> impl Iterator<Item = bool> + 'a
        where
            T: PartialEq + PartialOrd,
            F:  Fn(&T, &T) -> bool
    {
        let scan = left.into_iter()
            .zip(right.into_iter())
            .map(move |(x, y)|  {
                let r:bool = apply(x, y);
                r
            });
        scan
    }

    pub fn apply<T, F>(&mut self, left:&[T], right:&[T], apply:F)
        where
            T: PartialEq + PartialOrd,
            F: Fn(&T, &T) -> bool
    {
        let scan = self.scan(left, right, &apply)
            .enumerate();

        for  (pos, ok) in scan {
            self.result.set(pos, ok)
        }
    }

    fn lift_op_cmp<T>(&mut self, op:Operator, x:&[T], y:&[T])
        where T:PartialEq,
              T:PartialOrd
    {
        match op {
            Operator::Eq => {
                return self.apply(x, y, PartialEq::eq)
            }
            Operator::NotEq => {
                return self.apply(x, y, PartialEq::ne)
            }
            Operator::Less => {
                return self.apply(x, y, PartialOrd::lt)
            }
            Operator::LessEq => {
                return self.apply(x, y, PartialOrd::le)
            }
            Operator::Greater => {
                return self.apply(x, y, PartialOrd::gt)
            }
            Operator::GreaterEq => {
                return self.apply(x, y, PartialOrd::ge)
            }
            Operator::Not => {
                return self.apply(x, y, |x, y| !(x == y))
            }
            _ => panic!(" Operator {:?} not boolean", op)
        };
    }

    pub fn decode_both(&mut self, op:Operator) {
        match (self.left.clone(), self.right.clone()) {
            (Column::I64(lhs), Column::I64(rhs))  => {
                self.lift_op_cmp(op, lhs.as_slice(), rhs.as_slice())
            }
            (Column::UTF8(lhs), Column::UTF8(rhs)) => {
                self.lift_op_cmp(op, lhs.as_slice(), rhs.as_slice())
            }
            (Column::BOOL(lhs), Column::BOOL(rhs))  =>{
                self.lift_op_cmp(op, lhs.as_slice(), rhs.as_slice())
            }
            (Column::ROW(lhs), Column::ROW(rhs)) =>{
                self.lift_op_cmp(op, lhs.as_slice(), rhs.as_slice())
            }
            (x , y) => panic!(" Incompatible {:?} and {:?}", x, y)
        }
    }

    pub fn filter_value<T>(&mut self, of:&[T]) -> Vec<T>
    where  T:Clone
    {
        let index = self.result.iter()
            .enumerate()
            .filter_map(|(pos, x)| {
                if x {
                    Some(pos)
                } else {
                    None
                }
            });
        let mut result = Vec::new();

        for pos in index {
            let r = &of[pos];
            result.push(r.clone())
        }
        result
    }

    pub fn materialize<T>(&mut self, of:&[T], index:&[usize]) -> Column
        where  T:Clone + Value + ColumnIter + ColumnType
    {
        let mut result = Vec::new();

        for pos in index {
            let r = &of[*pos];
            result.push(r.clone())
        }

        Column::from(result)
    }

    pub fn filter(&mut self) -> Column {
        match self.left.clone() {
            Column::I64(lhs) => {
                let r:Vec<_> = self.filter_value(lhs.as_slice());
                Column::from(r)
            }
            Column::UTF8(lhs) => {
                let r:Vec<_> = self.filter_value(lhs.as_slice());
                Column::from(r)
            }
            Column::BOOL(lhs) =>{
                let r:Vec<_> = self.filter_value(lhs.as_slice());
                Column::from(r)
            }
            Column::ROW(lhs) =>{
                let r:Vec<_> = self.filter_value(lhs.as_slice());
                Column::from(r)
            }
            x => panic!(" Incompatible {:?} ", x)
        }
    }

    pub fn drain_result(&self) -> BitVec {
        self.result.clone()
    }
}

pub fn drain_vec(of:&BitVec) -> Vec<bool> {
    of.iter().map(|x| x).collect()
}

fn _filter_selector<T>(left:&[T], right:&[bool]) -> Vec<T>
    where T: PartialEq,
          T: Value ,
{
    let mut values:Vec<T> = Vec::new();
    for check in right.into_iter().enumerate() {
        if *check.1 {
            values.push(left[check.0].clone())
        }
    }
    values
}

#[derive(Clone)]
pub struct CompareRel {
    rel:Rc<RelationRow>,
    pub op:Operator,
    pub left:ColumnExp,
    pub right:ColumnExp,
}

impl CompareRel {
    fn eq(rel:Rc<RelationRow>, left:ColumnExp, right:ColumnExp) -> Self {
        CompareRel {
            rel,
            op:Operator::Eq,
            left,
            right,
        }
    }
    fn noteq(rel:Rc<RelationRow>, left:ColumnExp, right:ColumnExp) -> Self {
        CompareRel {
            rel,
            op:Operator::NotEq,
            left,
            right,
        }
    }
}


pub fn select(of:Rc<RelationRow>, pick:&ColumnExp) -> Column {
    match pick {
        ColumnExp::Name(x) => of.col_named(x.as_str()),
        ColumnExp::Pos(x) => of.col(*x),
    }
}

pub fn deselect(of:Rc<RelationRow>, pick:&ColumnExp) -> Vec<Column> {
    match pick {
        ColumnExp::Name(x) =>  {
            let mut names = of.names();

            names.retain(|name| name != x);
            names.iter().map(|x| of.col_named(&x)).collect()
        }
        ColumnExp::Pos(x) =>  {
            let mut names = of.names();
            names.remove(*x);

            names.iter().map(|x| of.col_named(&x)).collect()
        }
    }
}

fn compare(of: CompareRel) -> BitVec {
    let op = of.op;
    let rel = of.rel;

    let col1 = select(rel.clone(), &of.left);
    let col2 = select(rel.clone(), &of.right);
    let mut cmp = CompareTwo::new(col1, col2, rel.row_count());

    cmp.decode_both(op);
    cmp.drain_result()
}


fn filter(of: CompareRel) -> Column {
    let op = of.op;
    let rel = of.rel;

    let col1 = select(rel.clone(), &of.left);
    let col2 = select(rel.clone(), &of.right);
    let mut cmp = CompareTwo::new(col1, col2, rel.row_count());

    cmp.decode_both(op);
    cmp.filter()
}
#[cfg(test)]
mod tests {
    use super::*;

    fn make_nums1() -> Vec<i64> {
        vec![1, 2, 3]
    }

    fn make_nums2() -> Vec<i64> {
        vec![4, 2, 3]
    }

    fn col(pos:usize) -> ColumnExp {
        ColumnExp::Pos(pos)
    }

    fn columns() -> (ColumnExp, ColumnExp) {
        let pick1 = ColumnExp::Name("col0".to_string());
        let pick2 = col(1);

        (pick1, pick2)
    }

    fn make_rel1() -> Rc<Frame> {
        let nums1 = make_nums1();
        let nums2 = make_nums2();

        let col1 = Column::from(nums1);
        let col2 = Column::from(nums2);

        Rc::new(Frame::new(vec!(col1, col2)))
    }

    #[test]
    fn test_select() {
        let nums1 = make_nums1();
        let f1 = make_rel1();

        let (pick1, pick2) = columns();

        let col3 = select(f1.clone(), &pick1);
        let nums3:Vec<i64> = col3.as_slice().into();
        assert_eq!(nums1, nums3);

        let cols = deselect(f1.clone(), &pick1);
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn test_compare() {
        let f1 = make_rel1();
        let (pick1, pick2) = columns();

        let filter_eq = CompareRel::eq(f1.clone(), pick1, pick2);
        let filter_not_eq = CompareRel::noteq(f1.clone(), col(0), col(1));

        let result_eq = drain_vec(&compare(filter_eq));
        println!("= {:?}", result_eq);
        assert_eq!(result_eq, [false, true, true]);

        let result_not_eq =  drain_vec(&compare(filter_not_eq));
        println!("<> {:?}", result_eq);
        assert_eq!(result_not_eq, [true, false, false]);
    }

    #[test]
    fn test_filter() {
        let f1 = make_rel1();
        let (pick1, pick2) = columns();

        let filter_eq = CompareRel::eq(f1.clone(), pick1, pick2);
        let filter_not_eq = CompareRel::noteq(f1.clone(), col(0), col(1));

        let result_eq = filter(filter_eq);
        println!("= {:?}", result_eq);
//        assert_eq!(result_eq, [false, true, true]);

        let result_not_eq =   filter(filter_not_eq);
        println!("<> {:?}", result_eq);
//        assert_eq!(result_not_eq, [true, false, false]);
    }
//    #[test]
//    fn math() {
//
//    }
}
