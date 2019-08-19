use std::rc::Rc;

use tablam_core::dsl::*;
use tablam_core::types::*;

mod common;
use crate::common::*;

#[test]
fn scalar() {
    let s1 = int64(1);
    let s2 = int64(2);

    let cmp = Query::intersection(s2.into());
    check_query(s1.clone(), cmp.clone(), empty_I64());

    let cmp = Query::intersection(s1.clone().into());
    check_query(s1.clone(), cmp.clone(), s1);
}

#[test]
fn vector() {
    let r1 = array(&[1i64]);
    let r2 = array(&[2i64]);

    let cmp = Query::intersection(r2.clone().into());
    check_query(r1.clone(), cmp.clone(), empty_I64());

    let cmp = Query::intersection(r1.clone().into());
    check_query(r1.clone(), cmp.clone(), r1);
}

#[test]
fn table() {
    let r1 = table_rows(table_1().schema.clone(), vec![vec![int64(1)]]);
    let r2 = table_rows(r1.schema.clone(), vec![vec![int64(2)]]);

    let cmp = Query::intersection(r2.clone().into());
    check_query(r1.clone(), cmp.clone(), table_empty(r1.schema.clone()));

    let cmp = Query::intersection(r1.clone().into());
    check_query(r1.clone(), cmp.clone(), r1);
}

#[test]
fn seq() {
    let mut r1 = array(&[1i64]).as_seq();
    let r2 = int64(2).as_seq();

    let cmp = Query::intersection(r2.into());
    check_query(r1.clone(), cmp.clone(), empty_I64());

    let cmp = Query::intersection(r1.clone().into());
    let result = r1.materialize();
    check_query(r1.clone(), cmp.clone(), result);
}
