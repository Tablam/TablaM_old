use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn test_project() {}

#[test]
fn test_where() {
    let v1 = range_to(10);

    check_query(v1.clone(), Query::eq(0, pint(1)), array(&[1isize]));
    check_query(v1, Query::eq(0, pint(1000)), array_empty(DataType::ISize));
}

#[test]
fn test_union() {
    let r1 = range_to(10);
    let r2 = range_to(100);

    let result = range_to(100);

    let cmp = Query::union(r2.into());
    check_query(r1, cmp.clone(), result);
}

#[test]
fn test_intersection() {}

#[test]
fn test_difference() {}

#[test]
fn test_joins() {}
