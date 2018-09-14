#![allow(unused_imports)]
use std::rc::Rc;
use std::ops::{Add};
use std::collections::HashMap;

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
        use self::Scalar::*;
        match (self, other) {
            (I32(x), I32(y)) => I32(x + y),
            (I64(x), I64(y)) => I64(x + y),
            // (UTF8(s), UTF8(o)) => UTF8(s+o),
            p => panic!("Not implemented: cannot add {:?}", p),
        }
    }
}

// TODO: The operators follow this patterns:
// maps:   ColumnExp & ColumnExp & fn = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp & fn = Column (+ [1, 2] = 3)
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
fn joiner<'a, 'b, T: 'a, U: 'b>(left:&'a DataSource<T>, right:&'b DataSource<U>, cols_left:&Vec<ColumnExp>,  cols_right:&Vec<ColumnExp>) -> Both<'a, 'b, T, U>
    where
        T: Relation,
        U: Relation
{
    let cols_left = left.source.resolve_pos_many(cols_left);
    let cols_right = right.source.resolve_pos_many(cols_right);

    Both {
        left,
        right,
        cols_left,
        cols_right
    }
}

fn join_schema<T, U>(of:&Both<T, U>) -> Schema
    where
        T: Relation,
        U: Relation
{
    of.left.source.names().extend( of.right.source.names())
}

fn join_both<T, U>(of:&Both<T, U>, apply:&BoolExpr) -> JoinPos
    where
        T: Relation,
        U: Relation
{
    of.join(apply)
}

fn equal_schema<T, U>(left:&DataSource<T>, right:&DataSource<U>) -> bool
    where
        T: Relation,
        U: Relation
{
    left.source.names().is_equal(right.source.names())
}

///Set operations
fn union_all<T, U>(left:&DataSource<T>, right:&DataSource<U>) -> Frame
    where
        T: Relation,
        U: Relation
{
    assert!(equal_schema(left, right), "The schema of both relations must be equal");
    let mut columns = Vec::with_capacity(left.source.col_count() + right.source.col_count());

    for (i, field) in left.source.names().columns.iter().enumerate() {
        let col = left.source.col(i);
        let more = right.source.col(i).data.as_ref().clone();

        columns.push(col.append(more));
    }

    Frame::new(left.source.names().clone(), columns)
}

fn _check_not_found(cmp:&HashMap<Data, usize>, row:&Data) -> bool {
    !cmp.contains_key(&row)
}

fn _check_found(cmp:&HashMap<Data, usize>, row:&Data) -> bool {
    cmp.contains_key(&row)
}

fn _compare_hash<T, U>(left:&DataSource<T>, right:&DataSource<U>, mark_found:bool) -> Vec<isize>
    where
        T: Relation,
        U: Relation
{
    assert!(equal_schema(left, right), "The schema of both relations must be equal");
    left.first();
    right.first();

    let mut results = Vec::new();
    let cmp = hash_rel(&left);
    let check =
        if mark_found {
            _check_found
        }  else {
            _check_not_found
        };

    while !right.eof() {
        let row = right.row();
        if check(&cmp, &row) {
            results.push(right.pos() as isize)
        }
        right.next();
    }
    results
}

fn intersection<T, U>(left:&DataSource<T>, right:&DataSource<U>) -> Frame
    where
        T: Relation,
        U: Relation
{
    let results = _compare_hash(&left, &right, true);
    let columns = materialize(&right.source, &results, false);

    Frame::new(left.source.names().clone(), columns)
}

fn difference<T, U>(left:&DataSource<T>, right:&DataSource<U>) -> Frame
    where
        T: Relation,
        U: Relation
{
    let results1 = _compare_hash(&left, &right, false);
    let results2 = _compare_hash(&right, &left, false);

    let mut columns = Vec::with_capacity(left.source.col_count());

    let col1 = materialize(&right.source, &results1, false);
    let col2 = materialize(&left.source, &results2, false);

    for (i, col) in col1.into_iter().enumerate() {
        let data = col2[i].data.clone();
        columns.push(col.append_data(data));
    }

    Frame::new(left.source.names().clone(), columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nums1() -> Vec<i64> {
        vec![1, 2, 3]
    }

    fn make_nums2() -> Vec<i64> {
        vec![2, 3, 4]
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

    fn make_single1() -> DataSource<Frame> {
        let nums = make_nums1();
        make_rel(vec![nums])
    }

    fn make_single2() -> DataSource<Frame> {
        let nums = make_nums2();
        make_rel(vec![nums])
    }

    fn make_single3() -> DataSource<Frame> {
        let nums = make_nums3();
        make_rel(vec![nums])
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

    fn make_both(left:Vec<i64>, right:Vec<i64>) -> (DataSource<Frame>, DataSource<Frame>, Vec<ColumnExp>, Vec<ColumnExp>) {
        let f1 = make_rel(vec![left]);
        let f2 = make_rel(vec![right]);
        let (pick1, pick2) = (vec![col(0)],  vec![col(0)]);

        (f1, f2, pick1, pick2)
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
        println!("Select {}", query1);
        assert_eq!(query1.names.len(), 1);

        let query2 = deselect(&f1, names2);
        println!("Deselect {}", query2);
        assert_eq!(query2.names.len(), 1);
    }

    fn _test_join(left:Vec<i64>, right:Vec<i64>, mut join_left:Vec<isize>, mut join_right:Vec<isize>)
    {
        let (ds1, ds2, p1, p2) = make_both(left, right);
        let both = joiner(&ds1, &ds2, &p1, &p2);

        let pos = join_both(&both, &PartialEq::eq);

        let mut l = pos.left.clone();
        let mut r = pos.right.clone();

        l.sort();
        r.sort();
        join_left.sort();
        join_right.sort();

        assert_eq!(l, join_left, "Left Side");
        assert_eq!(r, join_right, "Right Side");
    }

    #[test]
    fn test_join() {
        _test_join(vec![1], vec![1], vec![0], vec![0]);
        _test_join(vec![1], vec![2], vec![-1, 0], vec![-1, 0]);
        _test_join(vec![1], vec![], vec![0], vec![-1]);
        _test_join(vec![1, 2, 3], vec![1, 2, 3], vec![0, 1, 2], vec![0, 1, 2]);
        _test_join(vec![1, 2, 3], vec![2, 3, 4], vec![0, 1, 2, -1], vec![-1, 0, 1, 2]);
        _test_join(vec![1, 1, 1], vec![2, 3, 4], vec![0, 1, 2, -1, -1, -1], vec![-1, -1, -1, 0, 1, 2]);
    }

    #[test]
    fn test_cross_join() {
        let left = make_nums1();
        let right = make_nums2();

        let (ds1, ds2, p1, p2) = make_both(left, right);
        let both = joiner(&ds1, &ds2, &p1, &p2);
        let result = both.cross();
        println!("Cross {}", result);
        let col1 = result.col(0);
        let col2 = result.col(1);
        let r1:Vec<i64> =vec![1, 1, 1, 2, 2, 2, 3, 3, 3];
        let r2:Vec<i64> =vec![2, 3, 4, 2, 3, 4, 2, 3, 4];

        assert_eq!(col1, Data::from(r1));
        assert_eq!(col2, Data::from(r2));
    }

    #[test]
    fn test_union() {
        let left = make_nums1();
        let right = make_nums2();
        let (ds1, ds2, p1, p2) = make_both(left, right);

        let result = union_all(&ds1, &ds2);
        let r1:Vec<i64> =vec![1, 2, 3, 2, 3, 4];
        println!("Union {}", result);

        assert_eq!(result.col(0), Data::from(r1));
    }

    #[test]
    fn test_intersection() {
        let left = make_nums1();
        let right = make_nums2();
        let (ds1, ds2, p1, p2) = make_both(left, right);

        let result = intersection(&ds1, &ds2);
        let r1:Vec<i64> =vec![2, 3];
        println!("intersection {}", result);

        assert_eq!(result.col(0), Data::from(r1));
    }

    #[test]
    fn test_difference() {
        let left = make_nums1();
        let right = make_nums2();
        let (ds1, ds2, p1, p2) = make_both(left, right);

        let result = difference(&ds1, &ds2);
        let r1:Vec<i64> =vec![4, 1];
        println!("difference {}", result);

        assert_eq!(result.col(0), Data::from(r1));
    }
}
