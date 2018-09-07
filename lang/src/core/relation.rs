use std::cell::Cell;

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
        println!("P:{}", self.pos());
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

#[derive(Debug, Clone)]
pub struct JoinPos {
    len: usize,
    left: Vec<usize>,
    right: Vec<Vec<usize>>,
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
    pub fn cmp_both(&self, left:usize, right:usize, apply: &BoolExpr) -> bool
    {
        let lv = self.left.value(left);
        let lr = self.right.value(right);
        apply(lv, lr)
    }

    pub fn cmp_left(&self, left:&Scalar, apply: &BoolExpr) -> Vec<usize>
    {
        let mut result = Vec::new();
        while !self.right.eof() {
            let right = self.right.tuple(&self.cols_right);
            if apply(left, &right) {
                result.push(self.right.pos());
            }
            self.right.next();
        }
        result
    }

    pub fn tuples(&self) -> (Scalar, Scalar)
    {
        let lv = self.left.tuple(&self.cols_left);
        let lr = self.right.tuple(&self.cols_right);

        (lv, lr)
    }

    pub fn join(&self, apply:&BoolExpr, keep_nulls:bool) -> JoinPos
    {
        let mut cols_left = Vec::new();
        let mut cols_right = Vec::new();
        let mut total:usize = 0;

        while !self.left.eof() {
            let left = self.left.tuple(&self.cols_left);
            let result = self.cmp_left(&left, apply);
            let count = result.len();
            if count > 0 {
                cols_left.push(self.left.pos());
                cols_right.push(result);
            } else {
                if keep_nulls {
                    cols_left.push(self.left.pos());
                    cols_right.push(result);
                }
            }
            total = total + count;
            self.left.next();
            self.right.first();
        }

        JoinPos {
            left:cols_left,
            right:cols_right,
            len:total,
        }
    }

    pub fn eof(&self) -> bool {
        self.left.eof()
    }

    pub fn reset(&self) {
        self.left.first();
        self.right.first();
    }

    pub fn next(&self) -> bool {
        self.left.next()
    }

    pub fn materialize(&self, positions:JoinPos) -> Frame {
        self.reset();
        let schema = self.join_schema();
        let mut results = Vec::with_capacity(schema.len());

        for (pos, field) in self.left.source.names().columns.iter().enumerate() {
            let mut column = Vec::with_capacity(positions.len);
            for i in &positions.left {
                let cols = &positions.right[*i];
                //println!("MatL {:?} with {:?}", i, cols);
                if cols.len() > 0 {
                    let value = self.left.source.value(*i,pos);
                    let mut values = Scalar::repeat(value, cols.len());
                    column.append(&mut values);
                } else {
                    column.push(Scalar::None);
                }
            }
            results.push(Data::new(column, field.kind.clone()));
        }

        for (pos, field) in self.right.source.names().columns.iter().enumerate() {
            let mut column = Vec::with_capacity(positions.len);
            for i in &positions.left {
                let cols = &positions.right[*i];
                //println!("MatR {:?} with {:?}", i, cols);
                if cols.len() > 0 {
                    for r in cols {
                        let value = self.right.source.value(*r, pos).clone();
                        column.push(value);
                    }
                } else {
                    column.push(Scalar::None);
                }
            }
            results.push(Data::new(column, field.kind.clone()));
        }

//        println!("Schema: {:?}", schema);
//        println!("results: {:?}", results);

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