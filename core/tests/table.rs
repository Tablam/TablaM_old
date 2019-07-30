use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn test_project() {}

#[test]
fn test_where() {
    let v1 = table_1();
    let result = table_rows(
        v1.schema.clone(),
        vec![vec![int64(1), int64(4), bool(true)]],
    );
    let empty = table_rows(v1.schema.clone(), vec![]);

    check_query(v1.clone(), find_1(), result);
    check_query(v1, find_1000(), empty);
}

#[test]
fn test_union() {}

#[test]
fn test_intersection() {}

#[test]
fn test_difference() {}

#[test]
fn test_joins() {}
