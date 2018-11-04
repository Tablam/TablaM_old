use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;

extern crate bit_vec;
use self::bit_vec::BitVec;

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

    (trues, trues - of.len())
}

fn _count_join(of:&BitVec, keep_nulls:bool) -> usize {
    if keep_nulls {
        let (_, total) = _bitvector_count(of);
        total
    } else {
        0
    }
}

fn _bitvector_to_pos(of:&BitVec) -> Vec<isize> {
    let mut pos =  vec![-1isize; of.len()];

    for (i, found) in of.iter().enumerate() {
        if found {
            pos[i] = i as isize;
        }
    }
    pos
}

fn _compare_hash<T, U>(left:&T, right:&U, mark_found:bool) -> (BitVec, usize)
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

fn _join_late<T:Relation, U:Relation>(from:&T, to:&U, cols_from:&[usize], cols_to:&[usize], apply: &BoolExpr) -> (BitVec, BitVec) {
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
            if first {
                right_not_founds.insert(y);
            }
            let l = &from.tuple(x, cols_from);
            let r = &to.tuple(y, cols_to);
            if apply(l , r) {
                //println!("{} -> {} true", x, y);
                cols_left.set(x, true);
                cols_right.set(y, true);
                right_not_founds.remove(&y);
                found = true;
            }
        }
        if !found {
            //println!("..{} true", x);
            cols_left.set(x, true);
        }
        found = false;
        first = false;
    }

    if right_not_founds.len() > 0 {
        cols_left.grow(right_not_founds.len(), false);
        cols_right.grow(right_not_founds.len(), false);

        for pos in right_not_founds {
            cols_right.set(pos, true);
        }
    }

    (cols_left, cols_right)
}

pub trait Relation {
    fn empty(names: Schema) -> Self;
    fn from_raw(names: Schema, layout: Layout, cols: usize, rows: usize, of: Col) -> Self;
    fn new(names: Schema, of: &[Col]) -> Self;

    //fn append(to:&mut Self, from:&[Scalar]) ;
    fn layout(&self) -> &Layout;
    fn names(&self) -> &Schema;

    fn row_count(&self) -> usize;
    fn col_count(&self) -> usize;
    fn row(&self, pos: usize) -> Col;

    fn col(&self, pos: usize) -> Col;
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
        let positions:Vec<(usize, bool)> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x || keep_null).collect();

        //println!("Raw r:{} c:{} total: {} {}", rows, cols, total_rows, positions.len());

        let mut new_row = 0;
        for (row, _) in positions {
            for col in 0..cols {
                let _pos = index(layout, cols, total_rows, new_row, col);

                data[_pos] = self.value(row, col).clone();
            }
            new_row += 1;
        }

        data
    }

    fn rows_pos(&self, pick: Pos) -> Vec<Col> {
        let total = self.row_count();
        let row_size = pick.len();
        let mut columns = Vec::with_capacity(total);

        for pos in 0..total {
            let mut row = Vec::with_capacity(row_size);
            let old = self.row(pos);
            for p in &pick {
                row.push(old[*p].clone());
            }
            columns.push(row)
        }

        columns
    }

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

    fn rows(&self) -> Vec<Col> {
        let total = self.row_count();
        let mut columns = Vec::with_capacity(total);
        for pos in 0..total {
            let row = self.row(pos);
            columns.push(row)
        }

        columns
    }

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
        println!("CMP {:?}, {:?}", value, old);
        apply(old, value)
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

    fn find_all_rows(&self, start:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<Col>
    {
        let mut pos = Vec::new();
        let mut cursor = Cursor::new(start, self.row_count());

        while let Some(next) = self.find(&mut cursor, col, value, apply) {
            pos.push(self.row(next));
        }

        pos
    }

    fn rename<T:Relation>(of:&T, change:&[(ColumnName, &str)]) -> T {
        let schema = of.names().rename(change);
        T::from_raw(schema, of.layout().clone(), of.col_count(), of.row_count(), of.flat_raw(of.layout()))
    }

    fn select<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
        let old = of.names();
        let pos = old.resolve_pos_many(pick);
        let names = old.only(pos.as_slice());
        T::new(names, of.rows_pos(pos).as_slice())
    }

    fn deselect<T:Relation>(of:&T, pick:&[ColumnName]) -> T {
        let old = of.names();
        let pos = old.resolve_pos_many(pick);

        let deselect = old.except(pos.as_slice());
        let names = old.only(deselect.as_slice());
        T::new(names, of.rows_pos(deselect).as_slice())
    }

    fn where_value_late<T:Relation>(of:&T, col:usize, value:&Scalar, apply:&BoolExpr) -> T {
        let rows = T::find_all_rows(of, 0, col, value, apply);

        T::new(of.names().clone(), rows.as_slice())
    }

    fn union<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        assert_eq!(names, to.names(), "The schemas must be equal");
        let layout = from.layout();
        let rows = from.row_count() + to.row_count();

        let mut left = from.flat_raw(layout);
        let mut right = to.flat_raw(layout);
        left.append(&mut right);

        T::from_raw(names.clone(), layout.clone(), names.len(), rows, left)
    }

    fn intersection<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        assert_eq!(names, to.names(), "The schemas must be equal");
        let layout = to.layout();
        let (pos, null_count) = _compare_hash(from, to, true);

        let data = to.materialize_raw(&pos, null_count, layout, false);

        T::from_raw(names.clone(), layout.clone(), names.len(), pos.len() - null_count, data)
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

        T::from_raw(names.clone(), layout.clone(), names.len(), total_rows, data)
    }

    fn cross<T:Relation, U:Relation>(from:&T, to:&U) -> T {
        let names = from.names();
        let others = &from.names().join(to.names());
        let layout = to.layout();
        let cols = names.len() + others.len();
        let rows = from.row_count() * to.row_count();
        //println!("{:?} {:?} ",names, others);
        let mut data = vec![Scalar::None; rows * cols];
        let mut pos:usize = 0;

        for  left in &from.rows() {
            for right in 0..to.row_count() {
                let mut extra_row = to.row_only(right, others);
                //println!("{:?} {:?} {} {} {}", left, extra_row, cols, rows,pos);
                let mut row = left.clone();
                row.append(&mut extra_row);

                write_row(&mut data, layout, cols, rows, pos, row);
                pos += 1;
            }
        }
        let schema = names.extend(to.names().only(others));

        T::from_raw(schema, layout.clone(), cols, rows, data)
    }

    fn join<T:Relation, U:Relation>(from:&T, to:&U, join:Join, cols_from:&[usize], cols_to:&[usize], apply:&BoolExpr) -> T
    {
        let  (left, right) = _join_late(from, to, cols_from, cols_to, apply);
        let names = from.names();
        let others= &names.join(to.names());
        let cols = names.len() + others.len();
        let layout = from.layout();

        let null_lefts = join.produce_null(true);
        let null_rights = join.produce_null(false);
        let null_count1= _count_join(&left, null_lefts);
        let null_count2= _count_join(&right, null_rights);

        let mut data = from.materialize_raw(&right, null_count2, layout, null_rights);
        let mut data2 = to.materialize_raw(&left, null_count1, layout, null_lefts);
        data.append(&mut data2);
        let total_rows = cmp::max(left.len(), right.len());

        let schema = names.extend(to.names().only(others));
        T::from_raw(schema, layout.clone(), cols, total_rows, data)
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

impl Relation for Data {
    fn empty(names:Schema) -> Self {
        Data::empty(names, Layout::Col)
    }

    fn from_raw(names: Schema, layout: Layout, cols:usize, rows:usize, of:Col) -> Self
    {
        Data::new(names, layout, cols, rows, of)
    }

    fn new(names: Schema, of:&[Col]) -> Self {
        Data::new_rows(names, of)
    }

    fn layout(&self) -> &Layout {
        &self.layout
    }
    fn names(&self) -> &Schema {
        &self.names
    }

    fn row_count(&self) -> usize {
        self.rows
    }

    fn col_count(&self) -> usize {
        self.cols
    }

    fn row(&self, pos:usize) -> Col {
        self.row_copy(pos)
    }

    fn col(&self, pos:usize) -> Col {
        self.col_slice(pos).to_vec()
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        self.value_owned(row, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::values::test_values::*;
    use super::super::values::DataType::*;
    use super::super::stdlib::*;

    pub fn rel_empty() -> Data { array_empty(I32) }
    pub fn rel_nums1() -> Data {
        array(nums_1().as_slice())
    }
//    pub fn rel_nums2() -> Data {
//        array(nums_2().as_slice())
//    }
    pub fn rel_nums3() -> Data {
        array(nums_3().as_slice())
    }

    fn make_both(left:Vec<i64>, right:Vec<i64>) -> (Data, Data, usize, usize) {
        let f1 = array(left.as_slice());
        let f2 = array(right.as_slice());
        let (pick1, pick2) = (0,  0);

        (f1, f2, pick1, pick2)
    }

    pub fn table_1() -> Data {
        let fields = [field("one", I32), field("two", I32), field("three", Bool)].to_vec();
        let schema = Schema::new(fields);
        let c1 = encode(nums_1().as_slice());
        let c2 = encode(nums_2().as_slice());
        let c3 = encode(bools_1().as_slice());

        table_cols::<Data>(schema, &[c1, c2, c3])
    }

    pub fn table_2() -> Data {
        let fields = [field("four", I32), field("five", I32), field("six", Bool)].to_vec();
        let schema = Schema::new(fields);
        let c1 = encode(nums_1().as_slice());
        let c2 = encode(nums_3().as_slice());
        let c3 = encode(bools_1().as_slice());

        table_cols::<Data>(schema, &[c1, c2, c3])
    }

    fn open_test_file(path:&str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let paths = [root,  "test_data", path].to_vec();

        let paths =  path_combine(paths.as_slice());
        println!("{:?}, {:?}",paths, root);

        let name =paths.to_str().expect("Wrong path?");
        read_all(name).expect("File not exist")
    }

    #[test]
    fn test_create() {
        let num1 = nums_1();
        let empty_schema = &Schema::scalar_field(I32);

        let fnull = rel_empty();
        assert_eq!(fnull.names(), empty_schema);
        assert_eq!(fnull.layout(), &Layout::Col);
        println!("Empty {:?}", fnull);

        assert_eq!(fnull.col_count(), 0);
        assert_eq!(fnull.row_count(), 0);

        let fcol1 = rel_nums1();
        println!("Array {}", fcol1);
        assert_eq!(fnull.names(), empty_schema);
        assert_eq!(fcol1.layout(), &Layout::Col);

        assert_eq!(fcol1.col_count(), 1);
        assert_eq!(fcol1.row_count(), 3);

        let frow1 = row_infer(num1.as_slice());
        assert_eq!(frow1.layout(), &Layout::Row);

        println!("Rows {}", frow1);
        assert_eq!(frow1.col_count(), 3);
        assert_eq!(frow1.row_count(), 1);

        let table1 = table_1();
        println!("Table Cols {}", table1);
        assert_eq!(table1.col_count(), 3);
        assert_eq!(table1.row_count(), 3);
        println!("Table Col1 {:?}", table1.names());
        println!("Table Col1 {:?}", table1.col(0));
    }

    #[test]
    fn test_select() {
        let (pick1, pick2, pick3) = (colp(0),  colp(1),  coln("three"));

        let table1 = &table_1();
        println!("Table1 {}", table1);
        let query_empty = select(table1, &[]);
        println!("Select 0 {}", query_empty);
        assert_eq!(query_empty.names.len(), 0);

        let query1 = select(table1, &[pick1]);
        println!("Select 0 {}", query1);
        assert_eq!(query1.names.len(), 1);
        let query2 = select(table1, &[pick2.clone()]);
        println!("Select 1 {}", query2);
        assert_eq!(query2.names.len(), 1);

        let query3 = deselect(table1, &[pick2, pick3]);
        println!("DeSelect 1 {}", query3);
        assert_eq!(query3.names.len(), 1);
        assert_eq!(query1.names, query3.names);
    }

    #[test]
    fn test_compare() {
        let table1 = &table_1();
        println!("Table1 {}", table1);
        let none = &none();
        let one = &value(1i64);

        let pos1 = table1.find_all(0, 0, one, &PartialEq::eq);
        assert_eq!(pos1, [0].to_vec());

        let query1 = where_value_late(table1, 0, none, &PartialEq::eq);
        println!("Where1 = None {}", query1);
        assert_eq!(query1.row_count(), 0);

        let query2 = where_value_late(table1,0, one, &PartialEq::eq);
        println!("Where2 = 1 {}", query2);
        assert_eq!(query2.row_count(), 1);
    }

    #[test]
    fn test_rename() {
        let table =  &table_1();
        let renamed = rename(table, &[(colp(0), "changed")]);

        assert_eq!(table.col_count(), renamed.col_count());
        assert_eq!(renamed.names.columns[0].name, "changed".to_string());
    }

    #[test]
    fn test_union() {
        let table1 =  &table_1();
        let table2 =  &table_1();

        let table3 = union(table1, table2);
        println!("Union {}", table3);

        assert_eq!(table1.col_count(), table3.col_count());
        assert_eq!(table1.row_count() * 2, table3.row_count());
    }

    #[test]
    fn test_hash() {
        let table1 =  &table_1();
        let hashed = table1.hash_rows();
        println!("Hash {:?}", hashed);
        assert_eq!(hashed.len(), table1.row_count());
        assert_eq!(hashed.contains_key(&hash_column(table1.row(0))), true);
    }

    #[test]
    fn test_intersection() {
        let left = &rel_nums1();
        let right = &rel_nums3();

        let result = _compare_hash(left, right, true);
        println!("CMP {:?}", result);

        println!("Left {}", left);
        println!("Right {}", right);

        let result = intersection(left, right);
        println!("Intersection {}", result);
        assert_eq!(result,  array(&[2i64, 3i64]));
    }

    #[test]
    fn test_difference() {
        let left = &rel_nums1();
        let right = &rel_nums3();

        let result1 = _compare_hash(left, right, false);
        let result2 = _compare_hash(right, left, false);
        println!("CMP {:?}, {:?}", result1, result2);

        println!("Left {}", left);
        println!("Right {}", right);

        let result = difference(left, right);
        println!("Difference {}", result);
        assert_eq!(result,  array(&[4i64, 1i64]));
    }

    #[test]
    fn test_cross_join() {
        let left = &rel_nums1();
        let right= &rename(&rel_nums3(), &[(colp(0), "changed")]);

        let both = cross(left, right);
        println!("Cross {}", both);
        assert_eq!(both.col_count(), 2);

        let col1 = both.col(0);
        let col2 = both.col(1);
        let r1:Vec<i64> =vec![1, 1, 1, 2, 2, 2, 3, 3, 3];
        let r2:Vec<i64> =vec![2, 3, 4, 2, 3, 4, 2, 3, 4];

        assert_eq!(col1, col(r1.as_slice()));
        assert_eq!(col2, col(r2.as_slice()));
    }

    fn _test_join(left:Vec<i64>, right:Vec<i64>, mut join_left:Vec<isize>, mut join_right:Vec<isize>)
    {
        println!("CMP {:?}, {:?}", left, right);

        let (ds1, ds2, p1, p2) = make_both(left, right);

        let (left, right) = _join_late(&ds1, &ds2, &[p1], &[p2], &PartialEq::eq);
        println!("BIT {:?}, {:?}", left, right);

        let mut l = _bitvector_to_pos(&left);
        let mut r =_bitvector_to_pos(&right);

        l.sort();
        r.sort();
        join_left.sort();
        join_right.sort();

        assert_eq!(l, join_left, "Left Side");
        assert_eq!(r, join_right, "Right Side");
    }

    #[test]
    fn test_join_raw() {
        _test_join(vec![1], vec![1], vec![0], vec![0]);
        _test_join(vec![1], vec![2], vec![-1, 0], vec![-1, 0]);
        _test_join(vec![1], vec![], vec![0], vec![-1]);
        _test_join(vec![1, 2, 3], vec![1, 2, 3], vec![0, 1, 2], vec![0, 1, 2]);
        _test_join(vec![1, 2, 3], vec![2, 3, 4], vec![0, 1, 2, -1], vec![-1, 0, 1, 2]);
        _test_join(vec![1, 1, 1], vec![2, 3, 4], vec![0, 1, 2, -1, -1, -1], vec![-1, -1, -1, 0, 1, 2]);
    }


    fn _test_joins(left:&Data, right:&Data, using:Join, total_cols:usize, test_file:&str)
    {
        let result = &join(left, right, using.clone(), &[0], &[0], &PartialEq::eq);
        println!("Result {:?}: {}", using, result);
        assert_eq!(result.col_count(), total_cols, "Wrong columns");

        let txt = format!("{}", result);
        let compare = open_test_file(test_file);
        assert_eq!(compare, txt, "Bad join");
    }

    #[test]
    fn test_joins() {
        let table1 =  &table_1();
        let table2 =  &table_2();
        println!("Table1 {}", table1);
        println!("Table2 {}", table2);

        _test_joins(table1, table2, Join::Inner, 6, "inner_join.txt");
    }
}