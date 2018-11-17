use std::fmt;

use std::fmt::Debug;

/// Marker trait for the values
trait Value: Clone + Debug {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum Layout {
    Row,
    Col,
}

#[derive(Debug, Clone)]
pub enum Operator {
    //Compare
    Eq, NotEq, Less, LessEq, Greater, GreaterEq, Not,
    //Math
    Add, Minus, Mul, Div,
    //Relational
    Union, Diff,
    //Utils
    IndexByPos, IndexByName,
}

//NOTE: This define a total order, so it matter what is the order
//of the enum! The overall sorting order is defined as:
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum DataType {
    None,
    Bool,
    //Numeric
    I32, I64,
    //    Planed:
//    F32,
//    F64,
//    Decimal,
//    Time,
//    Date,
//    DateTime,
//    Char,
    //Text
    UTF8,
    //    Byte,
    //Complex
    Tuple,
//    Sum(DataType), //enums
//    Product(DataType), //struct
//    Rel(Vec<Field>), //Relations, Columns
//    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scalar {
    None, //null
    Bool(bool),
    I32(i32),
    I64(i64),
    UTF8(String),
    Tuple(Vec<Scalar>),
}

impl Default for Scalar {
    fn default() -> Scalar { Scalar::None }
}

impl Scalar {
    pub fn repeat(of:&Scalar, times:usize) -> Vec<Scalar> {
        let mut result = Vec::with_capacity(times);
        for _i in 0..times {
            result.push(of.clone());
        }
        result
    }

    pub fn kind(self:&Scalar) -> DataType {
        match self {
            Scalar::None => DataType::None,
            Scalar::Bool(_) => DataType::Bool,
            Scalar::I32(_) => DataType::I32,
            Scalar::I64(_) => DataType::I64,
            Scalar::UTF8(_) => DataType::UTF8,
            Scalar::Tuple(_) => DataType::Tuple,
        }
    }
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
pub enum ColumnName {
    Name(String),
    Pos(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    pub name: String,
    pub kind: DataType,
}

pub type Col = Vec<Scalar>;
pub type Pos = Vec<usize>;

pub type BoolExpr = Fn(&Scalar, &Scalar) -> bool;
pub type BinExpr = Fn(&Scalar, &Scalar) -> Scalar;
pub type UnaryExpr = Fn(&Scalar) -> Scalar;

//TODO: Must implement Eq/Ord/Hash manually.
#[derive(Debug, Clone, Eq, PartialOrd, Hash)]
pub struct Schema {
    pub columns: Vec<Field>,
}

macro_rules! convert {
    ($kind:ident, $bound:path) => (
        impl <'a> From<&'a $kind> for Scalar {
            fn from(i: &'a $kind) -> Self {
                $bound(*i)
            }
        }

        impl From<$kind> for Scalar {
            fn from(i: $kind) -> Self {
                $bound(i)
            }
        }
        impl From<Scalar> for $kind {
            fn from(i: Scalar) -> Self {
                match i {
                    $bound(x) => x,
                    _ =>  unreachable!()
                }
            }
        }
    )
}

convert!(bool, Scalar::Bool);
convert!(i32, Scalar::I32);
convert!(i64, Scalar::I64);

pub fn decode<T:From<Scalar>>(values:&[Scalar]) -> Vec<T> {
    values.iter().map(move |x| From::from(x.clone())).collect()
}

// Pretty printers..
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Scalar::None =>  write!(f, "{}", "None"),
            Scalar::Bool(x) => write!(f, "{}", x),
            Scalar::I32(x) => write!(f, "{}", x),
            Scalar::I64(x) => write!(f, "{}", x),
            Scalar::UTF8(x) => write!(f, "{}", x),
            Scalar::Tuple(x) => write!(f, "{:?}", x),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.kind)
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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