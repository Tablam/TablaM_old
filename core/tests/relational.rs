use std::rc::Rc;

use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn test_project() {}

#[test]
fn test_where() {
    let cmp = Query::eq(0, int64(1));
    let fail = Query::eq(0, int64(1000));
    let empty = array_empty(DataType::I64);

    let s1 = int64(1);
    check_query(s1.clone(), cmp.clone(), int64(1));
    check_query(s1, fail.clone(), empty.clone());

    let v1 = rel_nums1();
    check_query(v1.clone(), cmp.clone(), array(&[1i64]));
    check_query(v1, fail.clone(), empty);
}

#[test]
fn test_union() {
    let s1 = int64(1);
    let s2 = int64(2);
    let r1 = array(&[1i64]);
    let r2 = array(&[2i64]);

    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(s2.into());
    check_query(s1, cmp.clone(), result.clone());

    let cmp = Query::union(r2.clone().into());
    check_query(r1.clone(), cmp.clone(), result.clone());

    let iter = r1.as_seq();
    let iter2 = r2.as_seq();

    let cmp = Query::union(iter2.into());
    check_query(iter, cmp.clone(), result);
}

#[test]
fn test_intersection() {}

#[test]
fn test_difference() {}

#[test]
fn test_joins() {}
