use std::rc::Rc;

use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn test_project() {}

#[test]
fn test_where() {
    let s1 = int64(1);
    check_query(s1.clone(), find_1(), int64(1));
    check_query(s1, find_1000(), empty_I64());
}

#[test]
fn test_union() {
    let s1 = int64(1);
    let s2 = int64(2);

    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(s2.into());
    check_query(s1, cmp.clone(), result.clone());
}

#[test]
fn test_intersection() {}

#[test]
fn test_difference() {}

#[test]
fn test_joins() {}
