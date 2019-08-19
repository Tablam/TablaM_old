use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn scalar() {
    let s1 = int64(1);
    check_query(s1.clone(), find_1(), int64(1));
    check_query(s1, find_1000(), empty_I64());
}

#[test]
fn vector() {
    let v1 = rel_nums1();
    check_query(v1.clone(), find_1(), array(&[1i64]));
    check_query(v1, find_1000(), empty_I64());
}

#[test]
fn table() {
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
fn seq() {
    let v1 = rel_nums1().as_seq();
    check_query(v1.clone(), find_1(), array(&[1i64]));
    check_query(v1, find_1000(), empty_I64());
}
