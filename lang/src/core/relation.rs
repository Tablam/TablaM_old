use std::cmp;
use std::cell::Cell;
use std::collections::HashSet;
use std::collections::HashMap;
//extern crate bit_vec;
//use self::bit_vec::BitVec;

use super::types::*;

pub trait Relation {
    fn layout(&self) -> Layout;
    fn col_count(&self) -> usize;
    fn row_count(&self) -> usize;
    fn names(&self)  -> Schema;
    fn row(&self, pos:usize) -> Data;
    fn col(&self, pos:usize) -> Data;
    fn col_at(&self, col:usize, pos:&Vec<usize>) -> Data;
    fn cols_at(&self, pos:&Vec<usize>) -> Vec<Data>;
    //fn rows_values_at(&self, pos:Vec<usize>) -> Data;
    fn value(&self, row:usize, col:usize) -> &Scalar;
    ///Recover the column position from the relative ColumnExp
    fn resolve_pos(&self, of: &ColumnExp) -> usize {
        let fields = self.names();
        match of {
            ColumnExp::Pos(x) => {
                *x
            },
            ColumnExp::Name(x) => {
                let (pos, _f) = fields.named(x).unwrap();
                pos
            }
        }
    }

    fn resolve_pos_many(&self, of: &Vec<ColumnExp>) -> Vec<usize>
    {
        of.into_iter().map(|x| self.resolve_pos(x)).collect()
    }

    ///Recover the column names from a list of relative ColumnExp
    fn resolve_names(&self, of: Vec<&ColumnExp>) -> Schema {
        let mut names = Vec::with_capacity(of.len());
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

    fn col_at(&self, col:usize, pos:&Vec<usize>) -> Data
    {
        let mut values = Vec::with_capacity(pos.len());

        for i in pos {
            values.push(self.data[*i].clone());
        }

        Data::new(values, self.kind.clone())
    }

    fn cols_at(&self, pos:&Vec<usize>) -> Vec<Data>
    {
        vec![self.col_at(0, pos)]
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        &self.data[row]
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

    fn col_at(&self, col:usize, pos:&Vec<usize>) -> Data
    {
        self.data[col].col_at(col, pos)
    }

    fn cols_at(&self, pos:&Vec<usize>) -> Vec<Data>
    {
        let mut columns = Vec::with_capacity(self.col_count());
        for i in 0..self.col_count() {
            columns.push(self.col_at(i, pos));
        }
        columns
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        &self.data[col].data[row]
    }
}

pub fn to_columns(fields:&Schema, data:Vec<Data>) -> Frame {
    let mut columns = Vec::with_capacity(fields.len());

    for (col, field) in fields.columns.iter().enumerate() {
        let mut column = Vec::with_capacity(data.len());

        for (pos, row) in data.iter().enumerate() {
            column.push(row.value(pos, col).clone());
        }
        columns.push(Data::new(column, field.kind.clone()));
    }

    Frame::new(fields.clone(), columns)
}

pub struct DataSource<T>
    where T:Relation
{
    pub source:T,
    pos: Cell<usize>,

}

impl <T> DataSource<T>
    where T:Relation
{
    pub fn new(source: T) -> Self {
        DataSource {
            pos: Cell::new(0),
            source
        }
    }

    fn _set(&self, pos:usize) {
        self.pos.set(pos)
    }
    pub fn pos(&self) -> usize {
        self.pos.get()
    }

    pub fn len(&self) -> usize {
        self.source.row_count()
    }

    pub fn eof(&self) -> bool {
        self.pos() + 1 > self.len()
    }

    pub fn first(&self) {
        self._set(0)
    }

    pub fn back(&self) -> bool {
        self.skip(-1)
    }

    pub fn next(&self) -> bool {
        self.skip(1)
    }

    pub fn last(&self) {
        self._set(self.len())
    }

    pub fn skip(&self, steps:isize) -> bool {
        let pos = (self.pos() as isize) + steps;

        if pos < 0 || pos > (self.len() as isize) {
            self._set(self.len() + 1);
            return false
        }
        self._set(pos as usize);
        true
    }

    pub fn value(&self, col:usize) -> &Scalar {
        //println!("P:{}", self.pos());
        self.source.value(self.pos(), col)
    }

    pub fn tuple(&self, columns:&Vec<usize>) -> Scalar {
        let mut values = Vec::with_capacity(columns.len());
        let pos = self.pos();

        for col in columns {
            values.push(self.source.value(pos, *col).clone())
        }
        Scalar::Tuple(values)
    }

    pub fn row(&self) -> Data {
        self.source.row(self.pos())
    }

    pub fn cmp(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> bool
    {
        let lv = self.value(col);
        println!("{:?}={:?}", lv, value);
        apply(lv, value)
    }

    /// Eager compare & collect all the values that match the value
    pub fn cmp_collect(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<bool>
    {
        let mut result = Vec::with_capacity(self.len());
        loop {
            let lv = self.cmp(col, value, apply);
            result.push(lv);
            self.next();
            if self.eof() {
                break;
            }
        }
        result
    }

    /// Find the first value that match
    pub fn find(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> bool
    {
        while !self.eof() {
            if self.cmp(col, value, apply) {
                return true
            }
            self.next();
        }
        false
    }

    /// Eager collect all the values that match the value
    pub fn find_collect(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<usize>
    {
        let mut result = Vec::new();
        while self.find(col, value, apply) {
            result.push(self.pos());
            self.next();
        }
        result
    }
}

pub fn hash_rel<T>(of:&DataSource<T>) -> HashMap<Data, usize>
    where T:Relation
{
    let mut rows = HashMap::with_capacity(of.len());

    while !of.eof() {
        rows.insert(of.row(), of.pos());
        of.next();
    }

    rows
}

#[derive(Debug, Clone)]
pub struct JoinPos {
    pub left:  Vec<isize>,
    pub right: Vec<isize>,
}

pub fn materialize<R>(source:&R, positions:&Vec<isize>, keep_null:bool) -> Vec<Data>
    where R:Relation
{
    let total = positions.len();
    let schema = source.names();
    let mut results = Vec::with_capacity(schema.len());
    let null = -1isize;

    for (pos, field) in schema.columns.iter().enumerate() {
        let mut column = Vec::with_capacity(total);
        for found in positions.iter() {
            if (found == &null) & keep_null {
                column.push(Scalar::None);
            } else {
                column.push(source.value(*found as usize,pos).clone());
            }
        }
        results.push(Data::new(column, field.kind.clone()));
    }

    results
}

pub struct Both<'a, 'b, T: 'a, U: 'b>
    where
        T: Relation,
        U: Relation
{
    pub left:   &'a DataSource<T>,
    pub right:  &'b DataSource<U>,
    pub cols_left: Vec<usize>,
    pub cols_right: Vec<usize>,
}

impl <'a, 'b, T, U> Both<'a, 'b, T, U>
    where
        T: Relation,
        U: Relation
{
    pub fn join(&self, apply:&BoolExpr) -> JoinPos
    {
        let total = cmp::max(self.left.len(), self.right.len());

        let mut right_not_founds = HashSet::new();
        let mut cols_left  = Vec::with_capacity(total);
        let mut cols_right = Vec::with_capacity(total);
        let mut found = false;
        let mut first = true;

        while !self.left.eof() {
            let left = self.left.tuple(&self.cols_left);
            while !self.right.eof() {
                let right = self.right.tuple(&self.cols_right);
                if first {
                    right_not_founds.insert(self.right.pos());
                }

                if apply(&left, &right) {
//                    println!("{} -> {} true", left, right);
                    cols_left.push(self.left.pos() as isize);
                    cols_right.push(self.right.pos() as isize);
                    right_not_founds.remove(&self.right.pos());

                    found = true;
                }
                self.right.next();
            }
            if !found {
                cols_left.push(self.left.pos() as isize);
                cols_right.push(-1);
            }
            found = false;
            first = false;
            self.left.next();
            self.right.first();
        }

        for pos in right_not_founds {
            cols_left.push(-1);
            cols_right.push(pos as isize);
        }

        JoinPos {
            left:cols_left,
            right:cols_right,
        }
    }

    pub fn reset(&self) {
        self.left.first();
        self.right.first();
    }

    pub fn cross(&self) -> Frame {
        self.reset();
        let total = self.left.len() * self.right.len();
        let right_size = self.right.len();

        let schema = self.join_schema();
        let mut results = Vec::with_capacity(schema.len());

        for (i, field) in self.left.source.names().columns.iter().enumerate() {
            let left = self.left.source.col(i);
            let mut result = Vec::with_capacity(total);

            for value in left.data.iter() {
                let mut value = Scalar::repeat(&value, right_size);
                result.append(&mut value);
            }

            results.push(Data::new(result, field.kind.clone()));
        }

        for (i, field) in self.right.source.names().columns.iter().enumerate()  {
            let result = self.right.source.col(i).data;

            let mut data = Vec::with_capacity(total);

            for i in 0..self.left.len() {
                for value in result.iter() {
                    data.push(value.clone());
                }
            }

            results.push(Data::new(data, field.kind.clone()));
        }

        Frame::new(schema, results)
    }

    pub fn materialize(&self, positions:JoinPos, join:Join) -> Frame {
        self.reset();
        let total = positions.left.len();
        let schema = self.join_schema();
        let mut results = Vec::with_capacity(schema.len());

        let null_lefts = join.produce_null(true);
        let null_rights = join.produce_null(false);

        let mut left = materialize(&self.left.source, &positions.left, null_lefts);
        let mut right = materialize(&self.right.source, &positions.right, null_rights);
        results.append(&mut left);
        results.append(&mut right);

        Frame::new(schema, results)
    }

    pub fn join_schema(&self) -> Schema
    {
        self.left.source.names().extend( self.right.source.names())
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