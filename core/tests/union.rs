use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn scalar() {
    let s1 = int64(1);
    let s2 = int64(2);

    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(s2.into());
    check_query(s1, cmp.clone(), result.clone());
}

#[test]
fn vector() {
    let r1 = array(&[1i64]);
    let r2 = array(&[2i64]);

    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(r2.into());
    check_query(r1, cmp.clone(), result);
}

#[test]
fn table() {
    let r1 = table_rows(
        table_1().schema.clone(),
        vec![vec![int64(1), int64(2), bool(true)]],
    );

    let r2 = table_rows(
        r1.schema.clone(),
        vec![vec![int64(2), int64(1), bool(false)]],
    );

    let result = table_rows(
        r1.schema.clone(),
        vec![
            vec![int64(1), int64(2), bool(true)],
            vec![int64(2), int64(1), bool(false)],
        ],
    );

    let cmp = Query::union(r2.into());
    check_query(r1, cmp.clone(), result);
}

#[test]
fn seq() {
    let r1 = array(&[1i64]).as_seq();
    let r2 = array(&[2i64]).as_seq();
    let result = array(&[1i64, 2i64]);

    let cmp = Query::union(r2.into());
    check_query(r1, cmp.clone(), result);
}
