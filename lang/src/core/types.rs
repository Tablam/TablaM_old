#![allow(dead_code)]
use std::fmt::Debug;
use std::rc::Rc;
use std::slice::Iter;

extern crate bytes;

use self::bytes::*;

pub type RVec<T> = Rc<Vec<T>>;
pub type Names = Vec<String>;

/// Marker trait for the values
pub trait Value: Clone + Debug {}

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
    Int64,
//    Planed:
//    Decimal,
//    Time,
//    Date,
//    DateTime,
//    Char,
//    UTF8,
//    Byte,
//    Sum(DataType), //enums
//    Product(DataType), //struct
//    Rel((String, DataType)), //Relations, Columns
//    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
	None, //null
    BOOL(bool),
    I32(i32),
    I64(i64),
    UTF8(BytesMut),
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Column {
    BOOL(RVec<bool>),
    I32(RVec<i32>),
    I64(RVec<i64>),
    UTF8(RVec<BytesMut>),    
    ROW(RVec<Column>),
}

impl Value for bool {}
impl Value for i32 {}
impl Value for i64 {}
impl Value for BytesMut {}
impl Value for Column {}

/// Recovers the elements type through iterators and slices
pub trait ColumnIter: Value {
    fn as_slice<'b>(col: &'b Column) -> &'b [Self];
    fn iter<'b>(col: &'b Column) -> Iter<'b, Self> {
        Self::as_slice(col).iter()
    }
}

impl ColumnIter for bool {
    fn as_slice<'b>(col: &'b Column) -> &'b [bool] {
        if let Column::BOOL(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [bool]", col)
        }
    }
}

impl ColumnIter for BytesMut {
    fn as_slice<'b>(col: &'b Column) -> &'b [BytesMut] {
        if let Column::UTF8(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [BytesMut]", col)
        }
    }
}

impl ColumnIter for Column {
    fn as_slice<'b>(col: &'b Column) -> &'b [Column] {
        if let Column::ROW(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [Column]", col)
        }
    }
}

impl ColumnIter for i32 {
    fn as_slice<'b>(col: &'b Column) -> &'b [i32] {
        if let Column::I32(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [i32]", col)
        }
    }
}

impl ColumnIter for i64 {
    fn as_slice<'b>(col: &'b Column) -> &'b [i64] {
        if let Column::I64(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [i64]", col)
        }
    }
}

/// `ColumnType` is the type of the elements if the columns.
/// It composes all column traits and is used as a type bound
/// to bring all the dependencies at once
pub trait ColumnType: Value {
    fn to_column(Vec<Self>) -> Column;
    fn iter<'b>(&'b Column) -> Iter<'b, Self>;
    fn as_slice<'b>(&'b Column) -> &'b [Self];
}

/// Implement `ColumnType` for each type that implements
/// `ColumnIter<Self>` and `From<Vec<Self>>` for `Column`
impl<T> ColumnType for T
    where
        T: ColumnIter + Value,
        Column: From<Vec<T>>,
{
    fn to_column(vec: Vec<T>) -> Column {
        vec.into()
    }

    fn iter<'b>(col: &'b Column) -> Iter<'b, T> {
        T::iter(col)
    }

    fn as_slice<'b>(col: &'b Column) -> &'b [T] {
        T::as_slice(col)
    }
}

impl Column {
    /// Construct a column from a vector
    pub fn from<T: ColumnType>(vec: Vec<T>) -> Column {
        T::to_column(vec)
    }

    pub fn from_scalar<T: ColumnType>(value: T) -> Column {
        T::to_column(vec!(value))
    }

    /// column.iter()
    pub fn iter<T: ColumnType>(&self) -> Iter<T> {
        T::iter(self)
    }

    /// column.as_slice()
    pub fn as_slice<T: ColumnType>(&self) -> &[T] {
        T::as_slice(self)
    }

    /// column.len()
    pub fn len<T: ColumnType>(&self) -> usize {
        T::as_slice(self).len()
    }

}

#[derive(Debug, Clone)]
pub struct Frame {
    pub names:Names,
    pub data :RVec<Column>,
}

impl Frame {
    pub fn empty() -> Self {
        Frame {
           names: Vec::new(),
           data: Rc::new(Vec::new()),
        }
    }

    pub fn new(columns: Vec<Column>) -> Self {
        let total = 0..columns.len();
        let names:Names = total.map(| x | format!("col{}", x).to_string() ).collect();

        Frame {
            names,
            data: Rc::new(columns),
        }
    }

}
#[derive(Debug, Clone)]
pub enum ColumnExp {
    //TODO: This complicate things. Support later constant values...
	//Value(Scalar),
    Name(String),
    Pos(usize),
}

#[derive(Debug, Clone)]
pub enum Operator {
    //Compare
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Not,
    //Math
    Add,
    Minus,
    Mul,
    Div,
    //Relational
    Union,
    Diff,
    //Utils
    IndexByPos,
    IndexByName,
}

#[derive(Debug, Clone)]
pub struct Compare {
    pub op:Operator,
    pub left:ColumnExp,
    pub right:ColumnExp,
}

pub type SelectColumns = Vec<ColumnExp>;

#[derive(Debug, Clone)]
pub enum Algebra {
    Project(Option<SelectColumns>),
    Rename(ColumnExp, String),    

	Filter(Compare), //aka: Select

    // Union,
    // Intersection,
    // Difference,
}

#[derive(Debug, Clone)]
pub struct QueryPlanner {
    pub ops: Vec<Algebra>,
}

pub trait RelationRow {
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;

    fn layout(&self) -> Layout {
       Layout::Row
    }

    fn names(&self) -> Names;
    fn row(&self, pos:usize) -> Frame;
    fn col(&self, pos:usize) -> Column;
    fn col_named(&self, name:&str) -> Column {
        let pos = self.names().as_slice().iter().position(|r| r == name).unwrap();
        self.col(pos)
    }
}

pub trait RelationCol:RelationRow {
    fn frame(&self) -> &Frame;
}

pub fn encode_str(value:&str) -> BytesMut {
    BytesMut::from(value)
}

//TODO: Use a macro for automate convertions?
impl From<i32> for Column {
    fn from(vec: i32) -> Self {
        Column::I32(Rc::from(vec!(vec)))
    }
}

impl From<Vec<i32>> for Column {
    fn from(vec: Vec<i32>) -> Self {
        Column::I32(Rc::from(vec))
    }
}

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

impl From<Vec<Column>> for Column {
    fn from(vec: Vec<Column>) -> Self {
        Column::ROW(Rc::from(vec))
    }
}

pub fn len(of:&Column) -> usize {
    match of {
        Column::I32(data)  => data.len(),
        Column::I64(data)  => data.len(),
        Column::BOOL(data) => data.len(),
        Column::UTF8(data) => data.len(),
        Column::ROW(data)  =>  data.len(),
    }
}

pub fn value(of:&Column, pos:usize) -> Column {
    match of {
        Column::I32(data)  => Column::from_scalar(data[pos]),
        Column::I64(data)  => Column::from_scalar(data[pos]),
        Column::BOOL(data) => Column::from_scalar(data[pos]),
        Column::UTF8(data) => Column::from_scalar(data[pos].clone()),
        Column::ROW(data)  => Column::from_scalar(data[pos].clone()),
    }        
}

pub fn row(of:&Vec<Column>, pos:usize) -> Vec<Column> {
    let data = of.as_slice().iter().map(| x | value(x, pos));

    data.collect()
}

impl RelationRow for Frame {
    fn row_count(&self) -> usize {
        if self.column_count() > 0 {
            return len(&self.data[0])
        }
        0
    }

    fn column_count(&self) -> usize {
        self.data.len()
    }

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

    fn col(&self, pos:usize) -> Column {
        self.data[pos].clone()
    }
}

impl RelationCol for Frame {
    fn frame(&self) -> &Frame {self}
}

pub fn to_i64(from:&Column) -> &RVec<i64> {
    if let Column::I64(data) = from {
        data
    } else {
        panic!("Improper cast of {:?} to [f64]", from)
    }
}
