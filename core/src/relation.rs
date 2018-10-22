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

    fn row(self, pos:usize) -> Col {
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

    #[test]
    fn test_create() {
        let num1 = nums_1();
        let emptySchema = &Schema::scalar_field(I32);

        let fnull = &array_empty(I32);
        assert_eq!(fnull.names(), emptySchema);
        assert_eq!(fnull.layout(), &Layout::Col);
        println!("Empty {:?}", fnull);

        assert_eq!(fnull.col_count(), 0);
        assert_eq!(fnull.row_count(), 0);

        let fcol1 = array(num1.as_slice());
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

    }

}