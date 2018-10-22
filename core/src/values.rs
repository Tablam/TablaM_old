use std::fmt;
use std::ops::Index;
use std::fmt::Debug;
use std::slice::Iter;

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
pub enum ColumnExp {
    Name(String),
    Pos(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Field {
    pub name: String,
    pub kind: DataType,
}

impl Field {
    pub fn new(name: &str, kind: DataType) -> Self {
        Field {
            name: name.to_string(),
            kind,
        }
    }

    pub fn new_owned(name: String, kind: DataType) -> Self {
        Field {
            name,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
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

pub type Col = Vec<Scalar>;
pub type Pos = Vec<usize>;

pub type BoolExpr = Fn(&Scalar, &Scalar) -> bool;
pub type BinExpr = Fn(&Scalar, &Scalar) -> Scalar;
pub type UnaryExpr = Fn(&Scalar) -> Scalar;

//TODO: Must implement Eq/Ord/Hash manually. dd
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
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

    pub fn generate(types:&[DataType]) -> Self {
        let mut names = Vec::with_capacity(types.len());

        for (pos, kind) in types.iter().enumerate() {
            names.push(Field::new_owned(pos.to_string(), kind.clone()));
        }

        Self::new(names)
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

    ///Recover the column position from the relative ColumnExp
    fn resolve_pos(&self, of: &ColumnExp) -> usize {
        match of {
            ColumnExp::Pos(x) => {
                *x
            },
            ColumnExp::Name(x) => {
                let (pos, _f) = self.named(x).unwrap();
                pos
            }
        }
    }

    fn resolve_pos_many(&self, of: &Vec<ColumnExp>) -> Pos
    {
        of.into_iter().map(|x| self.resolve_pos(x)).collect()
    }

    ///Recover the column names from a list of relative ColumnExp
    fn resolve_names(&self, of: Vec<&ColumnExp>) -> Schema {
        let mut names = Vec::with_capacity(of.len());

        for name in of.into_iter() {
            let pick =
                match name {
                    ColumnExp::Pos(x) => {
                        self.columns[*x].clone()
                    },
                    ColumnExp::Name(x) => {
                        let (_pos, f) = self.named(x).unwrap();
                        f.clone()
                    }
                };
            names.push(pick);
        }
        Schema::new(names)
    }

    /// Helper for select/projection
    pub fn only_pos(&self, position:Pos) -> Self {
        let mut fields = Vec::with_capacity(position.len());
        for pos in position {
            fields.push(self.columns[pos].clone());
        }
        Schema::new(fields)
    }

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

macro_rules! convert {
    ($kind:ident, $bound:path) => (
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
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}':{}", self.name, self.kind)
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

pub mod test_values {
    pub fn nums_1() -> Vec<i64> {
        vec![1, 2, 3]
    }

    pub fn nums_2() -> Vec<i64> {
        vec![4, 5, 6]
    }
}