#![allow(dead_code)]
#![allow(unused_imports)]
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::Cell;
use std::ops::Index;

extern crate bytes;

use self::bytes::*;
//use super::cursor::Cursor;

pub type RVec<T> = Rc<Vec<T>>;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Scalar {
	None, //null
    Bool(bool),
    I32(i32),
    I64(i64),
    UTF8(BytesMut),
    Tuple(Vec<Scalar>),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub kind:   DataType,
    pub len:    usize,
    pub data:   RVec<Scalar>,
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

#[derive(Debug, Clone)]
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

    pub fn named(&self, name:&str) -> Option<(usize, &Field)> {
        self.columns
            .iter()
            .enumerate()
            .find(|&(_, field)| field.name == name)
    }

    pub fn len(&self) -> usize {
        self.columns.len()
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

macro_rules! to_data {
    ($ARRAY:expr, $TY:expr) => {
        Data::new($ARRAY.into_iter().map(|x| Scalar::from(x)).collect(), $TY)
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
#[derive(Debug, Clone)]
pub struct Frame {
    pub layout  :Layout,
    pub len     :usize,
    pub names   :Schema,
    pub data    :RVec<Data>,
}

fn layout_of_data(of:&Data) -> Layout {
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
        let mut names:Vec<Field> = Vec::new();
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
        let mut rows = Vec::new();
        for col in of.data.iter() {
            rows.push(col.clone())
        }

        Data::new(rows, DataType::Tuple)
    }

    pub fn row(of:&Frame, pos:usize) -> Data {
        let mut rows = Vec::new();
        for col in of.data.iter() {
            rows.push(col.data[pos].clone())
        }

        Data::new(rows, DataType::Tuple)
    }

    //TODO: Remove this hack, and put type on field name
    pub fn col(of:&Frame, pos:usize) -> Data {
        let mut rows = Vec::new();
        let mut last = DataType::None;

        for col in of.data.iter() {
            last = col.kind.clone();
            rows.push(col.data[pos].clone())
        }

        Data::new(rows, last)
    }

}

pub trait Relation {
    fn layout(&self) -> Layout;
    fn col_count(&self) -> usize;
    fn row_count(&self) -> usize;
    fn names(&self)  -> Schema;
    fn row(&self, pos:usize) -> Data;
    fn col(&self, pos:usize) -> Data;
    fn value(&self, row:usize, col:usize) -> &Scalar;
    fn resolve_names(&self, of: Vec<&ColumnExp>) -> Schema {
        let mut names = Vec::new();
        let fields = self.names();

        for name in of.into_iter() {
            let pick =
                    match name {
                        ColumnExp::Pos(x) => {
                            fields[*x].clone()
                        },
                        ColumnExp::Name(x) => {
                            let (_pos, f) = fields.named(x).unwrap();
                            f.clone()
                        }
                    };
            names.push(pick);
        }
        Schema::new(names)
    }

    fn get_col(&self, name:&String) -> Data
    {
        let fields = self.names();
        let (pos, _f) = fields.named(name).unwrap();
        self.col(pos)
    }
}

/// Encapsulate 2d relations (aka: Tables)
impl Relation for Frame {
    fn layout(&self) -> Layout {
        self.layout.clone()
    }
    fn col_count(&self) -> usize {
        self.names.len()
    }
    fn row_count(&self) -> usize {
        self.len
    }
    fn names(&self) -> Schema {
        self.names.clone()
    }
    fn row(&self, pos:usize) -> Data {
        Frame::row(&self, pos)
    }

    fn col(&self, pos:usize) -> Data {
        match self.layout {
            Layout::Row => {
                Frame::col(self, pos)
            },
            _ => self.data[pos].clone(),
        }
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        &self.data[row].data[col]
    }
}

/// Encapsulate 1d relations (aka: arrays)
impl Relation for Data {
    fn layout(&self) -> Layout {
        Layout::Col
    }
    fn col_count(&self) -> usize {
        1
    }
    fn row_count(&self) -> usize {
        self.len
    }
    fn names(&self) -> Schema
    {
        Schema::new(vec![Field::new("item", self.kind.clone())])
    }
    fn row(&self, pos:usize) -> Data {
        Frame::row_data(&self, pos)
    }

    fn col(&self, pos:usize) -> Data {
        self.clone()
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        &self.data[row]
    }
}

struct DataSource<T>
where T:Relation
{
    source:T,
    pos: Cell<usize>,

}

impl <T> DataSource<T>
    where T:Relation
{
    fn new(source: T) -> Self {
        DataSource {
            pos: Cell::new(0),
            source
        }
    }

    fn _set(&self, pos:usize) {
        self.pos.set(pos)
    }
    fn pos(&self) -> usize {
        self.pos.get()
    }

    fn len(&self) -> usize {
        self.source.row_count()
    }

    fn eof(&self) -> bool {
        self.pos() == self.len()
    }
    fn first(&self) {
        self._set(0)
    }
    fn back(&self) -> bool {
        self.skip(-1)
    }
    fn next(&self) -> bool {
        self.skip(1)
    }
    fn last(&self) {
        let pos = self.len();
        self._set(pos)
    }
    fn skip(&self, steps:isize) -> bool {
        let pos = (self.pos() as isize) + steps;

        if pos < 0 || pos > (self.len() as isize) {
            return false
        }
        self._set(pos as usize);
        true
    }

    /// Find the first value that match
    fn filter(&self, col:usize, value:&Scalar) -> bool {
        while !self.eof() {
            let lv = self.source.value(self.pos(), col);
            if lv == value {
                return true
            }
            self.next();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _name(name:&str, kind:DataType) -> Schema {
        Schema::new_single(name, kind)
    }

    fn _name2(name:&str, name2:&str, kind:DataType) -> Schema {
        let fields = vec![Field::new(name, kind.clone()), Field::new(name2, kind.clone())];
        Schema::new(fields)
    }

    #[test]
    fn test_create_frame() {
        let null1 = Data::empty(DataType::I32);
        let s1 = Data::from(1);
        let col1 = Data::from(vec![1, 2, 3]);
        let col2 = col1.clone();
        let row1 = to_data!(vec![3, 4, 5], DataType::Tuple);
        let row2 = row1.clone();

        let name = _name("x", DataType::I32);
        let names = _name2("x", "y", DataType::I32);

        let fnull = Frame::new(name.clone(), vec![null1]);
        assert_eq!(fnull.layout, Layout::Scalar);
        assert_eq!(fnull.col_count(), 1);
        assert_eq!(fnull.row_count(), 0);

        let fs1 = Frame::new(name.clone(), vec![s1]);
        assert_eq!(fs1.layout, Layout::Scalar);
        assert_eq!(fs1.col_count(), 1);
        assert_eq!(fs1.row_count(), 1);

        let fcol1 = Frame::new(name.clone(), vec![col1.clone()]);
        assert_eq!(fcol1.layout, Layout::Col);
        assert_eq!(fcol1.row_count(), 3);

        let fcols = Frame::new(names.clone(), vec![col1, col2]);
        assert_eq!(fcols.layout, Layout::Col);
        assert_eq!(fcols.col_count(), 2);
        assert_eq!(fcols.row_count(), 3);

        let frow1 = Frame::new(name.clone(), vec![row1.clone()]);
        assert_eq!(frow1.layout, Layout::Row);
        assert_eq!(frow1.col_count(), 1);
        assert_eq!(frow1.row_count(), 1);

        let frows = Frame::new(names.clone(), vec![row1, row2]);
        assert_eq!(frows.layout, Layout::Row);
        assert_eq!(frows.col_count(), 2);
        assert_eq!(frows.row_count(), 2);

        //TODO: What type is a empty frame?
//        let fempty = Frame::empty(names.clone());
//        assert_eq!(fempty.layout, Layout::Row);
//        assert_eq!(fempty.col_count(), 2);
//        assert_eq!(fempty.row_count(), 0);

    }

    #[test]
    fn test_create_col() {
        let null1 = Data::empty(DataType::I32);
        assert_eq!(layout_of_data(&null1), Layout::Scalar);

        let s1 = Data::from(1);
        assert_eq!(layout_of_data(&s1), Layout::Scalar);

        let col1 = Data::from(vec![1, 2, 3]);
        assert_eq!(layout_of_data(&col1), Layout::Col);

        let row1 = to_data!(vec![3, 4, 5], DataType::Tuple);
        assert_eq!(layout_of_data(&row1), Layout::Row);
    }
}