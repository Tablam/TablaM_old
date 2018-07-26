#![allow(dead_code)]

use std::rc::Rc;
use std::fmt::Debug;

extern crate bytes;

use self::bytes::*;

/// Marker trait for the values
pub trait Value: Clone + Debug {}

impl Value for f64 {}
impl Value for i64 {}
impl Value for String {}
impl Value for BytesMut {}

type RVec<T> = Rc<Vec<T>>;

#[derive(Debug, Clone)]
pub enum Layout {
	Scalar,
    Row,
    Col,
}

#[derive(Debug, Clone)]
pub enum Ops {
	Project,
    Filter,
}

#[derive(Debug, Clone)]
pub enum Scalar {
	None,
    //I32(i32),
    I64(i64),
    UTF8(BytesMut),
}

#[derive(Debug, Clone)]
pub enum Column {
    //F64(RVec<f64>),
    I64(RVec<i64>),
    UTF8(RVec<BytesMut>),
}

#[derive(Debug)]
struct Frame {
    columns: Vec<Column>,
    rows: usize,
}

#[derive(Debug, Clone)]
pub enum ColumnExp {
	Value(Scalar, String),
    Colum(String),
}

#[derive(Debug, Clone)]
pub enum Compare {
    Eq(ColumnExp, ColumnExp),
    NotEq(ColumnExp, ColumnExp),
    Less(ColumnExp, ColumnExp),
    Bigger(ColumnExp, ColumnExp),
    //Like(ColumnExp, ColumnExp),
}

pub type Data = Vec<Scalar>;
pub type Names = Vec<String>;

#[derive(Debug, Clone)]
pub struct Relation {
    pub layout: Layout,
	pub names: Names,
    pub data:  Vec<Data>
}

pub type SelectColumns = Vec<ColumnExp>;

#[derive(Debug, Clone)]
pub enum Algebra {
    Project(Option<SelectColumns>),
    Rename(ColumnExp, String),    

	Filter(Compare), //aka: Select

    // Union,
    // Interseccion,
    // Difference,
}

#[derive(Debug, Clone)]
pub struct QueryPlanner {
    pub ops: Vec<Algebra>,
}

fn encode_str(value:String) -> BytesMut {
    BytesMut::from(value)
}
