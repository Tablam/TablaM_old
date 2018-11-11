use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
//use std::rc::Rc;

use super::ndarray::*;
use super::values::*;

#[inline]
fn size_rel(of:&[Col], layout:Layout) -> (usize, usize) {
    if layout == Layout::Col {
        let cols = of.len();
        if cols > 0 {
            (cols, of[0].len())
        } else {
            (0, 0)
        }
    } else {
        let rows = of.len();
        if rows > 0 {
            (of[0].len(), rows)
        } else {
            (0, 0)
        }
    }
}

/// Calculate the appropriated index in the flat array
#[inline]
pub fn index(layout:&Layout, col_count:usize, row_count:usize, row:usize, col:usize) -> usize {
    //println!("pos {:?} Row:{}, Col:{}, R:{}, C:{}", layout, row, col, row_count , col_count);
    match layout {
        Layout::Col => col * row_count + row,
        Layout::Row => row * col_count + col,
    }
}

#[inline]
pub fn write_row(to:&mut Col, layout:&Layout, col_count:usize, row_count:usize, row:usize, data:Col) {
    for (col, value) in data.into_iter().enumerate() {
        let index = index(layout, col_count, row_count, row, col);
        to[index] = value;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Data {
    pub layout: Layout,
    pub names: Schema,
    pub ds: NDArray,
}

impl From<Scalar> for Data {
    fn from(of: Scalar) -> Self {
        Self::new_scalar(of)
    }
}

impl Data {
    pub fn new(names: Schema, layout: Layout, cols:usize, rows:usize, data:&[Scalar]) -> Self {
        Data {
            layout,
            names,
            ds: NDArray::new(rows, cols, data)
        }
    }

    pub fn new_scalar(data:Scalar) -> Self {
        Data {
            layout: Layout::Col,
            names: Schema::scalar_field(data.kind()),
            ds: NDArray::new(1, 1, [data].to_vec())
        }
    }

    pub fn empty(names: Schema, layout: Layout) -> Self {
        Self::new(names, layout, 0, 0, &[].to_vec())
    }

    pub fn new_cols(names: Schema, of:NDArraySlice) -> Self {
        let (cols, rows) = (of.cols(), of.rows());
        let columns = of.pivot();
        Self::new(names, Layout::Col, cols, rows, columns.into_array().data())
    }

    pub fn new_rows(names: Schema, of:NDArraySlice) -> Self {
        let (cols, rows) = (of.cols(), of.rows());

        Self::new(names, Layout::Row, cols, rows, of.into_array().data())
    }
}

/// Auxiliary functions and shortcuts
//macro_rules! array {
//    () => {
//        {
//            // Handle the case when called with no arguments, i.e. matrix![]
//            use $crate::array::NDArray;
//            NDArray::new(0, 0, vec![])
//        }
//    };
//    ($( $( $x: expr ),*);*) => {
//        {
//            use $crate::array::NDArray;
//            let data_as_nested_array = [ $( [ $($x),* ] ),* ];
//            let rows = data_as_nested_array.len();
//            let cols = data_as_nested_array[0].len();
//            let data_as_flat_array: Vec<_> = data_as_nested_array.into_iter()
//                .flat_map(|row| row.into_iter())
//                .cloned()
//                .collect();
//            NDArray::new(rows, cols, data_as_flat_array)
//        }
//    }
//}

pub fn hash_column(vec: Row) -> u64 {
    //println!("HASH {:?}", vec);
    let mut hasher = DefaultHasher::new();

    vec.into_iter().for_each(| x | x.hash(&mut hasher));

    let x = hasher.finish();
    //println!("HASH {:?}",x);
    x
}

pub fn colp(pos:usize) -> ColumnName {
    ColumnName::Pos(pos)
}
pub fn coln(name:&str) -> ColumnName {
    ColumnName::Name(name.to_string())
}


pub fn value<T>(x:T) -> Scalar
    where T:From<Scalar>, Scalar: From<T>
{
    Scalar::from(x)
}

pub fn none() -> Scalar
{
    Scalar::default()
}

pub fn infer_type(of:&NDArray) -> DataType {
    if of.is_empty() {
        DataType::None
    } else {
        of.get([0, 0]).unwrap().kind()
    }
}

pub fn infer_types(of:&NDArray) -> Vec<DataType> {
    of.iter().map(|x| x.kind()).collect()
}

pub fn col<T>(x:&[T]) -> Vec<Scalar>
where
    T:From<Scalar>, Scalar: From<T>,
    T: Clone
{
    x.iter().map( |v| value(v.clone())).collect()
}

pub fn concat(values:Vec<Col>) -> (usize, Col)
{
    let mut data = Vec::new();
    let mut rows = 0;
    for mut row in values {
        rows += 1;
        data.extend(row.into_iter());
    }

    (rows, data)
}

pub fn nd_array<T>(of:&[T], rows:usize, cols:usize) -> NDArray
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let col = col(of);
    NDArray::new(rows, cols, col)
}

pub fn field(name:&str, kind:DataType) -> Field {
    Field::new(name, kind)
}

pub fn schema_single(name:&str, kind:DataType) -> Schema {
    Schema::new_single(name, kind)
}
pub fn schema(names:&[(&str, DataType)]) -> Schema {
    let fields = names
            .into_iter()
            .map(|(name, kind)| Field::new(name, kind.clone())).collect();

    Schema::new(fields)
}

pub fn rcol_t<T>(name:&str, kind:DataType, of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, of.len(), 1);

    Data::new_cols(schema_single(name, kind), data.as_slice())
}

pub fn rcol<T>(name:&str, of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, of.len(), 1);
    let kind = infer_type(&data);

    Data::new_cols(schema_single(name, kind), data.as_slice())
}

pub fn array<T>(of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    rcol("it", of)
}

pub fn array_t<T>(kind:DataType, of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    rcol_t("it", kind, of)
}

pub fn array_empty(kind:DataType) -> Data
{
    Data::empty(Schema::scalar_field(kind), Layout::Col)
}

pub fn row<T>(names:Schema, of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, 1, of.len());
    Data::new_rows(names, data.as_slice())
}

pub fn row_infer<T>(of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = nd_array(of, 1, of.len());

    let types = infer_types(&data);
    let names = Schema::generate(&types);
    Data::new_rows(names, data.as_slice())
}

pub fn table_cols_infer(of: NDArray) -> Data {
    let mut types = Vec::with_capacity(of.cols());
    for c in of.col_iter() {
        types.push(infer_type(&c.into_array()));
    }
    let names = Schema::generate(&types);

    Data::new_cols(names, of.as_slice())
}

pub fn table_cols(schema:Schema, of: NDArray) -> Data {
    Data::new_cols(schema, of.as_slice())
}

pub fn table_rows(schema:Schema, of: NDArray) -> Data {
    Data::new_rows(schema, of.as_slice())
}

fn _print_cols(of: &Column, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, value) in of.iter().enumerate() {
        if i == of.cols() - 1{
            write!(f, "{}", value)?;
        } else {
            write!(f, "{}, ", value)?;
        }
    }
    Ok(())
}

fn _print_rows(of: &Row, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, value) in of.iter().enumerate() {
        if i == of.cols() - 1{
            write!(f, "{}", value)?;
        } else {
            write!(f, "{}, ", value)?;
        }
    }
    Ok(())
}

fn print_columns(of: &Data, f: &mut fmt::Formatter) -> fmt::Result {
    let (sep1, sep2) =  ("[|", "|]");

    write!(f, "{}", sep1)?;
    if of.ds.cols() > 0 {
        for (col, field) in of.names.columns.iter().enumerate() {
            write!(f, "{}= ", field)?;

            let item =  of.ds.col(col);
            _print_cols(&item, f)?;
            if col < of.ds.cols() - 1 {
                writeln!(f, ";")?;
            }
        }
    }
    writeln!(f, " {}", sep2)?;
    Ok(())
}

fn print_rows(of: &Data, f: &mut fmt::Formatter) -> fmt::Result {
    let (sep1, sep2) =  ("[<", ">]");

    write!(f, "{}", sep1)?;
    if of.ds.cols() > 0 {
        write!(f, "{}", of.names)?;
        writeln!(f, ";")?;

        for (pos, row) in of.ds.row_iter().enumerate() {
            _print_rows(&row, f)?;
            if pos < of.ds.rows() - 1 {
                writeln!(f, ";")?;
            }
        }

    }
    writeln!(f, " {}", sep2)?;
    Ok(())
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.layout == Layout::Col {
            print_columns(self, f)
        } else {
            print_rows(self, f)
        }
    }
}