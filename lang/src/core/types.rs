#![allow(dead_code)]
#![allow(unused_imports)]
use std::fmt::Debug;
use std::rc::Rc;
use std::ops::Index;
use std::fmt;

extern crate bytes;
use self::bytes::*;

pub type RVec<T> = Rc<Vec<T>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
	Scalar,
    Row,
    Col,
}

//NOTE: This define a total order, so matter what is the order
//of the enum! The overall sorting order is defined as:
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum DataType {
    None,
    Bool,
    I32,
    I64,
//    Planed:
//    F32,
//    F64,
//    Decimal,
//    Time,
//    Date,
//    DateTime,
//    Char,
    UTF8,
//    Byte,
    Tuple,
//    Sum(DataType), //enums
//    Product(DataType), //struct
//    Rel(Vec<Field>), //Relations, Columns
//    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub kind: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum Scalar {
	None, //null
    Bool(bool),
    I32(i32),
    I64(i64),
    UTF8(BytesMut),
    Tuple(Vec<Scalar>),
}

impl Scalar {
    pub fn repeat(of:&Scalar, times:usize) -> Vec<Scalar> {
        let mut result = Vec::with_capacity(times);
        for i in 0..times {
            result.push(of.clone());
        }
        result
    }
}

pub type BoolExpr = Fn(&Scalar, &Scalar) -> bool;

fn type_of_scalar(value:&Scalar) -> DataType {
   match value {
       Scalar::None => DataType::None,
       Scalar::Bool(_) => DataType::Bool,
       Scalar::I32(_) => DataType::I32,
       Scalar::I64(_) => DataType::I64,
       Scalar::UTF8(_) => DataType::UTF8,
       Scalar::Tuple(_) => DataType::Tuple,
   }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Data {
    pub kind:   DataType,
    pub len:    usize,
    pub data:   RVec<Scalar>,
}

#[derive(Debug, Clone)]
pub enum ColumnExp {
    Name(String),
    Pos(usize),
}

pub fn col(pos:usize) -> ColumnExp {
    ColumnExp::Pos(pos)
}
pub fn coln(name:&str) -> ColumnExp {
    ColumnExp::Name(name.to_string())
}

#[derive(Debug, Clone)]
pub enum Join {
    Left,
    Right,
    Inner,
    Full,
}

impl Join {
    pub fn produce_null(&self, is_left:bool) -> bool
    {
        match self {
            Join::Left  => !is_left,
            Join::Right => is_left,
            Join::Inner => false,
            Join::Full  => true,
        }
    }
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

pub fn encode_str(value:&str) -> BytesMut {
    BytesMut::from(value)
}

impl Field {
    pub fn new(name: &str, kind: DataType) -> Self {
        Field {
            name: name.to_string(),
            kind,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn kind(&self) -> &DataType {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub columns: Vec<Field>,
}

impl Schema {
    pub fn new(fields:Vec<Field>) -> Self {
        Schema {
            columns: fields
        }
    }

    pub fn new_single(name:&str, kind:DataType) -> Self {
        let field = Field::new(name, kind);
        Self::new(vec![field])
    }

    pub fn scalar_field(kind:DataType) -> Self {
        Self::new_single("it", kind)
    }

    pub fn named(&self, name:&str) -> Option<(usize, &Field)> {
        self.columns
            .iter()
            .enumerate()
            .find(|&(_, field)| field.name == name)
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn as_slice(&self) -> Vec<&str> {
        self.columns.iter().map(|x| x.name.as_ref()).collect()
    }

    /// Helper for select/projection
    pub fn only(&self, names:Vec<&str>) -> Self {
        let mut fields = Vec::with_capacity(names.len());
        for name in names {
            let (_pos, f) = self.named(name).unwrap();
            fields.push(f.clone());
        }
        Schema::new(fields)
    }

    /// Helper for deselect/projection
    pub fn except(&self, remove:Vec<&str>) -> Self {
        let count = self.len() - remove.len();
        let mut fields = Vec::with_capacity(count);

        for field in self.columns.clone() {
            let name = &field.name[..];
            if !remove.contains(&name) {
                fields.push(field.clone());
            }
        }
        Schema::new(fields)
    }

    pub fn exist(&self, field:&str) -> bool {
        let mut find = self.columns.iter().filter(|x| x.name == field);

        find.next().is_some()
    }

    pub fn extend(&self, right:Schema) -> Self {
        let count = self.len() + right.len();
        let mut fields = Vec::with_capacity(count);
        let mut left = self.columns.clone();
        let mut _right = right.columns.clone();

        fields.append(&mut left);
        let mut cont = 0;
        //Avoid duplicated field names...
        for f in _right {
            if right.exist(&f.name) {
                let name = format!("{}_{}", f.name, cont);
                fields.push(Field::new(&name, f.kind));
                cont = cont + 1;
            } else {
                fields.push(f);
            }
        }

        Schema::new(fields)
    }

    pub fn is_equal(&self, to:Schema) -> bool {
        let mut left = self.columns.clone();
        let mut right = to.columns.clone();
        left.sort_by(|a, b| b.name.cmp(&a.name));
        right.sort_by(|a, b| b.name.cmp(&a.name));

        left == right
    }
}

impl Index<usize> for Schema {
    type Output = Field;

    fn index(&self, pos: usize) -> &Field {
        &self.columns[pos]
    }
}

impl Data {
    pub fn empty(kind:DataType) -> Self {
        Data {
            kind,
            len: 0,
            data: Rc::new([].to_vec()),
        }
    }

    pub fn new(of:Vec<Scalar>, kind:DataType) -> Self {
        Data {
            kind,
            len: of.len(),
            data: Rc::new(of),
        }
    }

    pub fn new_row(of:Vec<Scalar>) -> Self {
        Data::new(of, DataType::Tuple)
    }

    pub fn append(&self, of:Vec<Scalar>) -> Self {
        let mut data = Vec::with_capacity(self.len + of.len());
        let mut other= of;

        for item in self.data.iter() {
            data.push(item.clone());
        }

        data.append(&mut other);

        Data::new(data, self.kind.clone())
    }

    pub fn append_data(&self, of:RVec<Scalar>) -> Self {
        let mut data = Vec::with_capacity(self.len + of.len());
        let other= of;

        for item in self.data.iter() {
            data.push(item.clone());
        }

        for item in other.iter() {
            data.push(item.clone());
        }

        Data::new(data, self.kind.clone())
    }
}

impl From<i32> for Scalar {
    fn from(i: i32) -> Self {
        Scalar::I32(i)
    }
}

impl From<i64> for Scalar {
    fn from(i: i64) -> Self {
        Scalar::I64(i)
    }
}

impl From<bool> for Scalar {
    fn from(i: bool) -> Self {
        Scalar::Bool(i)
    }
}

impl From<BytesMut> for Scalar {
    fn from(i: BytesMut) -> Self {
        Scalar::UTF8(i)
    }
}

macro_rules! to_scalar {
    ($ARRAY:expr) => {
        $ARRAY.into_iter().map(|x| Scalar::from(x)).collect()
    }
}

macro_rules! to_data {
    ($ARRAY:expr, $TY:expr) => {
        Data::new(to_scalar!($ARRAY), $TY)
    }
}

impl From<i32> for Data {
    fn from(of: i32) -> Self {
        to_data!(vec![of], DataType::I32)
    }
}

impl From<Vec<i32>> for Data {
    fn from(of: Vec<i32>) -> Self {
        to_data!(of, DataType::I32)
    }
}

impl From<i64> for Data {
    fn from(of: i64) -> Self {
        to_data!(vec![of], DataType::I64)
    }
}

impl From<Vec<i64>> for Data {
    fn from(of: Vec<i64>) -> Self {
        to_data!(of, DataType::I64)
    }
}

impl From<bool> for Data {
    fn from(of: bool) -> Self {
        to_data!(vec![of], DataType::Bool)
    }
}

impl From<Vec<bool>> for Data {
    fn from(of: Vec<bool>) -> Self {
        to_data!(of, DataType::Bool)
    }
}

impl From<BytesMut> for Data {
    fn from(of: BytesMut) -> Self {
        to_data!(vec![of], DataType::UTF8)
    }
}

impl From<Vec<BytesMut>> for Data {
    fn from(of: Vec<BytesMut>) -> Self {
        to_data!(of, DataType::UTF8)
    }
}

//TODO: How deal with mutable relations?

/// The frame is the central storage unit, for data in columnar or row layout
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub layout  :Layout,
    pub len     :usize,
    pub names   :Schema,
    pub data    :RVec<Data>,
}

impl From<Scalar> for Frame {
    fn from(of: Scalar) -> Self {
        let dt = match of {
            Scalar::None => DataType::None,
            Scalar::Bool(_) => DataType::Bool,
            Scalar::I32(_) => DataType::I32,
            Scalar::I64(_) => DataType::I64,
            Scalar::UTF8(_) => DataType::UTF8,
            Scalar::Tuple(_) => DataType::Tuple,
        };
        Frame {
            layout: Layout::Scalar,
            len: 1,
            names: Schema::scalar_field(dt.clone()),
            data: vec!(to_data!(vec![of], dt.clone()).into()).into(),
        }
    }
}

impl From<i32> for Frame {
    fn from(of: i32) -> Self {
        Frame {
            layout: Layout::Scalar,
            len: 1,
            names: Schema::scalar_field(DataType::I32),
            data: vec!(to_data!(vec![of], DataType::I32).into()).into(),
        }
    }
}

use std::ops::{Add};
impl Add for Frame {
    type Output = Frame;

    fn add(self, other: Frame) -> Frame {
        match (self.layout, other.layout) {
            (Layout::Scalar, Layout::Scalar) =>
                (self.data[0].data[0].clone() + other.data[0].data[0].clone()).into(),
            p => panic!("Cannot add {:?}", p),
        }
    }
}

pub fn layout_of_data(of:&Data) -> Layout {
    match of.len {
        0 => Layout::Scalar,
        1 => Layout::Scalar,
        _ => {
            if of.kind == DataType::Tuple {
                Layout::Row
            } else {
                Layout::Col
            }
        }
    }
}

impl Frame {
    //TODO: Validate equal size of headers and columns here or in the parser?
    pub fn new(names:Schema, data:Vec<Data>) -> Self {
        assert_eq!(names.len(), data.len(), "Field count <> # of columns");

        let size = data.len();

        let layout =
            match size {
                0 => Layout::Scalar,
                _ => layout_of_data(&data[0])
            };

        let count =
            if size > 0 {
                match layout {
                    Layout::Row => {
                        //println!("{:?} : {}", &layout, size);
                        size
                    },
                    _ => {
                        //println!("{:?} : {}", &layout, data[0].len);
                        data[0].len
                    }
                }
            } else {
                0
            };

        Frame{
            len:count,
            layout,
            names,
            data:Rc::new(data),
        }
    }

    pub fn new_anon(data:Vec<Data>) -> Self {
        let mut names:Vec<Field> = Vec::with_capacity(data.len());
        for (pos, d) in data.iter().enumerate() {
            let name = format!("{}", pos);
            names.push(Field::new(&name, d.kind.clone()));
        }
        Frame::new(Schema::new(names), data)
    }

    pub fn empty(names:Schema) -> Self {
        Frame::new(names, [].to_vec())
    }

    pub fn row_data(of:&Data, pos:usize) -> Data {
        let mut rows = Vec::with_capacity(of.len);
        for col in of.data.iter() {
            rows.push(col.clone())
        }

        Data::new(rows, DataType::Tuple)
    }

    pub fn row(of:&Frame, pos:usize) -> Data {
        let mut rows = Vec::with_capacity(of.len);
        for col in of.data.iter() {
            rows.push(col.data[pos].clone())
        }

        Data::new(rows, DataType::Tuple)
    }

    //TODO: Remove this hack, and put type on field name
    pub fn col(of:&Frame, pos:usize) -> Data {
        let mut rows = Vec::with_capacity(of.len);
        let mut last = DataType::None;

        for col in of.data.iter() {
            last = col.kind.clone();
            rows.push(col.data[pos].clone())
        }

        Data::new(rows, last)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scalar::None    => write!(f, "None"),
            Scalar::Bool(x) => write!(f, "{}", x),
            Scalar::I32(x)  => write!(f, "{}", x),
            Scalar::I64(x)  => write!(f, "{}", x),
            Scalar::UTF8(x) => write!(f, "{:?}", x),
            Scalar::Tuple(x)=> write!(f, "{:?}", x),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.kind)
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[it:{}; ", self.kind)?;
        for i in 0..self.len {
            let item =  &self.data[i];
            if i > 0 {
                write!(f, ", {}",item)?;
            } else {
                write!(f, "{}", item)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[< ")?;

        for (pos, field) in self.names.columns.iter().enumerate() {
            writeln!(f, "{} = {};", field, self.data[pos])?;
        }
        writeln!(f, ">]")?;
        Ok(())
    }
}