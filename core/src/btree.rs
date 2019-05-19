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

    fn from_vector(schema:Schema, vector:Vec<Col>) -> Self {
        let mut data:Tree =  BTreeMap::new();

        for row in vector {
            data.insert(row[0].clone(), row[1].clone());
        }

        Self::new(schema, data)
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

    fn filter(self, query:&CmOp) -> Self {
        let range = self.query_range(query);
        let mut tree = BTreeMap::new();

        for (key, value) in range {
            tree.insert(key.clone(), value.clone());
        }
        Self::new(self.schema.clone(), tree)
    }

    fn sorted(self, asc:bool, pos:usize) -> Self {
        if pos == 0 {
            if asc {
                self
            } else {
                self
            }
        } else {
            self
        }
    }

    fn union(self, other:Self) -> Self {
        let names = self.schema();
        assert_schema(names, other.schema());
        let mut data =self.data.clone();
        let mut x = other.data;
        data.append(&mut x);
        Self::new(self.schema.clone(), data)
    }
//
//    fn find_all_rows(&self, col:usize, value:&Scalar, apply: &BoolExpr ) -> Self
//    {
//        let mut data =  BTreeMap::new();
//        if col == 0 {
//            if let Some(v) = self.data.get(value) {
//                if apply(value, value) {
//                    data.insert(value.clone(), v.clone());
//                }
//            }
//        } else {
//            for old in self.data.values() {
//                if apply(old, value) {
//                    data.insert(old.clone(), value.clone());
//                    break;
//                }
//            }
//        }
//
//        Self::new(self.schema.clone(), data)
//    }
//
//    fn union<T:Relation>(&self, to:&T) -> Self {
//        let mut data = self.data.clone();
//
//        for row in to.rows_iter() {
//            data.insert(row[0].clone(), row[1].clone());
//        }
//
//        Self::new(self.schema.clone(), data)
//    }
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

    pub fn pick_col(&self, pos:&usize) -> KeyValue {
        assert!(pos > &0 || pos < &2, "BTree must have a schema of 0 to 2 fields");

        if pos == &1 {
            KeyValue::Key
        } else {
            KeyValue::Value
        }
    }

    pub fn query_range(&self, query:&CmOp) -> BRange<Scalar, Scalar> {
        let value = query.rhs.as_ref();

        match query.op {
            CompareOp::Eq => {
                self.data.range(value..value)
            },
            CompareOp::Greater => {
                self.data.range(value..)
            }
            _ => unimplemented!()
        }
    }
}

impl fmt::Display for BTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_rows(Self::type_name(), self, f)
    }
}