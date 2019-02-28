#![allow(unused_imports)]
extern crate bit_vec;
use self::bit_vec::BitVec;

use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;

use super::ndarray::*;
use super::types::*;
use crate::types::Query::Where;
//use super::relational::*;

pub fn decode<T:From<Scalar>>(values:&[Scalar]) -> Vec<T> {
    values.iter().map(move |x| From::from(x.clone())).collect()
}

pub fn field(name:&str, kind:DataType) -> Field {
    Field::new(name, kind)
}

pub fn schema(names:&[(&str, DataType)]) -> Schema {
    let fields = names
        .into_iter()
        .map(|(name, kind)| Field::new(name, *kind)).collect();

    Schema::new(fields)
}

pub fn schema_single(name:&str, kind:DataType) -> Schema {
    Schema::new_single(name, kind)
}
pub fn schema_it(kind:DataType) -> Schema {
    schema_single("it", kind)
}

pub fn schema_build(names:&[(&str, DataType)]) -> Schema {
    let fields = names
        .into_iter()
        .map(|(name, kind)| Field::new(name, *kind)).collect();

    Schema::new(fields)
}

pub fn schema_kv(types:[DataType; 2]) -> Schema {
    let key = field("key", types[0]);
    let value = field("value", types[1]);

    Schema::new(vec![key, value])
}

pub fn colp(pos:usize) -> ColumnName {
    ColumnName::Pos(pos)
}
pub fn coln(name:&str) -> ColumnName {
    ColumnName::Name(name.to_string())
}

pub fn value<T>(x:T) -> Scalar
    where T:From<Scalar>, Scalar: From<T>
{
    Scalar::from(x)
}

pub fn none() -> Scalar
{
    Scalar::default()
}

pub fn infer_type(of:&NDArray) -> DataType {
    if of.is_empty() {
        DataType::None
    } else {
        of.get([0, 0]).unwrap().kind()
    }
}

pub fn infer_types(of:&NDArray) -> Vec<DataType> {
    of.iter().map(|x| x.kind()).collect()
}

pub fn col<T>(x:&[T]) -> Vec<Scalar>
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    x.iter().map( |v| value(v.clone())).collect()
}

pub fn concat(values:Vec<Col>) -> (usize, Col)
{
    let mut data = Vec::new();
    let mut rows = 0;
    for row in values {
        rows += 1;
        data.extend(row.into_iter());
    }

    (rows, data)
}

pub fn nd_array_empty(rows:usize, cols:usize) -> NDArray
{
    NDArray::new(0, 0, [].to_vec())
}

pub fn nd_array<T>(of:&[T], rows:usize, cols:usize) -> NDArray
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let col = col(of);
    NDArray::new(rows, cols, col)
}

pub fn rcol_t<T>(name:&str, kind:DataType, of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, of.len(), 1);

    Table::new(schema_single(name, kind), data)
}

pub fn rcol<T>(name:&str, of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, of.len(), 1);
    let kind = infer_type(&data);

    Table::new(schema_single(name, kind), data)
}

pub fn array<T>(of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    rcol("it", of)
}

pub fn array_t<T>(kind:DataType, of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    rcol_t("it", kind, of)
}

pub fn array_empty(kind:DataType) -> Table
{
    Table::empty(Schema::scalar_field(kind))
}

pub fn row<T>(names:Schema, of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, 1, of.len());
    Table::new(names, data)
}

pub fn row_infer<T>(of:&[T]) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, 1, of.len());

    let types = infer_types(&data);
    let names = Schema::generate(&types);
    Table::new(names, data)
}

pub fn value_t<T>(of:T) -> Table
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    row_infer(&[of])
}

pub fn table_cols_infer(of: &NDArray) -> Table {
    let mut types = Vec::with_capacity(of.cols());
    for c in of.col_iter() {
        types.push(infer_type(&c.into_array()));
    }
    let names = Schema::generate(&types);

    Table::new(names, of.pivot())
}

pub fn table_cols(schema:Schema, of: &NDArray) -> Table {
    Table::new(schema, of.pivot())
}

pub fn table_btree(schema:Schema, of: &NDArray) -> BTree {
    BTree::new_from(schema, &table_cols_infer(&of))
}

pub fn table_rows(schema:Schema, of: NDArray) -> Table {
    Table::new(schema, of)
}

//Fundamental relational operators.

//
//pub fn select<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
//    let old = of.schema();
//    let pos = old.resolve_pos_many(pick);
//    let names = old.only(pos.as_slice());
//    T::new_from(names, &of.rows_pos(pos))
//}
//
//pub fn deselect<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
//    let old = of.schema();
//    let pos = old.resolve_pos_many(pick);
//
//    let deselect = old.except(pos.as_slice());
//    let names = old.deselect(&pos);
//    T::new_from(names, &of.rows_pos(deselect))
//}
//
//pub fn rename<T:Relation>(of:&T, change:&[(ColumnName, &str)]) -> T {
//    let schema = of.schema().rename(change);
//    of.clone_schema(&schema)
//}
//
//pub fn where_value_late<T:Relation>(of:&T, col:usize, value:&Scalar, apply:&BoolExpr) -> T {
//    let rows = T::find_all_rows(of, col, value, apply);
//
//    T::new_from(of.schema().clone(), &rows)
//}
//
//pub fn cross<T:Relation, U:Relation>(from:&T, to:&U) -> T
//{
//    let names = from.schema();
//    let others = &from.schema().join(to.schema());
//
//    let cols = names.len() + others.len();
//    let rows = from.row_count() * to.row_count();
//    //println!("{:?} {:?} ",names, others);
//
//    let mut data = vec![Scalar::None; rows * cols];
//    let mut pos:usize = 0;
//
//    for left in from.rows_iter() {
//        for right in 0..to.row_count() {
//            let mut extra_row = to.row_only(right, others);
//            let mut row = left.to_vec();
//            row.append(&mut extra_row);
//            //println!("{:?} {} {} {}", row, cols, rows,pos);
//            write_row(&mut data,  cols, rows, pos, row);
//            pos += 1;
//        }
//    }
//    let schema = names.extend(&to.schema().only(others));
//
//    T::from_vector(schema, rows, cols, data)
//}
//
//pub fn union<T:Relation, U:Relation>(from:&T, to:&U) -> T
//{
//    assert_eq!(from.schema(), to.schema(), "The schemas must be equal");
//
//    T::union(from, to)
//}
//
//pub fn intersection<T:Relation, U:Relation>(from:&T, to:&U) -> T
//{
//    let names = from.schema();
//    assert_eq!(names, to.schema(), "The schemas must be equal");
//    let (pos, null_count) = _compare_hash(from, to, true);
//
//    let data = to.materialize_raw(&pos, null_count, false);
//
//    T::from_vector(names.clone(), pos.len() - null_count,names.len(),  data)
//}
//
//pub fn difference<T:Relation, U:Relation>(from:&T, to:&U) -> T
//{
//    let names = from.schema();
//    assert_eq!(names, to.schema(), "The schemas must be equal");
//    let (pos1, null_count1) = _compare_hash(from, to, false);
//    let (pos2, null_count2) = _compare_hash(to, from, false);
//
//    let mut data = to.materialize_raw(&pos1, null_count1,  false);
//    let mut data2 = from.materialize_raw(&pos2, null_count2, false);
//    data.append(&mut data2);
//    let total_rows = (pos1.len() - null_count1) + (pos2.len() - null_count2);
//
//    T::from_vector(names.clone(), total_rows, names.len(), data)
//}
//
//pub fn _join_late<T:Relation, U:Relation>(from:&T, to:&U, cols_from:&[usize], null_from:bool, cols_to:&[usize], null_to:bool, apply: &BoolExpr) -> (BitVec, BitVec) {
//    let mut right_not_founds = HashSet::new();
//
//    let left = from.row_count();
//    let right = to.row_count();
//
//    let total = cmp::max(left, right);
//
//    let mut cols_left  = BitVec::from_elem(total, false);
//    let mut cols_right = BitVec::from_elem(total, false);
//
//    let mut found = false;
//    let mut first = true;
//
//    for x in 0..left {
//        for y in 0..right  {
//            if null_from && first {
//                right_not_founds.insert(y);
//            }
//            let l = &from.row_only(x, cols_from);
//            if to.cmp_cols(y, cols_to, l.as_slice(), apply) {
//                //println!("{} -> {} true", x, y);
//                cols_left.set(x, true);
//                cols_right.set(y, true);
//                right_not_founds.remove(&y);
//                found = true;
//            }
//        }
//        if null_from && !found {
//            //println!("..{} true", x);
//            cols_left.set(x, true);
//        }
//        found = false;
//        first = false;
//    }
//
//    if null_to && !right_not_founds.is_empty() {
//        cols_left.grow(right_not_founds.len(), false);
//        cols_right.grow(right_not_founds.len(), false);
//
//        for pos in right_not_founds {
//            cols_right.set(pos, true);
//        }
//    }
//
//    (cols_left, cols_right)
//}
//
//pub fn join<T:Relation, U:Relation>(from:&T, to:&U, join:Join, cols_from:&[usize], cols_to:&[usize], apply:&BoolExpr) -> T
//{
//    let null_lefts = join.produce_null(true);
//    let null_rights = join.produce_null(false);
//
//    let (left, right) = _join_late(from, to, cols_from, null_lefts, cols_to, null_rights, apply);
//    let names = from.schema();
//
//    let others= &names.join(to.schema());
//    let cols = names.len() + others.len();
//
//    let data = from.materialize_data(&left, null_lefts);
//    let data2 = to.materialize_data(&right, null_rights);
//
//    println!("L: {:?}", data);
//    println!("R: {:?}", data2);
//
//    let result = data.hcat(&data2);
//
//    println!("RESULT: {:?}", result);
//    let schema = names.extend(&to.schema().only(others));
//    T::from_vector(schema, result.rows(), cols, result.into_vec())
//}
//
//pub fn append<T:Relation, U:Relation>(to:&T, from:&U) -> T {
//    assert_eq!(to.schema(), from.schema(), "The schemas must be equal");
//
//    let total_rows = to.row_count() + from.row_count();
//    let mut left = to.flat_raw();
//    let mut right=  from.flat_raw();
//    //println!("APP: {} {:?}\n {:?}\n", total_rows, left, right);
//
//    left.append(&mut right);
//    //println!("R: {:?}", left);
//
//    T::from_vector(to.schema().clone(),  total_rows, to.col_count(), left)
//}

//
//pub fn scan_op<T:Relation>(op:T) -> Scan<T> {
//    Scan::new(op)
//}
//
//pub fn where_op<T:RelOp>(op:T, cmp:CmOp) -> Where<T> {
//    Where::new(op,  cmp)
//}