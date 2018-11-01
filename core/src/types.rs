use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
//use std::rc::Rc;

use super::values::*;

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
pub fn index(layout:&Layout, col_count:usize, row_count:usize, row:usize, col:usize) -> usize {
    //println!("pos {}, {}, {}, {}", row, col, row_count , col_count);
    match layout {
        Layout::Col => col * row_count + row,
        Layout::Row => row * col_count + col,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Data {
    pub layout: Layout,
    pub cols: usize,
    pub rows: usize,
    pub names: Schema,
    pub ds: Col,
}

impl Data {
    pub fn new(names: Schema, layout: Layout, cols:usize, rows:usize, data:Col) -> Self {
        assert_eq!(cols == 0 || cols == names.len(), true, "The # of columns of schema and data must be equal");

        Data {
            layout,
            cols,
            rows,
            names,
            ds: data
        }
    }

    pub fn empty(names: Schema, layout: Layout) -> Self {
        Self::new(names, layout, 0, 0, [].to_vec())
    }

    pub fn new_cols(names: Schema, of:&[Col]) -> Self {
        let (cols, rows) = size_rel(of, Layout::Col);

        let mut data = vec![Scalar::default(); rows * cols];
        for (c, col) in of.into_iter().enumerate() {
            for (r, row) in col.into_iter().enumerate() {
                let index = index(&Layout::Col, cols, rows, r, c);
                data[index] = row.clone()
            }
        }

        Self::new(names, Layout::Col, cols, rows, data)
    }

    pub fn new_rows(names: Schema, of:&[Col]) -> Self {
        let (cols, rows) = size_rel(&of, Layout::Row);

        let mut data = vec![Scalar::default(); rows * cols];
        for (r, row) in of.into_iter().enumerate() {
            for (c, col) in row.into_iter().enumerate() {
                let index = index(&Layout::Row, cols, rows, r, c);
                data[index] = col.clone()
            }
        }

        Self::new(names, Layout::Row, cols, rows, data)
    }

    pub fn row_copy(&self, pos:usize) -> Col {
        let mut row = Vec::with_capacity(self.cols);

        for i in 0..self.cols {
            let index = index(&self.layout, self.cols, self.rows, pos, i);
            row.push(self.ds[index].clone())
        }
        row
    }

    pub fn col_slice(&self, pos:usize) -> &[Scalar] {
        let start = pos * self.rows;
        let end = start + self.rows;
        &self.ds[start..end]
    }

    pub fn index(&self, row:usize, col:usize) -> usize {
        index(&self.layout, self.cols, self.rows, row, col)
    }

    pub fn value_owned(&self, row:usize, col:usize) -> &Scalar {
        let index = self.index(row, col);
        &self.ds[index]
    }
}

pub struct RowIter
{
    pos: usize,
    data: Data
}

impl IntoIterator for Data
{
    type Item = Col;
    type IntoIter = RowIter;

    fn into_iter(self) -> Self::IntoIter {
        RowIter {pos:0, data:self}
    }
}

impl Iterator for RowIter
{
    type Item = Col;

    fn next (&mut self) -> Option<Self::Item> {
        if self.pos < self.data.rows {
            self.pos = self.pos + 1;
            let row = self.data.row_copy(self.pos);
            Some(row)
        } else {
            None
        }
    }
}

/// Auxiliary functions and shortcuts
pub fn hash_column(vec: Col) -> u64 {
    //println!("HASH {:?}", vec);
    let mut hasher = DefaultHasher::new();

    vec.into_iter().for_each(| x | x.hash(&mut hasher));

    let x = hasher.finish();
    //println!("HASH {:?}",x);
    x
}

pub fn colp(pos:usize) -> ColumnExp {
    ColumnExp::Pos(pos)
}
pub fn coln(name:&str) -> ColumnExp {
    ColumnExp::Name(name.to_string())
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

pub fn infer_type(of:&[Scalar]) -> DataType {
    if of.len() > 0 {
        of[0].kind()
    } else {
        DataType::None
    }
}

pub fn infer_types(of:&[Scalar]) -> Vec<DataType> {
    of.iter().map(|x| x.kind()).collect()
}

pub fn col<T>(x:&[T]) -> Vec<Scalar>
where
    T:From<Scalar>, Scalar: From<T>,
    T: Clone
{
    x.iter().map( |v| value(v.clone())).collect()
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
    let data = col(of);

    Data::new_cols(schema_single(name, kind), vec![data].as_slice())
}

pub fn rcol<T>(name:&str, of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = col(of);
    let kind = infer_type(data.as_slice());

    Data::new_cols(schema_single(name, kind), vec![data].as_slice())
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
    let data = col(of);

    Data::new_rows(names, vec![data].as_slice())
}

pub fn row_infer<T>(of:&[T]) -> Data
    where
        T:From<Scalar>, Scalar: From<T>,
        T: Clone
{
    let data = col(of);
    let types = infer_types(&data);
    let names = Schema::generate(&types);
    Data::new_rows(names, vec![data].as_slice())
}

pub fn table_cols_infer<T>(of:&[Col]) -> Data {
    let mut types = Vec::with_capacity(of.len());
    for c in of {
        types.push(infer_type(c));
    }
    let names = Schema::generate(&types);

    Data::new_cols(names, of)
}

pub fn table_cols<T>(schema:Schema, of:&[Col]) -> Data {
    Data::new_cols(schema, of)
}

fn print_values(of: &[Scalar], f: &mut fmt::Formatter) -> fmt::Result {
    for (i, value) in of.iter().enumerate() {
        if i == of.len() - 1{
            write!(f, "{}", value)?;
        } else {
            write!(f, "{}, ", value)?;
        }
    }
    Ok(())
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (sep1, sep2) = if self.layout == Layout::Col { ("[|", "|]") } else { ("[<", ">]") };

        write!(f, "{} ", sep1)?;
        if self.cols > 0 {

            write!(f, "{}", self.names)?;
            write!(f, "; ")?;

            if self.layout == Layout::Col {
                for col in 0..self.cols {
                    let item =  self.col_slice(col);
                    print_values(item, f)?;
                    if col < self.cols - 1 {
                        write!(f, ";")?;
                    }
                }
            } else {
                for row in 0..self.rows {
                    for col in 0..self.cols  {
                        let item =  self.value_owned(row, col);
                        if col > 0 {
                            write!(f, ", {}", item)?;
                        } else {
                            write!(f, "{}", item)?;
                        }
                    }
                    if row < self.rows - 1 {
                        write!(f, "; ")?;
                    }
                }
            }
        }
        writeln!(f, " {}", sep2)?;
        Ok(())
    }
}