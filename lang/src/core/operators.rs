#![allow(unused_imports)]
use std::rc::Rc;
use std::ops::{Add};

extern crate bit_vec;
use self::bit_vec::BitVec;

use super::types::*;
use super::relation::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR must be same length

impl Add for Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        match (self, other) {
            (Scalar::I32(x), Scalar::I32(y)) => Scalar::I32(x + y),
            (Scalar::I64(x), Scalar::I64(y)) => Scalar::I64(x + y),
            _ => panic!("Not implemented")
        }
    }
}

// TODO: The operators follow this patterns:
// maps:   ColumnExp & ColumnExp & fn = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp & fn = Column (+ [1, 2] = 3)
//
//struct Cmp<'a, F>
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//{
//    left:   &'a Data,
//    right:  &'a Data,
//    pos  :  usize,
//    apply:  F,
//}
//
//impl <'a, F> Iterator for Cmp<'a, F>
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//{
//    type Item = (usize, bool);
//
//    fn next(&mut self) -> Option<(usize, bool)> {
//        if self.pos < self.left.len {
//            let l = &self.left.data[self.pos];
//            let r = &self.right.data[self.pos];
//            let result = (self.pos, (self.apply)(l, r));
//
//            self.pos +=1;
//            return Some(result)
//        };
//        None
//    }
//}
//
//fn scan_simple<'a, F>(left: &'a Data, right: &'a Data, apply: &'a F ) -> impl Iterator<Item = bool> + 'a
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//{
//    let value = &right.data[0];
//
//    let scan = left.data.iter()
//        .map(move |x|  {
//            let r:bool = apply(x, value);
//            r
//        });
//    scan
//}
//
//fn scan_both<'a, F>(left: &'a Data, right: &'a Data, apply: &'a F ) -> impl Iterator<Item = bool> + 'a
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//{
//    let scan = left.data.iter()
//        .zip(right.data.iter())
//        .map(move |(x, y)|  {
//            let r:bool = apply(x, y);
//            r
//        });
//    scan
//}
//
//fn eq(left:&Data, right:&Data) -> Data
//{
//    let result:Vec<bool> = scan_both(&left, &right, &PartialEq::eq).collect();
//
//    Data::from(result)
//}
//
//fn filter(left:&Data, right:&Data) -> Data
//{
//    let rows = left.clone();
//    let result=
//        if right.len == 1 {
//            let cmp = scan_simple(&left, &right, &PartialEq::eq);
//            rows.data.iter()
//                .zip(cmp)
//                .filter_map(|(x, check)| {
//                    if check {
//                        Some(x.clone())
//                    } else {
//                        None
//                    }
//                })
//                .collect()
//        } else {
//            let cmp = scan_simple(&left, &right, &PartialEq::eq);
//            rows.data.iter()
//                .zip(cmp)
//                .filter_map(|(x, check)| {
//                    if check {
//                        Some(x.clone())
//                    } else {
//                        None
//                    }
//                })
//                .collect()
//        };
//
//    Data::new(result, rows.kind)
//}
//
//fn filter_pos(left:&Data, right:&Data) -> Vec<usize>
//{
//    let result:Vec<usize>=
//        if right.len == 1 {
//            let cmp = scan_simple(&left, &right, &PartialEq::eq);
//            cmp.enumerate()
//                .filter_map(|(x, check)| {
//                    if check {
//                        Some(x)
//                    } else {
//                        None
//                    }
//                })
//                .collect()
//        } else {
//            let cmp = scan_simple(&left, &right, &PartialEq::eq);
//            cmp.enumerate()
//                .filter_map(|(x, check)| {
//                    if check {
//                        Some(x)
//                    } else {
//                        None
//                    }
//                })
//                .collect()
//        };
//
//    result
//}
//

//fn scan_simple<'a, F>(left: &'a Data, right: &'a Data, apply: &'a F ) -> impl Iterator<Item = bool> + 'a
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//fn lift_cmp<F>(of:Operator) -> F
//    where
//        F:  Fn(&Scalar, &Scalar) -> bool
//{
//    match cmp.op {
//        Operator::Eq        =>PartialEq::eq,
////        Operator::NotEq,
////        Operator::Less,
////        Operator::LessEq,
////        Operator::Greater,
////        Operator::GreaterEq,
////        Operator::Not,
//    }
//
//}
//fn lift_cmp(op:Operator) -> Box<BoolExpr>
//{
//    match op {
//        Operator::Eq        => Box::new(move |left:&Scalar, right:&Scalar| left == right),
//        Operator::NotEq     => Box::new(move |left:&Scalar, right:&Scalar| left != right),
//        Operator::Less      => Box::new(move |left:&Scalar, right:&Scalar| left < right),
//        Operator::LessEq    => Box::new(move |left:&Scalar, right:&Scalar| left <= right),
//        Operator::Greater   => Box::new(move |left:&Scalar, right:&Scalar| left > right),
//        Operator::GreaterEq => Box::new(move |left:&Scalar, right:&Scalar| left >= right),
//        _ => Box::new(move |left:&Scalar, right:&Scalar| false),
//    }
//}

fn _select<T>(of:&DataSource<T>, pick:Schema) -> Frame
    where T:Relation
{
    let mut columns = Vec::with_capacity(pick.len());

    for col in &pick.columns {
        columns.push(of.source.get_col(&col.name));
    }

    Frame::new(pick.clone(), columns)
}

/// Select: aka projection in relational algebra
fn select<T>(of:&DataSource<T>, pick:Schema) -> Frame
    where T:Relation
{
    let cols = pick.as_slice();
    let selected = of.source.names().only(cols);
    _select(of, selected)
}

fn deselect<T>(of:&DataSource<T>, remove:Schema) -> Frame
    where T:Relation
{
    let cols = remove.as_slice();
    let selected = of.source.names().except(cols);
    _select(of, selected)
}

/// Filters: aka where in sql
fn filter<T>(left:&DataSource<T>, col:ColumnExp, apply:&BoolExpr, value:&Scalar) -> Vec<usize>
    where T:Relation
{
    let col = left.source.resolve_pos(&col);
    left.find_collect(col, value, apply)
}

fn compare<T>(left:&DataSource<T>, col:ColumnExp, apply:&BoolExpr, value:&Scalar) -> Vec<bool>
    where T:Relation
{
    let col = left.source.resolve_pos(&col);
    left.cmp_collect(col, value, apply)
}

/// Joins
fn join_left<T, U>(of:&Both<T, U>, left:ColumnExp, right:ColumnExp, apply:&BoolExpr)
    where
        T: Relation,
        U: Relation
{
    let col_l = of.left.source.resolve_pos(&left);
    let col_r = of.right.source.resolve_pos(&left);
    let results = of.join(col_l, col_r, apply);

    println!("{:?}", results)
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

    fn make_nums3() -> Vec<i64> {
        vec![1, 2, 2]
    }

    fn columns() -> (ColumnExp, ColumnExp) {
        let pick1 = coln("0");
        let pick2 = col(1);

        (pick1, pick2)
    }

    fn make_rel(of:Vec<Vec<i64>>) -> DataSource<Frame>
    {
        let cols = of.into_iter().map(|x| Data::from(x)).collect();
        DataSource::new(Frame::new_anon(cols))
    }

    fn make_rel1() -> DataSource<Frame> {
        let nums1 = make_nums1();
        let nums2 = make_nums2();
        make_rel(vec![nums1, nums2])
    }

    fn make_rel2() -> DataSource<Frame> {
        let nums1 = make_nums3();
        let nums2 = make_nums1();
        make_rel(vec![nums1, nums2])
    }

    #[test]
    fn test_cursor() {
        let fields = Schema::new([].to_vec());
        let cursor =  DataSource::new(Frame::empty(fields));
        assert_eq!(cursor.eof(), true);
        assert_eq!(cursor.next(), false);

        let cursor = make_rel1();
        assert_eq!(cursor.eof(), false);
        assert_eq!(cursor.next(), true);
        assert_eq!(cursor.next(), true);
        assert_eq!(cursor.pos(), 2);

        assert_eq!(cursor.back(), true);
        assert_eq!(cursor.back(), true);
        assert_eq!(cursor.pos(), 0);

        cursor.first();
        assert_eq!(cursor.eof(), false);
        assert_eq!(cursor.pos(), 0);

        assert_eq!(cursor.skip(2), true);
        assert_eq!(cursor.pos(), 2);
        assert_eq!(cursor.eof(), false);

        assert_eq!(cursor.skip(-2), true);
        assert_eq!(cursor.pos(), 0);
        let col1 = Data::from(make_nums1());

        let mut data = Vec::new();
        while !cursor.eof() {
            data.push(cursor.value(0).clone());
            cursor.next();
        }
        let col = Data::new(data, DataType::I64);
        assert_eq!(col, cursor.source.col(0));
    }

    #[test]
    fn test_compare() {
        let f1 = make_rel1();
        let (pick1, pick2) = columns();
        let value = f1.value(0).clone();

        let query1 = filter(&f1, pick1.clone(), &PartialEq::eq, &value);
        let r1 = Data::from(1i64);

        println!("Where = {:?}", value);
        println!("Q1 {:?}", query1);
        assert_eq!(query1.len(), 1);
        //assert_eq!(query1, r1);

        f1.first();
        println!("Cmp = {:?} {}", value, f1.len());
        let query2 = compare(&f1, pick1.clone(), &PartialEq::eq, &value);
        println!("Q2 {:?}", query2);
        assert_eq!(query2.len(), 3);
        assert_eq!(query2, [true, false, false]);
    }

    #[test]
    fn test_select() {
        let f1 = make_rel1();

        let (pick1, pick2) = (col(1),  col(1));;
        //println!("{:?}", f1.names());
        let names1 = f1.source.resolve_names(vec![&pick1]);
        let names2 = f1.source.resolve_names(vec![&pick2]);

        let query1 = select(&f1, names1);
        println!("Select {:?}", query1);
        assert_eq!(query1.names.len(), 1);

        let query2 = deselect(&f1, names2);
        println!("Deselect {:?}", query2);
        assert_eq!(query2.names.len(), 1);
    }

    #[test]
    fn test_join() {
        let f1 = make_rel1();
        let f2 = make_rel2();

        let both = Both{
            left:f1,
            right:f2,
        };

        join_left(&both, col(0), col(0), &PartialEq::eq);
    }
}
