#![allow(dead_code)]
#![allow(unused_variables)]
//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::marker::PhantomData;
use std::fmt;
use std::fmt::Debug;

extern crate bit_vec;
use self::bit_vec::BitVec;

//use decorum::N64;
#[derive(Debug, Clone, Copy)]
pub enum Join { Left, Right, Inner, Full //, Natural, Cross
}

impl Join {
    pub fn produce_null(self, is_left:bool) -> bool
    {
        match self {
            Join::Left  => !is_left,
            Join::Right => is_left,
            Join::Inner => false,
            Join::Full  => true,
        }
    }
}

//NOTE: This define a total order, so it matter what is the order
//of the enum! The overall sorting order is defined as:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataType {
    None, Bool,
    //Numeric
    I32, ISIZE, I64, // Planed: F64, Decimal,
    //Dates
    //Time, Date, DateTime,
    //Text
    UTF8,
    //Complex
    Tuple, // Planed: BitVec, Blob, Sum(DataType), Product(DataType), Rel(Vec<Field>)
}

//Type Alias...
pub type BoolExpr = dyn Fn(&Scalar, &Scalar) -> bool;
pub type BinExpr = dyn Fn(&Scalar, &Scalar) -> Scalar;
pub type UnaryExpr = dyn Fn(&Scalar) -> Scalar;
pub type Col = Vec<Scalar>;
pub type Pos = Vec<usize>;
pub type Tree = BTreeMap<Scalar, Scalar>;

pub type Phantom<'a> = PhantomData<&'a Scalar>;
pub type PhantomMut<'a> = PhantomData<&'a mut Scalar>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add, Minus, Mul, Div
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CompareOp {
    Eq, NotEq, Less, LessEq, Greater, GreaterEq
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RelOp {
    Union, Diff
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrudOp {
    Create, Update, Delete
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexOp {
    Pos, Name,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    None, //null
    Bool(bool),
    ISize(isize),
    I32(i32),
    I64(i64),
    UTF8(String),
    //F64(N64),
    //Dec(Decimal),
    //Rows(Box<Data>),
}

impl Default for Scalar {
    fn default() -> Scalar { Scalar::None }
}

impl Scalar {
    pub fn repeat(of:&Scalar, times:usize) -> Vec<Scalar> {
        vec![of.clone(); times]
    }

    pub fn kind(self:&Scalar) -> DataType {
        match self {
            Scalar::None    => DataType::None,
            Scalar::Bool(_) => DataType::Bool,
            Scalar::ISize(_)=> DataType::ISIZE,
            Scalar::I32(_)  => DataType::I32,
            Scalar::I64(_)  => DataType::I64,
            Scalar::UTF8(_) => DataType::UTF8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Union {
    tag:i16,
    data:Col
}

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

#[derive(Debug, Clone, PartialOrd)]
pub struct Schema {
    pub columns: Vec<Field>,
}

/// The `NDArray` struct, for storing relational/table data (2d)
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NDArray {
    pub rows: usize,
    pub cols: usize,
    pub data: Col,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Index {
    data: BTreeMap<Scalar, usize>
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Range {
    pub start:isize,
    pub end:  isize,
    pub step: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Row<T:Relation>  {
    pub source: T,
}

//#[derive(Debug, Clone, PartialEq, PartialOrd)]
//pub struct Dict {
//    index: HashMap<Scalar, Col>,
//}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BTree {
    pub schema: Schema,
    pub data:  BTreeMap<Scalar, Scalar>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Data {
    pub schema: Schema,
    pub data: NDArray,
}

#[derive(Debug, Clone)]
pub struct DataSource<T> {
    pub schema: Schema,
    pos  :usize,
    batch:usize,
    pub source: T,
}

#[derive(Debug)]
pub struct ColIter<'a, R> {
    pub pos: usize,
    pub col: usize,
    pub rel: &'a R
}

/// Scalar iterator.
#[derive(Debug)]
pub struct RelIter<'a, R> {
    pub pos: usize,
    pub rel: &'a R
}

pub struct Cursor
{
    start: usize,
    last: usize,
}

impl Cursor
{
    pub fn new(start:usize, last:usize) -> Self {
        Cursor {
            last,
            start
        }
    }

    pub fn set(&mut self, pos:usize) {
        self.start = pos;
    }

    pub fn next(&mut self) {
        let pos = self.start;
        self.set(pos + 1)
    }

    pub fn eof(&self) -> bool {
        self.start >= self.last
    }
}


#[inline]
fn size_rel(of:&[Col]) -> (usize, usize) {
    let rows = of.len();
    if rows > 0 {
        (of[0].len(), rows)
    } else {
        (0, 0)
    }
}

/// Calculate the appropriated index in the flat array
#[inline]
pub fn index(col_count:usize, row_count:usize, row:usize, col:usize) -> usize {
    //println!("pos {:?} Row:{}, Col:{}, R:{}, C:{}", layout, row, col, row_count , col_count);
    row * col_count + col
}

#[inline]
pub fn write_row(to:&mut Col, col_count:usize, row_count:usize, row:usize, data:Col) {
    for (col, value) in data.into_iter().enumerate() {
        let index = index(col_count, row_count, row, col);
        to[index] = value;
    }
}

pub trait Relation:Sized + fmt::Display {
    fn type_name<'a>() -> &'a str;
    fn new_from<R: Relation>(names: Schema, of: &R) -> Self;
    fn from_vector(schema:Schema, rows:usize, cols:usize, vector:Col) -> Self;
    fn to_ndarray(&self) -> NDArray;
    fn flat_raw(&self) -> Col {
        let rows = self.row_count();
        let cols = self.col_count();

        let mut data = Vec::with_capacity(cols * rows);

        for row in 0..rows {
            for col in 0..cols {
                data.push(self.value(row, col).clone())
            }
        }

        data
    }

    fn clone_schema(&self, schema:&Schema) -> Self;
    fn schema(&self) -> &Schema;

    fn len(&self) -> usize {self.row_count() * self.col_count()}
    fn row_count(&self) -> usize;
    fn col_count(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0}

    fn value(&self, row:usize, col:usize) -> &Scalar;
    fn get_value(&self, row:usize, col:usize) -> Option<&Scalar>;

    //fn pk(&self) -> Option<usize>;
    fn get_row(&self, pos:usize) -> Option<Col>;
    fn rows_iter(&self) -> RelIter<'_, Self>;
    fn col_iter(&self, col: usize) -> ColIter<'_, Self>;
    fn col(&self, col: usize) -> Col;
    fn rows_pos(&self, pick: Pos) -> Self;

    fn hash_rows(&self) -> HashMap<u64, usize> {
        let mut rows = HashMap::with_capacity(self.row_count());

        for (i, row) in self.rows_iter().enumerate() {
            rows.insert(hash_column(&row), i);
        }

        rows
    }

    fn cmp(&self, row: usize, col: usize, value: &Scalar, apply: &BoolExpr) -> bool
    {
        let old = self.value(row, col);
        //println!("CMP {:?}, {:?}", value, old);
        apply(old, value)
    }

    fn cmp_cols(&self, row: usize, cols: &[usize], tuple: &[Scalar], apply: &BoolExpr) -> bool
    {
        let values = cols.iter().zip(tuple.iter());

        for (col, value) in values {
            let old = self.value(row, *col);
            if !apply(old, value) {
                return false;
            }
        }
        true
    }

    fn find(&self, cursor:&mut Cursor, col:usize, value:&Scalar, apply: &BoolExpr ) -> Option<usize>
    {
        //println!("FIND {:?}, {:?}", cursor.start, cursor.last);
        while !cursor.eof() {
            let row = cursor.start;
            cursor.next();
            if self.cmp(row, col, value, apply) {
                return Some(row)
            }
        }

        Option::None
    }

    fn row_only(&self, row: usize, cols: &[usize]) -> Col {
        let mut data = Vec::with_capacity(cols.len());

        for i in cols {
            data.push(self.value(row, *i).clone())
        }
        data
    }

    fn materialize_raw(&self, pos:&BitVec, null_count:usize, keep_null:bool) -> Col {
        let rows = pos.len();
        let cols = self.col_count();
        let total_rows = if keep_null {rows} else {rows - null_count};

        let mut data = vec![Scalar::None; cols * total_rows];
        println!("Raw r:{:?}", pos);

        let positions:Vec<(usize, bool)> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x || keep_null).collect();
        println!("Raw r:{:?}", positions);

        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());

        for (new_row, (row, found)) in positions.into_iter().enumerate() {
            for col in 0..cols {
                let _pos = index( cols, total_rows, new_row, col);
                if found {
                    data[_pos] = self.value(row, col).clone();
                }
            }
        }

        data
    }

    fn materialize_data(&self, pos:&BitVec, keep_null:bool) -> NDArray {
        let rows = pos.len();
        let cols = self.col_count();
        let positions:Vec<(usize, bool)> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x || keep_null).collect();
        println!("Raw rpos:{:?}", positions);

        let total_rows = if keep_null {rows} else { positions.len()};

        let mut data = vec![Scalar::None; cols * total_rows];
        println!("Raw r:{:?}", pos);

        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());

        for (new_row, (row, found)) in positions.into_iter().enumerate() {
            for col in 0..cols {
                if found {
                    let _pos = index(cols, total_rows, new_row, col);
                    data[_pos] = self.value(row, col).clone();
                }
            }
        }

        NDArray::new(total_rows, cols, data)
    }

    fn find_all(&self, start:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<usize>
    {
        let mut pos = Vec::new();

        let mut cursor = Cursor::new(start, self.row_count());

        while let Some(next) = self.find(&mut cursor, col, value, apply) {
            pos.push(next);
        }

        pos
    }
    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Self;


    fn union<T:Relation>(&self, to:&T) -> Self;
}

pub trait RelationMut:Sized {

}

impl<'a, T:Relation> Iterator for RelIter<'a, T>
{
    type Item = Col;

    fn next (&mut self) -> Option<Self::Item> {
        let next = self.rel.get_row(self.pos);

        if next.is_some() {
            self.pos += 1;
            next
        } else {
            None
        }
    }
}

impl<'a, T:Relation> Iterator for ColIter<'a, T>
{
    type Item = &'a Scalar;

    fn next (&mut self) -> Option<Self::Item> {
        self.rel.get_value(self.pos, self.col)
    }
}

/// Auxiliary functions and shortcuts
pub fn hash_column(vec: &[Scalar]) -> u64 {
    //println!("HASH {:?}", vec);
    let mut hasher = DefaultHasher::new();

    vec.into_iter().for_each(| x | x.hash(&mut hasher));

    hasher.finish()
    //println!("HASH {:?}",x);    
}

/// Pretty printers..
fn _print_rows(of: &[Scalar], f: &mut fmt::Formatter) -> fmt::Result {
    for (i, value) in of.iter().enumerate() {
        if i == of.len() - 1{
            write!(f, "{}", value)?;
        } else {
            write!(f, "{}, ", value)?;
        }
    }
    Ok(())
}

pub fn print_rows<T:Relation>(kind:&str, of: &T, f: &mut fmt::Formatter) -> fmt::Result {
    let (sep1, sep2) =  ("[<", ">]");

    write!(f, "{}{}", kind, sep1)?;
    if of.col_count() > 0 {
        write!(f, "{}", of.schema())?;
        writeln!(f, ";")?;
        let rows = of.rows_iter();

        for (pos, row) in rows.enumerate() {
            _print_rows(&row, f)?;
            if pos < of.row_count() - 1 {
                writeln!(f, ";")?;
            }
        }

    }
    writeln!(f, " {}", sep2)?;
    Ok(())
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scalar::None =>  write!(f, "None"),
            Scalar::Bool(x) => write!(f, "{}", x),
            Scalar::ISize(x) => write!(f, "{}", x),
            Scalar::I32(x) => write!(f, "{}", x),
            Scalar::I64(x) => write!(f, "{}", x),
            Scalar::UTF8(x) => write!(f, "{}", x),
//            Scalar::Tuple(x) => write!(f, "{:?}", x),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.kind)
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.len() {
            let item =  &self.columns[i];
            if i > 0 {
                write!(f, ", {}",item)?;
            } else {
                write!(f, "{}", item)?;
            }
        }

        Ok(())
    }
}
