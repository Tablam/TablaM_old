#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fmt;

use super::types::*;
use super::relational::*;
use bit_vec::BitVec;
use std::ptr::null;
use crate::dsl::row;

impl Relation for Table {
    fn type_name<'a>() -> &'a str { "Rel" }
    fn new_from<R: Relation>(names: Schema, of: &R) -> Self {
        let mut data = Vec::with_capacity(of.len());

        for row in of.rows_iter() {
            let new_row = row.to_vec();
            data.push(new_row);
        }

        Self::from_vector(names,data)
    }

    fn from_vector(schema:Schema, vector:Vec<Col>) -> Self {
        Self::new(schema, vector)
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

    fn col_count(&self) -> usize {
       self.count
    }

    fn value(&self, row:usize, col:usize) -> &Scalar {
        &self.data[row][col]
    }

    fn get_value(&self, row:usize, col:usize) -> Option<&Scalar> {
        if let Some(row) = self.data.get(row) {
            row.get(col)
        } else {
            None
        }
    }

    fn get_row(&self, pos:usize) -> Option<Col> {
        if pos < self.row_count() {
            let value = self.row(pos);

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
        self.data[col].to_vec()
    }

    fn rows_pos(&self, pick: Pos) -> Self {
        let schema = self.schema.only(&pick);
        let data = pick.into_iter().map(|x| self.row(x)).collect();
        Self::new(schema, data)
    }

    fn filter(self, query:&CmOp) -> Self {
        self.find_all(query)
    }

    fn sorted(self, asc:bool, pos:usize) -> Self {
        self.clone()
    }

    fn union(self, other:Self) -> Self {
        let names = self.schema();
        assert_schema(names, other.schema());

        let data = self.data.into_iter().chain(other.data.into_iter()).collect();
        Self::new(self.schema.clone(), data)
    }
}

impl Table {
    pub fn empty(schema:Schema) -> Self {
        let count = schema.len();
        Table {
            schema,
            count,
            data: vec![],
        }
    }

    pub fn row(&self, pos:usize) -> Col {
       self.data[pos].to_vec()
    }

    fn materialize(&self, pos:&BitVec, null_count:usize) -> (Vec<Col>, usize) {
        let mut data = Vec::with_capacity(self.col_count() * pos.len() - null_count);

        let positions:Vec<_> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x).collect();

        for (found, _) in &positions {
            let row = self.row(*found);
            data.push(row);

        }
        (data, positions.len())
    }

    pub fn inter(&self, with:&Self) -> Self {
        let names = self.schema();
        assert_schema(names, with.schema());

        let (pos, null_count) = compare_hash(self, with, true);

        let (mut data, rows) = self.materialize(&pos, null_count);

        let positions:Vec<_> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x).collect();

        for (found, _) in positions {
            let row = self.row(found);
            data.push(row);

        }

        Self::from_vector(names.clone(),data)
    }

    pub fn diff(&self, with:&Self) -> Self {
        let names = self.schema();
        assert_schema(names, with.schema());

        let (pos1, null_count1) = compare_hash(self, with, false);
        let (pos2, null_count2) = compare_hash(with, self, false);

        let (mut data, rows1) = self.materialize(&pos1, null_count1);
        let (mut data2, rows2) = self.materialize(&pos2, null_count2);
        data.append(&mut data2);

        Self::from_vector(names.clone(), data)
    }

    pub fn find_all(&self, filter:&CmOp) -> Self {
        let mut data =  Vec::new();
        let mut rows = 0;

        if self.col_count() > 0 {
            let value = filter.rhs.as_ref();
            let col = filter.lhs;
            let apply = filter.get_fn();

            for row in self.rows_iter() {
                let old = &row[col];

                if apply(old, value) {
                    let new_row = row.to_vec();
                    data.push(new_row);
                    rows += 1;
                }
            }
        }

        Self::from_vector(self.schema.clone(), data)
    }

    pub fn new(schema:Schema, data:Vec<Col>) -> Self {
        let (rows, count) = size_rel(&data);
        let columns = schema.len();
        if rows > 0 {
            assert_eq!(columns, count, "Table data must be of equal columns to schema");
        }

        Table {
            schema,
            count: columns,
            data,
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_rows(Self::type_name(),self, f)
    }
}