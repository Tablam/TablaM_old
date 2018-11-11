use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;

extern crate bit_vec;
use self::bit_vec::BitVec;

use super::ndarray::*;
use super::values::*;
use super::types::*;

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

fn _check_not_found(cmp:&HashMap<u64, usize>, row:u64) -> bool {
    !cmp.contains_key(&row)
}

fn _check_found(cmp:&HashMap<u64, usize>, row:u64) -> bool {
    cmp.contains_key(&row)
}

fn _bitvector_count(of:&BitVec) -> (usize, usize) {
    let trues = of.iter().filter(|x| *x).count();

    (trues, of.len() - trues)
}

pub(crate) fn _count_join(of:&BitVec, keep_nulls:bool) -> usize {
    if keep_nulls {
        let (_, total) = _bitvector_count(of);
        total
    } else {
        0
    }
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

pub fn _compare_hash<T, U>(left:&T, right:&U, mark_found:bool) -> (BitVec, usize)
    where
        T: Relation,
        U: Relation
{
    let cmp = left.hash_rows();
    let mut results = BitVec::from_elem(right.row_count(), false);
    let mut not_found = 0;
    let check =
        if mark_found {
            _check_found
        }  else {
            _check_not_found
        };

    let mut cursor = Cursor::new(0, right.row_count());

    while let Some(next) = right.next(&mut cursor) {
        let h = right.hash_row(next);

        if check(&cmp, h) {
            results.set(next, true);
        } else {
            not_found = not_found + 1;
        }
    }

    (results, not_found)
}

pub fn _join_late<T:Relation, U:Relation>(from:&T, to:&U, cols_from:&[usize], null_from:bool, cols_to:&[usize], null_to:bool, apply: &BoolExpr) -> (BitVec, BitVec) {
    let mut right_not_founds = HashSet::new();

    let left = from.row_count();
    let right = to.row_count();

    let total = cmp::max(left, right);

    let mut cols_left  = BitVec::from_elem(total, false);
    let mut cols_right = BitVec::from_elem(total, false);

    let mut found = false;
    let mut first = true;

    for x in 0..left {
        for y in 0..right  {
            if null_from && first {
                right_not_founds.insert(y);
            }
            let l = &from.row_only(x, cols_from);
            if to.cmp_cols(y, cols_to, l.as_slice(), apply) {
                //println!("{} -> {} true", x, y);
                cols_left.set(x, true);
                cols_right.set(y, true);
                right_not_founds.remove(&y);
                found = true;
            }
        }
        if null_from && !found {
            //println!("..{} true", x);
            cols_left.set(x, true);
        }
        found = false;
        first = false;
    }

    if null_to && right_not_founds.len() > 0 {
        cols_left.grow(right_not_founds.len(), false);
        cols_right.grow(right_not_founds.len(), false);

        for pos in right_not_founds {
            cols_right.set(pos, true);
        }
    }

    (cols_left, cols_right)
}

#[derive(Debug)]
pub struct JoinClause<'a> {
    pub rel: &'a[Scalar],
    pub kind: DataType,
}

fn cmp_row(row: &Row, col: usize, value: &Scalar, apply: &BoolExpr) -> Option<Col>
{
    let old = row.get([0, col]).unwrap();
    //println!("CMP {:?}, {:?}", value, old);
    if apply(old, value) {
      Some(row.raw_slice().to_vec())
    } else {
      None
    }
}

pub trait Relation {
    fn new(names: Schema, of: NDArray) -> Self;
    fn empty(names: Schema) -> Self;
    fn from_raw(names: Schema, layout: Layout, cols: usize, rows: usize, of: &[Scalar]) -> Self;
    fn from_slice(names: Schema, layout: Layout, of: NDArraySlice) -> Self;

    fn layout(&self) -> &Layout;
    fn names(&self) -> &Schema;

    fn row_count(&self) -> usize;
    fn col_count(&self) -> usize;
    fn row(&self, pos: usize) -> Row;

    fn col(&self, pos: usize) -> Column;
    fn value(&self, row: usize, col: usize) -> &Scalar;

    fn flat_raw(&self, layout: &Layout) -> Col {
        let rows = self.row_count();
        let cols = self.col_count();

        let mut data = Vec::with_capacity(cols * rows);

        if *layout == Layout::Col {
            for col in 0..cols {
                for row in 0..rows {
                    data.push(self.value(row, col).clone())
                }
            }
        } else {
            for row in 0..rows {
                for col in 0..cols {
                    data.push(self.value(row, col).clone())
                }
            }
        }

        data
    }

    fn materialize_raw(&self, pos:&BitVec, null_count:usize, layout: &Layout, keep_null:bool) -> Col {
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

        let mut new_row = 0;
        for (row, found) in positions {
            for col in 0..cols {
                let _pos = index(layout, cols, total_rows, new_row, col);
                if found {
                    data[_pos] = self.value(row, col).clone();
                }
            }
            new_row += 1;
        }

        data
    }

    fn materialize_data(&self, pos:&BitVec, layout: &Layout, keep_null:bool) -> NDArray {
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

        let mut new_row = 0;
        for (row, found) in positions {
            for col in 0..cols {
                if found {
                    let _pos = index(layout, cols, total_rows, new_row, col);
                    data[_pos] = self.value(row, col).clone();
                }
            }
            new_row += 1;
        }

        NDArray::new(total_rows, cols, data)
    }

    fn rows_pos(&self, pick: Pos) -> NDArray;

    fn hash_row(&self, row:usize) -> u64 {
        hash_column(self.row(row))
    }

    fn hash_rows(&self) -> HashMap<u64, usize> {
        let mut rows = HashMap::with_capacity(self.row_count());

        let mut cursor = Cursor::new(0, self.row_count());

        while let Some(next) = self.next(&mut cursor) {
            rows.insert(self.hash_row(next), next);
        }

        rows
    }

    fn rows(&self) -> Rows;

    fn row_only(&self, row: usize, cols: &[usize]) -> Col {
        let mut data = Vec::with_capacity(cols.len());

        for i in cols {
            data.push(self.value(row, *i).clone())
        }
        data
    }

    fn tuple(&self, row: usize, cols: &[usize]) -> Scalar {
        Scalar::Tuple(self.row_only(row, cols))
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

    //TODO: Specialize for columnar layout
    fn next(&self, cursor: &mut Cursor) -> Option<usize> {
        while !cursor.eof() {
            let row = cursor.start;
            cursor.next();
            return Some(row)
        }

        Option::None
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

    fn find_all(&self, start:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<usize>
    {
        let mut pos = Vec::new();

        let mut cursor = Cursor::new(start, self.row_count());

        while let Some(next) = self.find(&mut cursor, col, value, apply) {
            pos.push(next);
        }

        pos
    }

    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> NDArray
    {
        let rows:Vec<Scalar> = self.rows()
            .filter_map(|x| cmp_row(&x, col, value, apply))
            .flatten()
            .collect();
        let total_rows = rows.len() / self.col_count();

        NDArray::new(total_rows, self.col_count(), rows)
    }

    fn rename<T:Relation>(of:&T, change:&[(ColumnName, &str)]) -> T {
        let schema = of.names().rename(change);
        T::from_raw(schema, of.layout().clone(), of.col_count(), of.row_count(), &of.flat_raw(of.layout()))
    }

    fn select<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
        let old = of.names();
        let pos = old.resolve_pos_many(pick);
        let names = old.only(pos.as_slice());
        T::new(names, of.rows_pos(pos))
    }

    fn deselect<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
        let old = of.names();
        let pos = old.resolve_pos_many(pick);

        let deselect = old.except(pos.as_slice());
        let names = old.only(deselect.as_slice());
        T::new(names, of.rows_pos(deselect))
    }

    fn where_value_late<T:Relation>(of:&T, col:usize, value:&Scalar, apply:&BoolExpr) -> T {
        let rows = T::find_all_rows(of, col, value, apply);

        T::new(of.names().clone(), rows)
    }

    fn union<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        assert_eq!(names, to.names(), "The schemas must be equal");
        let layout = from.layout();
        let rows = from.row_count() + to.row_count();

        let mut left = from.flat_raw(layout);
        let mut right = to.flat_raw(layout);

        left.append(&mut right);

        T::from_raw(names.clone(), layout.clone(), names.len(), rows, &left)
    }

    fn intersection<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        assert_eq!(names, to.names(), "The schemas must be equal");
        let layout = to.layout();
        let (pos, null_count) = _compare_hash(from, to, true);

        let data = to.materialize_raw(&pos, null_count, layout, false);

        T::from_raw(names.clone(), layout.clone(), names.len(), pos.len() - null_count, &data)
    }

    fn difference<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        assert_eq!(names, to.names(), "The schemas must be equal");
        let layout = to.layout();
        let (pos1, null_count1) = _compare_hash(from, to, false);
        let (pos2, null_count2) = _compare_hash(to, from, false);

        let mut data = to.materialize_raw(&pos1, null_count1, layout, false);
        let mut data2 = from.materialize_raw(&pos2, null_count2, layout, false);
        data.append(&mut data2);
        let total_rows = (pos1.len() - null_count1) + (pos2.len() - null_count2);

        T::from_raw(names.clone(), layout.clone(), names.len(), total_rows, &data)
    }

    fn cross<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        let others = &from.names().join(to.names());
        let layout = &Layout::Row;
        let cols = names.len() + others.len();
        let rows = from.row_count() * to.row_count();
        //println!("{:?} {:?} ",names, others);

        let mut data = vec![Scalar::None; rows * cols];
        let mut pos:usize = 0;

        for left in from.rows() {
            for right in 0..to.row_count() {
                let mut extra_row = to.row_only(right, others);
                let mut row = left.into_array().into_vec();
                row.append(&mut extra_row);
                //println!("{:?} {} {} {}", row, cols, rows,pos);
                write_row(&mut data, layout, cols, rows, pos, row);
                pos += 1;
            }
        }
        let schema = names.extend(to.names().only(others));

        T::from_raw(schema, layout.clone(), cols, rows, &data)
    }

    fn join<T:Relation, U:Relation>(from:&T, to:&U, join:Join, cols_from:&[usize], cols_to:&[usize], apply:&BoolExpr) -> T
    {
        let null_lefts = join.produce_null(true);
        let null_rights = join.produce_null(false);

        let (left, right) = _join_late(from, to, cols_from, null_lefts, cols_to, null_rights, apply);
        let names = from.names();

        let others= &names.join(to.names());
        let cols = names.len() + others.len();

        let layout = from.layout();

        let data = from.materialize_data(&left, layout, null_lefts);
        let data2 = to.materialize_data(&right, layout, null_rights);

        println!("L: {:?}", data);
        println!("R: {:?}", data2);

        let result = data.hcat(&data2);

        println!("RESULT: {:?}", result);
        let schema = names.extend(to.names().only(others));
        T::from_raw(schema, layout.clone(), cols, result.rows(), result.into_vec().as_slice())
    }

    fn append<T:Relation, U:Relation>(to:&T, from:&U) -> T {
        assert_eq!(to.names(), from.names(), "The schemas must be equal");

        let layout = &Layout::Row ;//from.layout();
        let total_rows = to.row_count() + from.row_count();
        let mut left = to.flat_raw(layout);
        let mut right=  from.flat_raw(layout);
        //println!("APP: {} {:?}\n {:?}\n", total_rows, left, right);

        left.append(&mut right);
        //println!("R: {:?}", left);

        T::from_raw(to.names().clone(), layout.clone(), to.col_count(), total_rows, &left)
    }
}

/// Fundamental relational operators.

pub fn select<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
    T::select(of, pick)
}

pub fn deselect<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
    T::deselect(of, pick)
}

pub fn rename<T:Relation>(of:&T, change:&[(ColumnName, &str)]) -> T {
    T::rename(of, change)
}

pub fn where_value_late<T:Relation>(of:&T, col:usize, value:&Scalar, apply:&BoolExpr) -> T {
    T::where_value_late(of, col, value, apply)
}

pub fn cross<T:Relation, U:Relation>(from:&T, to:&U) -> T
{
    T::cross(from, to)
}

pub fn union<T:Relation, U:Relation>(from:&T, to:&U) -> T
{
    T::union(from, to)
}

pub fn intersection<T:Relation, U:Relation>(from:&T, to:&U) -> T
{
    T::intersection(from, to)
}

pub fn difference<T:Relation, U:Relation>(from:&T, to:&U) -> T
{
    T::difference(from, to)
}

pub fn join<T:Relation, U:Relation>(from:&T, to:&U, join:Join, cols_from:&[usize], cols_to:&[usize], apply:&BoolExpr) -> T
{
    T::join(from, to, join, cols_from, cols_to, apply)
}

pub fn append<T:Relation, U:Relation>(to:&T, from:&U) -> T
{
    T::append(to, from)
}

impl Relation for Data {
    fn new(names: Schema, of: NDArray) -> Self {
        Data::new_rows(names, of)
    }

    fn empty(names:Schema) -> Self {
        Data::empty(names, Layout::Col)
    }

    fn from_raw(names: Schema, layout: Layout, cols:usize, rows:usize, of:&[Scalar]) -> Self
    {
        Data::from_raw(names, layout, cols, rows, of)
    }

    fn from_slice(names: Schema, layout: Layout, of: NDArraySlice) -> Self {
        Data::new(names, layout, of.into_array())
    }

    fn layout(&self) -> &Layout {
        &self.layout
    }
    fn names(&self) -> &Schema {
        &self.names
    }

    fn row_count(&self) -> usize {
        self.ds.rows()
    }

    fn col_count(&self) -> usize {
        self.ds.cols()
    }

    fn row(&self, pos:usize) -> Row {
        self.ds.row(pos)
    }
    fn rows(&self) -> Rows {
        self.ds.row_iter()
    }
    fn col(&self, pos:usize) -> Column {
        self.ds.col(pos)
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        self.ds.get([row, col]).unwrap()
    }

    fn rows_pos(&self, pick: Pos) -> NDArray {
        self.ds.select_rows(&pick)
    }
}