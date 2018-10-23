#![allow(unused_imports)]

use super::values::*;
use super::values::DataType::*;
use super::types::*;

impl Relation for Data {
    fn empty(names:Schema) -> Self {
        Data::empty(names, Layout::Col)
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

    pub fn rel_empty() -> Data { array_empty(I32) }
    pub fn rel_nums1() -> Data {
        array(nums_1().as_slice())
    }
    pub fn rel_nums2() -> Data {
        array(nums_2().as_slice())
    }
    pub fn table_1() -> Data {
        let fields = [field("one", I32), field("two", I32), field("three", Bool)].to_vec();
        let schema = Schema::new(fields);
        let c1 = encode(nums_1().as_slice());
        let c2 = encode(nums_2().as_slice());
        let c3 = encode(bools_1().as_slice());

        table_cols::<Data>(schema, &[c1, c2, c3])
    }

    #[test]
    fn test_create() {
        let num1 = nums_1();
        let emptySchema = &Schema::scalar_field(I32);

        let fnull = rel_empty();
        assert_eq!(fnull.names(), emptySchema);
        assert_eq!(fnull.layout(), &Layout::Col);
        println!("Empty {:?}", fnull);

        assert_eq!(fnull.col_count(), 0);
        assert_eq!(fnull.row_count(), 0);

        let fcol1 = rel_nums1();
        println!("Array {}", fcol1);
        assert_eq!(fnull.names(), emptySchema);
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
        let query_empty = Data::select(table1, &[]);
        println!("Select 0 {}", query_empty);
        assert_eq!(query_empty.names.len(), 0);

        let query1 = Data::select(table1, &[pick1]);
        println!("Select 0 {}", query1);
        assert_eq!(query1.names.len(), 1);
        let query2 = Data::select(table1, &[pick2.clone()]);
        println!("Select 1 {}", query2);
        assert_eq!(query2.names.len(), 1);

        let query3 = Data::deselect(table1, &[pick2, pick3]);
        println!("DeSelect 1 {}", query3);
        assert_eq!(query3.names.len(), 1);
        assert_eq!(query1.names, query3.names);
    }
}