#![allow(dead_code)]
#![allow(unused_variables)]
//#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::cell::Cell;
use std::rc::Rc;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::marker::PhantomData;
use std::fmt;
use std::fmt::Debug;

extern crate bit_vec;
use self::bit_vec::BitVec;

//TODO: https://deterministic.space/elegant-apis-in-rust.html
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add, Minus, Mul, Div
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogicOp {
    And, Or, Not
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CompareOp {
    Eq, NotEq, Less, LessEq, Greater, GreaterEq
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrudOp {
    Create, Update, Delete
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexOp {
    Pos, Name,
}

pub enum KeyValue { Key, Value }

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
    Rows  // Planed: BitVec, Blob, Sum(DataType), Product(DataType), Rel(Vec<Field>)
}

//NOTE: The order of this enum must match DataType
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    None, //null
    Bool(bool),
    //Numeric
    I32(i32),
    ISize(isize),
    I64(i64),
    UTF8(String),
    //F64(N64),
    //Dec(Decimal),
    //Complex
    Rows(Box<Table>),
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
            Scalar::Rows(_) => DataType::Rows,
        }
    }
}

//Type Alias...
pub type BoolExpr = dyn Fn(&Scalar, &Scalar) -> bool;
pub type BinExpr = dyn Fn(&Scalar, &Scalar) -> Scalar;
pub type UnaryExpr = dyn Fn(&Scalar) -> Scalar;
pub type Col = Vec<Scalar>;
pub type Pos = Vec<usize>;
pub type Tree = BTreeMap<Scalar, Scalar>;
pub type RScalar = Rc<Scalar>;

pub type Phantom<'a> = PhantomData<&'a Scalar>;
pub type PhantomMut<'a> = PhantomData<&'a mut Scalar>;

pub trait Buffered {
    fn buffer(&mut self) -> &mut [Scalar];

    fn fill(&mut self, data:&[&Scalar]) {
        let buffer = self.buffer();
        for i in 0..data.len() {
            buffer[i] = data[i].clone();
        }
    }

    fn read_from_buffer(&mut self, pos:usize) -> Option<&Scalar>;
}

pub trait RelOp {
    fn exec(&mut self) -> Option<Col>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmOp {
    pub op:  CompareOp,
    pub lhs: usize,
    pub rhs: Rc<Scalar>
}

impl CmOp  {
    fn new(op:CompareOp, lhs: usize, rhs: Rc<Scalar>) -> Self {
        CmOp {op, lhs, rhs}
    }

    pub fn eq(lhs: usize, rhs: Rc<Scalar>) -> Self { Self::new(CompareOp::Eq, lhs, rhs) }
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
    pub fn greater_eq(lhs: usize, rhs: Rc<Scalar>) -> Self { Self::new(CompareOp::GreaterEq, lhs, rhs) }

    pub fn get_fn(&self) -> &BoolExpr {
        match self.op {
            CompareOp::Eq       => &PartialEq::eq,
            CompareOp::NotEq    => &PartialEq::ne,
            CompareOp::Less     => &PartialOrd::lt,
            CompareOp::LessEq   => &PartialOrd::le,
            CompareOp::Greater  => &PartialOrd::gt,
            CompareOp::GreaterEq=> &PartialOrd::ge,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SetQuery
{
    Union,
    Diff,
    Intersection,
    Join(Join, Pos)
}

#[derive(Debug, Clone)]
pub enum Query {
//  To ask for all the rows, send a empty query
    Distinct,
    Where(CmOp),
    Limit(usize, usize),// skip * limit
    Sort(bool, usize),  // true=ascending * col pos
    Select(Pos),
    Project(Col, String),
//    Project(Box<UnaryExpr>),
    Group(Pos),
}

impl Query  {
    pub fn eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::eq(lhs,rhs.into()))
    }

    pub fn not(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::not(lhs,rhs.into()))
    }

    pub fn less(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::less(lhs,rhs.into()))
    }

    pub fn less_eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::less_eq(lhs,rhs.into()))
    }

    pub fn greater(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::greater(lhs,rhs.into()))
    }

    pub fn greater_eq(lhs: usize, rhs: Scalar) -> Self {
        Query::Where(CmOp::greater_eq(lhs,rhs.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Generator<'a> {
    pub name: &'a str,
    pub cursor: Cursor,
    pub data:Box<&'a RelOp>
}

//#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
//pub enum RScalar<'a> {
//    Rows(Table),
//    BTree(BTree),
//    Range(Range),
//    Iter(Generator<'a>),
//}
//
//#[derive(Debug, Clone, PartialEq, PartialOrd)]
//pub struct Rel<'a> {
//    pub schema:Schema,
//    pub data:RScalar<'a>
//}

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

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct Schema {
    pub columns: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Vector {
    pub data: Col,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Range {
    pub start:isize,
    pub end:  isize,
    pub step: isize,
    pub buffer: Col,
}

//#[derive(Debug, Clone, PartialEq, PartialOrd)]
//pub struct Dict {
//    index: HashMap<Scalar, Col>,
//}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct BTree {
    pub schema: Schema,
    pub data:   BTreeMap<Scalar, Scalar>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct  Table {
    pub schema: Schema,
    pub count:usize,
    pub data: Vec<Col>,
}

#[derive(Debug)]
pub struct ColIter<'a, R> {
    pub pos: usize,
    pub col: usize,
    pub rel: &'a R
}

#[derive(Debug)]
pub struct RelIter<'a, R> {
    pub pos: usize,
    pub rel: &'a R
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cursor
{
    pub start: usize,
    pub last: usize,
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
pub fn size_rel(of:&[Col]) -> (usize, usize) {
    let rows = of.len();
    if rows > 0 {
        (rows, of[0].len())
    } else {
        (0, 0)
    }
}

#[inline]
pub fn cmp(of:&CompareOp, lhs:&Scalar, rhs:&Scalar) -> bool {
    match of {
        CompareOp::Eq       => lhs == rhs,
        CompareOp::NotEq    => lhs != rhs,
        CompareOp::Less     => lhs < rhs,
        CompareOp::LessEq   => lhs <= rhs,
        CompareOp::Greater  => lhs > rhs,
        CompareOp::GreaterEq=> lhs >= rhs,
    }
}

#[inline]
pub fn assert_schema(from:&Schema, to:&Schema) {
    assert_eq!(from, to, "The schemas must be equal");
}

fn _bitvector_count(of:&BitVec) -> (usize, usize) {
    let trues = of.iter().filter(|x| *x).count();

    (trues, of.len() - trues)
}

pub fn _bitvector_to_pos(of:&BitVec) -> Vec<isize> {
    let mut pos =  vec![-1isize; of.len()];

    for (i, found) in of.iter().enumerate() {
        if found {
            pos[i] = i as isize;
        }
    }
    pos
}

fn hash_rows<T:Relation>(of:&T) -> HashMap<u64, usize> {
    let mut rows = HashMap::with_capacity(of.row_count());

    for (i, row) in of.rows_iter().enumerate() {
        rows.insert(hash_column(&row), i);
    }

    rows
}

pub fn compare_hash<T, U>(left:&T, right:&U, mark_found:bool) -> (BitVec, usize)
    where
        T: Relation,
        U: Relation
{
    fn _check_not_found(cmp:&HashMap<u64, usize>, row:u64) -> bool {
        !cmp.contains_key(&row)
    }

    fn _check_found(cmp:&HashMap<u64, usize>, row:u64) -> bool {
        cmp.contains_key(&row)
    }

    let cmp = hash_rows(left);
    let mut results = BitVec::from_elem(right.row_count(), false);
    let mut not_found = 0;
    let check =
        if mark_found {
            _check_found
        }  else {
            _check_not_found
        };

    for (next, row) in right.rows_iter().enumerate() {
        let h = hash_column(&row);

        if check(&cmp, h) {
            results.set(next, true);
        } else {
            not_found += 1;
        }
    }

    (results, not_found)
}

fn query_iter(input: Box<dyn Iterator<Item=Col>>, query:&'static [Query]) -> Box<dyn Iterator<Item=Col>> {
    let mut result = input;
    for q in query {
        result =
            match q {
                Query::Where(value) => {
                    let lhs = value.rhs.as_ref();
                    let f =result.filter(move |x| cmp(&value.op, lhs, &x[value.lhs] ));
                    Box::new(f)
                },
                Query::Sort(asc, pos) => {
                    let mut r: Vec<_> = result.collect();
                    r.sort();
                    Box::new(r.into_iter())
                },
                _ => unimplemented!()
            };
    }

    result
}

pub trait Relation:Sized + fmt::Display + fmt::Debug + std::cmp::PartialEq {
    fn type_name<'a>() -> &'a str;
    fn new_from<R: Relation>(names: Schema, of: &R) -> Self;
    fn from_vector(schema:Schema, vector:Vec<Col>) -> Self;
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

    fn get_row(&self, pos:usize) -> Option<Col>;
    fn rows_iter(&self) -> RelIter<'_, Self>;
    fn col_iter(&self, col: usize) -> ColIter<'_, Self>;
    fn col(&self, col: usize) -> Col;
    fn rows_pos(&self, pick: Pos) -> Self;

    /// Relational operators
    fn query(self, query:&[Query]) -> Self {
        if query.len() > 0 {
            let mut next= self;
            for q in query {
                next =
                    match q {
                        Query::Where(filter) => {
                            next.filter(filter)
                        },
                        Query::Sort(asc, pos) => {
                            next.sorted(*asc, *pos)
                        },
                        _ => unimplemented!()
                    };
            };
            next
        } else {
            self
        }
    }

    fn filter(self, query:&CmOp) -> Self;
    fn sorted(self, asc:bool, pos:usize) -> Self;

    fn union(self, other:Self) -> Self;

//    fn cmp(&self, row: usize, col: usize, value: &Scalar, apply: &BoolExpr) -> bool
//    {
//        let old = self.value(row, col);
//        //println!("CMP {:?}, {:?}", value, old);
//        apply(old, value)
//    }
//
//    fn cmp_cols(&self, row: usize, cols: &[usize], tuple: &[Scalar], apply: &BoolExpr) -> bool
//    {
//        let values = cols.iter().zip(tuple.iter());
//
//        for (col, value) in values {
//            let old = self.value(row, *col);
//            if !apply(old, value) {
//                return false;
//            }
//        }
//        true
//    }
//
//    fn find(&self, cursor:&mut Cursor, col:usize, value:&Scalar, apply: &BoolExpr ) -> Option<usize>
//    {
//        //println!("FIND {:?}, {:?}", cursor.start, cursor.last);
//        while !cursor.eof() {
//            let row = cursor.start;
//            cursor.next();
//            if self.cmp(row, col, value, apply) {
//                return Some(row)
//            }
//        }
//
//        Option::None
//    }
//
//    fn row_only(&self, row: usize, cols: &[usize]) -> Col {
//        let mut data = Vec::with_capacity(cols.len());
//
//        for i in cols {
//            data.push(self.value(row, *i).clone())
//        }
//        data
//    }
//
//    fn materialize_raw(&self, pos:&BitVec, null_count:usize, keep_null:bool) -> Col {
//        let rows = pos.len();
//        let cols = self.col_count();
//        let total_rows = if keep_null {rows} else {rows - null_count};
//
//        let mut data = vec![Scalar::None; cols * total_rows];
//        println!("Raw r:{:?}", pos);
//
//        let positions:Vec<(usize, bool)> =  pos.iter()
//            .enumerate()
//            .filter(|(_, x)| *x || keep_null).collect();
//        println!("Raw r:{:?}", positions);
//
//        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());
//
//        for (new_row, (row, found)) in positions.into_iter().enumerate() {
//            for col in 0..cols {
//                let _pos = index( cols, total_rows, new_row, col);
//                if found {
//                    data[_pos] = self.value(row, col).clone();
//                }
//            }
//        }
//
//        data
//    }
//
//    fn materialize_data(&self, pos:&BitVec, keep_null:bool) -> Table {
//        let rows = pos.len();
//        let cols = self.col_count();
//        let positions:Vec<(usize, bool)> =  pos.iter()
//            .enumerate()
//            .filter(|(_, x)| *x || keep_null).collect();
//        println!("Raw rpos:{:?}", positions);
//
//        let total_rows = if keep_null {rows} else { positions.len()};
//
//        let mut data = vec![Scalar::None; cols * total_rows];
//        println!("Raw r:{:?}", pos);
//
//        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());
//
//        for (new_row, (row, found)) in positions.into_iter().enumerate() {
//            for col in 0..cols {
//                if found {
//                    let _pos = index(cols, total_rows, new_row, col);
//                    data[_pos] = self.value(row, col).clone();
//                }
//            }
//        }
//
//        Table::new(total_rows, cols, data)
//    }
//
//    fn find_all(&self, start:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<usize>
//    {
//        let mut pos = Vec::new();
//
//        let mut cursor = Cursor::new(start, self.row_count());
//
//        while let Some(next) = self.find(&mut cursor, col, value, apply) {
//            pos.push(next);
//        }
//
//        pos
//    }
//    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Self;
//
//
//    fn union<T:Relation>(&self, to:&T) -> Self;
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
            Scalar::Rows(x) => write!(f, "{:?}", x),
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
