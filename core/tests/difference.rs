use std::rc::Rc;

use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn scalar() {
    let s1 = int64(1);
    let s2 = int64(2);

    let cmp = Query::diff(s2.into());
    check_query(s1.clone(), cmp.clone(), s1.clone());

    let cmp = Query::diff(s1.clone().into());
    check_query(s1.clone(), cmp.clone(), empty_I64());
}

#[test]
fn vector() {
    let r1 = array(&[1i64]);
    let r2 = array(&[2i64]);

    let cmp = Query::diff(r2.clone().into());
    check_query(r1.clone(), cmp.clone(), r1.clone());

    let cmp = Query::diff(r1.clone().into());
    check_query(r1.clone(), cmp.clone(), empty_I64());
}

#[test]
fn table() {
    let r1 = table_rows(table_1().schema.clone(), vec![vec![int64(1)]]);
    let r2 = table_rows(r1.schema.clone(), vec![vec![int64(2)]]);

    let cmp = Query::diff(r2.clone().into());
    check_query(r1.clone(), cmp.clone(), r1.clone());

    let cmp = Query::diff(r1.clone().into());
    check_query(r1.clone(), cmp.clone(), table_empty(r1.schema.clone()));
}

#[test]
fn seq() {
    let result1 = array(&[1i64]);
    let result2 = array(&[2i64]);

    let mut r1 = result1.as_seq();
    let r2 = result2.as_seq();

    let cmp = Query::diff(r2.into());
    check_query(r1.materialize(), cmp.clone(), result2.clone());

    let cmp = Query::diff(r1.clone().into());
    check_query(r1.materialize(), cmp.clone(), empty_I64());
}
