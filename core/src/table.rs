#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fmt;

use super::ndarray::*;
use super::types::*;
use super::relational::*;

impl Relation for Data {
    fn type_name<'a>() -> &'a str { "Rel" }
    fn new_from<R: Relation>(names: Schema, of: &R) -> Self {
        let mut data = Vec::with_capacity(of.len());

        for row in of.rows_iter() {
            let mut new_row = row.to_vec();
            data.append(&mut new_row);
        }

        Self::from_vector(names, of.row_count(), of.col_count(), data)
    }

    fn from_vector(schema:Schema, rows:usize, cols:usize, vector:Col) -> Self {
        Self::new(schema, NDArray::new(rows, cols, vector))
    }

    fn to_ndarray(&self) -> NDArray {
        self.data.clone()
    }

    fn clone_schema(&self, schema:&Schema) -> Self {
        let schema = schema.clone();
        Self::new(schema, self.data.clone())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn row_count(&self) -> usize { self.data.rows }

    fn col_count(&self) -> usize {
        self.data.cols
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        self.data.get([row, col]).unwrap()
    }
    fn get_value(&self, row:usize, col:usize) -> Option<&Scalar> {
        self.data.get([row, col])
    }
    fn get_row(&self, pos:usize) -> Option<Col> {
        if pos < self.row_count() {
            let value = unsafe {self.data.row_unchecked(pos).raw_slice().to_vec()};

            Some(value)
        } else {
            None
        }
    }

    fn rows_iter(&self) -> RelIter<'_, Self> {
        RelIter {
            pos: 0,
            rel: self,
        }
    }

    fn col_iter(&self, col: usize) -> ColIter<'_, Self> {
        ColIter {
            pos: 0,
            col,
            rel:self
        }
    }

    fn col(&self, col: usize) -> Col {
        self.data.col(col).into_array().into_vec()
    }

    fn rows_pos(&self, pick: Pos) -> Self {
        let data = self.data.select_cols(&pick);
        Self::new(self.schema.only(&pick), data)
    }

    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Self
    {
        let mut data =  Vec::new();
        let mut rows = 0;

        if self.col_count() >0 {
            for row in self.rows_iter() {
                let old = &row[col];
                if apply(old, value) {
                    let mut new_row = row.to_vec();
                    data.append(&mut new_row);
                    rows += 1;
                }
            }
        }

        Self::from_vector(self.schema.clone(), rows, self.col_count(), data)
    }

    fn union<T:Relation>(&self, to:&T) -> Self {
        let data = self.data.vcat(&to.to_ndarray());

        Self::new(self.schema.clone(), data)
    }
}

impl Data {
    pub fn empty(schema:Schema) -> Self {
        let cols = schema.len();
        Data {
            schema,
            data: NDArray::new(0, cols, vec![]),
        }
    }

    pub fn new(schema:Schema, data:NDArray) -> Self {
        assert_eq!(schema.len(), data.cols);

        Data {
            schema,
            data,
        }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_rows(Self::type_name(),self, f)
    }
}