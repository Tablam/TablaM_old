use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn test_project() {}

#[test]
fn test_where() {
    let v1 = rel_nums1();
    check_query(v1.clone(), find_1(), array(&[1i64]));
    check_query(v1, find_1000(), empty_I64());
}

#[test]
fn test_union() {
    let r1 = array(&[1i64]);
    let r2 = array(&[2i64]);

    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(r2.into());
    check_query(r1, cmp.clone(), result);
}

#[test]
fn test_intersection() {}

#[test]
fn test_difference() {}

#[test]
fn test_joins() {}
