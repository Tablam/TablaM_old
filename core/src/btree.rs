#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fmt;
use std::collections::BTreeMap;
use std::collections::btree_map::Range as BRange;

use super::types::*;
use super::relational::*;

impl Relation for BTree {
    fn type_name<'a>() -> &'a str { "Btree" }
    fn new_from<R: Relation>(names: Schema, of: &R) -> Self {
        let mut data:Tree =  BTreeMap::new();

        if of.col_count() > 0 {
            if of.col_count() == 2 {
                for row in of.rows_iter() {
                    data.insert(row[0].clone(), row[1].clone());
                }
            } else {
                for row in of.rows_iter() {
                    data.insert(row[0].clone(), Scalar::None);
                }
            }
        }

        Self::new(names, data)
    }

    fn from_vector(schema:Schema, rows:usize, cols:usize, vector:Col) -> Self {
        let mut data:Tree =  BTreeMap::new();

        if !vector.is_empty() {
            for row in 0..rows {
                let mut new_row = Vec::with_capacity(rows);

                for col in 0..cols {
                    new_row.push(vector[index(cols, rows, row, col)].clone())
                }

                data.insert(new_row[0].clone(), new_row[1].clone());
            }
        }

        Self::new(schema, data)
    }

    fn to_ndarray(&self) -> NDArray {
        let mut data = Vec::with_capacity(self.len());

        for (key, row) in &self.data {
            data.push(key.clone());
            data.push(row.clone());
        }
        NDArray::new(self.row_count(), self.col_count(), data)
    }

    fn clone_schema(&self, schema:&Schema) -> Self {
        let schema = schema.clone();
        Self::new(schema, self.data.clone())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn row_count(&self) -> usize {
        self.data.len()
    }
    fn col_count(&self) -> usize { self.schema.len()}

    fn value(&self, row:usize, col:usize) -> &Scalar {
        let (key, value) = self.data.iter().nth(row).unwrap();
        if col == 0 {
            key
        } else {
            value
        }
    }
    fn get_value(&self, row:usize, col:usize) -> Option<&Scalar> {
        if self.row_count() <= row && self.row_count() <= col {
            return Some(self.value(row, col))
        }
        None
    }

    fn get_row(&self, pos:usize) -> Option<Col> {
        if pos < self.row_count() {
            let data =  &self.data;
            let (key, value) = data.into_iter().nth(pos).unwrap();

            Some(vec![key.clone(), value.clone()])
        } else {
            None
        }
    }

    fn rows_iter(&self) -> RelIter<'_, Self> {
        self.to_iter()
    }

    fn col_iter(&self, col: usize) -> ColIter<'_, Self> {
        ColIter {
            pos: 0,
            col,
            rel:self
        }
    }

    fn col(&self, col: usize) -> Col {
        if col == 0 {
            self.data.keys().cloned().collect()
        } else {
            self.data.values().cloned().collect()
        }
    }

    fn rows_pos(&self, pick: Pos) -> Self {
        let mut data =  BTreeMap::new();
        let count = pick.len();

        if count <= 2 {
            data.clone_from(&self.data);
        }

        Self::new(self.schema.only(&pick), data)
    }

    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Self
    {
        let mut data =  BTreeMap::new();
        if col == 0 {
            if let Some(v) = self.data.get(value) {
                if apply(value, value) {
                    data.insert(value.clone(), v.clone());
                }
            }
        } else {
            for old in self.data.values() {
                if apply(old, value) {
                    data.insert(old.clone(), value.clone());
                    break;
                }
            }
        }

        Self::new(self.schema.clone(), data)
    }

    fn union<T:Relation>(&self, to:&T) -> Self {
        let mut data = self.data.clone();

        for row in to.rows_iter() {
            data.insert(row[0].clone(), row[1].clone());
        }

        Self::new(self.schema.clone(), data)
    }
}

//TODO: https://newspaint.wordpress.com/2016/07/05/implementing-custom-sort-for-btree-in-rust/
impl BTree {
    pub fn empty(schema:Schema) -> Self {
        let data = BTreeMap::new();
        Self::new(schema, data)
    }

    pub fn new(schema:Schema, data:Tree) -> Self {
        assert!(schema.len() <= 2, "BTree must have a schema of 0 to 2 fields");

        BTree {
            schema,
            data,
        }
    }

    pub fn to_iter(&self) -> RelIter<'_, Self> {
        RelIter {
            pos: 0,
            rel: self,
        }
    }

    pub fn query_range<'a>(op:&'a BTree, query:Option<CmOp>) -> BRange<'a, Scalar, Scalar> {
        match query {
            None => op.data.range(..),
            Some(cmp) => {
                let value = cmp.rhs.as_ref();

                match cmp.op {
                    CompareOp::Eq => {
                        op.data.range(value..value)
                    },
                    CompareOp::Greater => {
                        op.data.range(value..)
                    }
                    _ => unimplemented!()
                }
            }
        }
    }
}

impl fmt::Display for BTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_rows(Self::type_name(), self, f)
    }
}