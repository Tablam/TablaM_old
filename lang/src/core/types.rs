#![allow(dead_code)]

use std::rc::Rc;

extern crate bytes;

use self::bytes::*;

pub type RVec<T> = Rc<Vec<T>>;
pub type Names = Vec<String>;

#[derive(Debug, Clone)]
pub enum Layout {
	Scalar,
    Row,
    Col,
}

//NOTE: This define a total order, so matter what is the order
//of the enum! The overall sorting order is defined as:
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum DataType {
    None,
    Bool,
    Int32,
    UTF8,
}

#[derive(Debug, Clone)]
pub enum Ops {
	Project,
    Filter,
}

#[derive(Debug, Clone)]
pub enum Scalar {
	None, //null
    BOOL(bool),
    I64(i64),
    UTF8(BytesMut),
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Column {
    BOOL(RVec<bool>),
    I64(RVec<i64>),
    UTF8(RVec<BytesMut>),    
    ROW(RVec<Column>),
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub names:Names,
    pub data :RVec<Column>,
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

pub trait RelationRow {
    fn layout(&self) -> Layout {
       Layout::Row
    }

    fn names(&self) -> Names;
    fn row(&self, pos:usize) -> Frame;
}

pub fn encode_str(value:&str) -> BytesMut {
    BytesMut::from(value)
}

//TODO: Use a macro for automate convertions?
impl From<i64> for Column {
    fn from(vec: i64) -> Self {
        Column::I64(Rc::from(vec!(vec)))
    }
}

impl From<Vec<i64>> for Column {
    fn from(vec: Vec<i64>) -> Self {
        Column::I64(Rc::from(vec))
    }
}

impl From<bool> for Column {
    fn from(vec: bool) -> Self {
        Column::BOOL(Rc::from(vec!(vec)))
    }
}

impl From<Vec<bool>> for Column {
    fn from(vec: Vec<bool>) -> Self {
        Column::BOOL(Rc::from(vec))
    }
}

impl From<BytesMut> for Column {
    fn from(vec: BytesMut) -> Self {
        Column::UTF8(Rc::from(vec!(vec)))
    }
}

impl From<Vec<BytesMut>> for Column {
    fn from(vec: Vec<BytesMut>) -> Self {
        Column::UTF8(Rc::from(vec))
    }
}

pub fn value(of:&Column, pos:usize) -> Column {
    match of {
        Column::I64(data) =>  Column::from(data[pos]),
        Column::BOOL(data) => Column::from(data[pos]),
        Column::UTF8(data) => Column::from(data[pos].clone()),
        Column::ROW(data) =>  Column::from(data[pos].clone()),
    }        
}

pub fn row(of:&Vec<Column>, pos:usize) -> Vec<Column> {
    let data = of.into_iter().map(| x | value(x, pos));

    data.collect()
}

pub trait RelationCol:RelationRow {
    fn frame(&self) -> &Frame;
    fn col(&self, pos:usize) -> Column;
}

impl RelationRow for Frame {
    fn layout(&self) -> Layout {
        Layout::Col
    }

    fn names(&self) -> Names {
        self.names.clone()
    }

    fn row(&self, pos: usize) -> Frame
    {
        Frame {
            names: self.names(),
            data: Rc::new(row(&self.data, pos))
        }
    }
}

impl RelationCol for Frame {
    fn frame(&self) -> &Frame {self}

    fn col(&self, pos:usize) -> Column {
        self.data[pos].clone()
    }
}

pub fn to_i64(from:&Column) -> &RVec<i64> {
    if let Column::I64(data) = from {
        data
    } else {
        panic!("Improper cast of {:?} to [f64]", from)
    }
}