use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

fn _schema() -> Schema {
    schema(&[("col0", DataType::I64), ("col1", DataType::I64)])
}

#[test]
fn scalar() {
    let s1 = int64(1);
    let s2 = int64(2);

    let result = table_rows(_schema(), vec![vec![int64(1), int64(2)]]);

    check_query(s1, Query::cross(s2.into()), result);
}

fn _cross_result() -> Table {
    table_rows(
        _schema(),
        vec![
            vec![int64(1), int64(1)],
            vec![int64(1), int64(2)],
            vec![int64(1), int64(3)],
            vec![int64(2), int64(1)],
            vec![int64(2), int64(2)],
            vec![int64(2), int64(3)],
            vec![int64(3), int64(1)],
            vec![int64(3), int64(2)],
            vec![int64(3), int64(3)],
        ],
    )
}

#[test]
fn vector() {
    let v1 = rel_nums1();

    check_query(v1.clone(), Query::cross(v1.clone().into()), _cross_result());
}

#[test]
fn table() {
    let nums = col(&nums_1());

    let v1 = table_cols(schema(&[("col0", DataType::I64)]), &vec![nums.clone()]);
    let v2 = table_cols(schema(&[("col1", DataType::I64)]), &vec![nums]);

    check_query(v1.clone(), Query::cross(v2.into()), _cross_result());
}

#[test]
fn seq() {
    let v1 = rel_nums1().as_seq();
    check_query(v1.clone(), Query::cross(v1.into()), _cross_result());
}
