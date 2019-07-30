#![allow(unused_variables)]
#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

extern crate decorum;
use decorum::R64;

extern crate bit_vec;
use self::bit_vec::BitVec;

extern crate chrono;
use chrono::prelude::*;

extern crate rust_decimal;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
pub enum Join {
    Left,
    Right,
    Inner,
    Full, //, Natural, Cross
}

impl Join {
    pub fn produce_null(self, is_left: bool) -> bool {
        match self {
            Join::Left => !is_left,
            Join::Right => is_left,
            Join::Inner => false,
            Join::Full => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    Scalar,
    KV(usize),
    Row(usize),
    Vector(usize),
    Table(usize, usize),
}

impl Shape {
    pub fn size(&self) -> (usize, usize) {
        match self {
            Shape::Scalar => (1, 1),
            Shape::KV(rows) => (1, *rows),
            Shape::Row(cols) => (*cols, 1),
            Shape::Vector(rows) => (1, *rows),
            Shape::Table(cols, rows) => (*cols, *rows),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Minus,
    Mul,
    Div,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CompareOp {
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrudOp {
    Create,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexOp {
    Pos,
    Name,
}

pub enum KeyValue {
    Key,
    Value,
}

#[derive(Debug, Clone)]
pub enum SetQuery {
    Union,
    Diff,
    Intersection,
}

//NOTE: This define a total order, so it matter what is the order
//of the enum! The overall sorting order is defined as:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataType {
    None,
    Any,
    Bool,
    //Numeric
    I32,
    ISize,
    I64,
    F64,
    Decimal,
    //Dates
    DateTime,
    //Text
    UTF8,
    //Complex
    Rel, // Planed: BitVec, Blob, Sum(DataType), Product(DataType), Rel(Vec<Field>)
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type TimeStamp = DateTime<Local>;

//NOTE: The order of this enum must match DataType
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    None, //null
    Bool(bool),
    //Numeric
    I32(i32),
    ISize(isize),
    I64(i64),
    F64(R64),
    Decimal(Decimal),
    DateTime(TimeStamp),
    UTF8(String),
    //Complex
    Rel(Rc<Rel>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rel {
    One(Scalar),
    Vector(Vector),
    Range(Range),
    Table(Table),
    Seq(Seq),
}

//Type Alias...
pub type BoolExpr = dyn Fn(&Scalar, &Scalar) -> bool;
pub type BinExpr = dyn Fn(&Scalar, &Scalar) -> Scalar;
pub type UnaryExpr = dyn Fn(&Scalar) -> Scalar;
pub type Col = Vec<Scalar>;
pub type BCol = Vec<Box<Scalar>>;
pub type Pos = Vec<usize>;
pub type Tree = BTreeMap<Scalar, Scalar>;
pub type RScalar = Rc<Scalar>;
pub type RSchema = Rc<Schema>;

#[derive(Debug, Clone)]
pub enum ColumnName {
    Name(String),
    Pos(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    pub name: String,
    pub kind: DataType,
}

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct Schema {
    pub columns: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Vector {
    pub schema: Schema,
    pub data: Col,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Range {
    pub schema: Schema,
    pub start: usize,
    pub end: usize,
    pub step: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct BTree {
    pub schema: Schema,
    pub data: BTreeMap<Scalar, Scalar>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Table {
    pub schema: Schema,
    pub data: Vec<Col>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmOp {
    pub op: CompareOp,
    pub lhs: usize,
    pub rhs: Rc<Scalar>,
}

impl CmOp {
    fn new(op: CompareOp, lhs: usize, rhs: Rc<Scalar>) -> Self {
        CmOp { op, lhs, rhs }
    }

    pub fn eq(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::Eq, lhs, rhs)
    }
    pub fn not(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::NotEq, lhs, rhs)
    }
    pub fn less(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::Less, lhs, rhs)
    }
    pub fn less_eq(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::LessEq, lhs, rhs)
    }
    pub fn greater(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::Greater, lhs, rhs)
    }
    pub fn greater_eq(lhs: usize, rhs: Rc<Scalar>) -> Self {
        Self::new(CompareOp::GreaterEq, lhs, rhs)
    }

    pub fn get_fn(&self) -> &BoolExpr {
        match self.op {
            CompareOp::Eq => &PartialEq::eq,
            CompareOp::NotEq => &PartialEq::ne,
            CompareOp::Less => &PartialOrd::lt,
            CompareOp::LessEq => &PartialOrd::le,
            CompareOp::Greater => &PartialOrd::gt,
            CompareOp::GreaterEq => &PartialOrd::ge,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Query {
    //  To ask for all the rows, send a empty query
    //    Distinct,
    Where(CmOp),
    Limit(usize, usize), // skip * limit
    //Sort(bool, usize),   // true=ascending * col pos
    //Select(Pos),
    //Project(Col, String),
    //    Project(Box<UnaryExpr>),
    //Group(Pos),
    //Join(Join),
    Set(SetQuery, Rc<Rel>),
}

impl Query {
    pub fn eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::eq(lhs, rhs.into()))
    }

    pub fn not(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::not(lhs, rhs.into()))
    }

    pub fn less(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::less(lhs, rhs.into()))
    }

    pub fn less_eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::less_eq(lhs, rhs.into()))
    }

    pub fn greater(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::greater(lhs, rhs.into()))
    }

    pub fn greater_eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::greater_eq(lhs, rhs.into()))
    }

    pub fn union(rhs: Rel) -> Self {
        Query::Set(SetQuery::Union, Rc::new(rhs))
    }

    pub fn diff(rhs: Rel) -> Self {
        Query::Set(SetQuery::Diff, Rc::new(rhs))
    }

    pub fn intersection(rhs: Rel) -> Self {
        Query::Set(SetQuery::Intersection, Rc::new(rhs))
    }
}

#[derive(Debug)]
pub struct ColIter<'a, R> {
    pub pos: usize,
    pub col: usize,
    pub rel: &'a R,
}

#[derive(Debug)]
pub struct RowsIter<R> {
    pub pos: usize,
    pub rel: R,
}

impl<R> RowsIter<R> {
    pub fn new(rel: R) -> Self {
        RowsIter { pos: 0, rel }
    }
}

pub trait RelIter {
    fn pos(&self) -> usize;
    fn advance(&mut self) -> bool;
    fn row(&mut self) -> Col;
    fn next(&mut self) -> Option<Col> {
        if self.advance() {
            Some(self.row())
        } else {
            None
        }
    }
}

pub fn ref_cell<T>(of: T) -> Rc<RefCell<dyn RelIter>>
where
    T: RelIter + 'static,
{
    Rc::new(RefCell::new(of))
}

pub struct Seq {
    pub schema: Schema,
    pub shape: Shape,
    pub iter: Rc<RefCell<dyn RelIter>>,
}

pub trait Relation: Debug {
    //fn schema(&self) -> Rc<Schema>;

    fn shape(&self) -> Shape;

    fn len(&self) -> usize {
        let (col_count, row_count) = self.shape().size();
        row_count * col_count
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn rows(&self) -> RowsIter<Self>
    where
        Self: Sized;

    fn as_seq(&self) -> Seq;

    fn filter(&self, cmp: CmOp) -> Rel;

    fn union(&self, other: &Rel) -> Rel;
    fn diff(&self, other: &Rel) -> Rel;
    fn intersect(&self, other: &Rel) -> Rel;

    //fn join(&self, cmp: CmOp) -> Self;

    //fn project(&self, cmp: CmOp) -> Self;
    //fn extend(&self, cmp: CmOp) -> Self;
    //fn rename(&self, cmp: CmOp) -> Self;
}
