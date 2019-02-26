#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fmt;

use super::ndarray::*;
use super::types::*;
use super::relational::*;
use bit_vec::BitVec;
use std::ptr::null;

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
        self.data.col(col).into_array().into_vec()
    }

    fn rows_pos(&self, pick: Pos) -> Self {
        let data = self.data.select_cols(&pick);
        Self::new(self.schema.only(&pick), data)
    }

    fn query(self, query:&[Query]) -> Self {
        if query.len() > 0 {
            let mut next= self;
            for q in query {
                next =
                    match q {
                        Query::Where(filter) => {
                            next.find_all(filter)
                        },
                        Query::Sort(asc, pos) => {
                            next.sorted(*asc, *pos)
                        },
                        _ => unimplemented!()
                    };
            };
            next
        } else {
            self.clone()
        }
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

    fn row(&self, pos:usize) -> Col {
        unsafe {self.data.row_unchecked(pos).raw_slice().to_vec()}
    }

    fn materialize(&self, pos:&BitVec, null_count:usize) -> (Col, usize) {
        let mut data = Vec::with_capacity(self.col_count() * pos.len() - null_count);

        let positions:Vec<_> =  pos.iter()
            .enumerate()
            .filter(|(_, x)| *x).collect();

        for (found, _) in &positions {
            let mut row = self.row(*found);
            data.append(&mut row);

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
            let mut row = self.row(found);
            data.append(&mut row);

        }

        Self::from_vector(names.clone(), rows, names.len(), data)
    }

    pub fn diff(&self, with:&Self) -> Self {
        let names = self.schema();
        assert_schema(names, with.schema());

        let (pos1, null_count1) = compare_hash(self, with, false);
        let (pos2, null_count2) = compare_hash(with, self, false);

        let (mut data, rows1) = self.materialize(&pos1, null_count1);
        let (mut data2, rows2) = self.materialize(&pos2, null_count2);
        data.append(&mut data2);

        Self::from_vector(names.clone(), rows1 + rows2, names.len(), data)
    }

    pub fn union(&self, with:&Self) -> Self {
        let names = self.schema();
        assert_schema(names, with.schema());

        let data = self.data.vcat(&with.data);
        Self::new(self.schema.clone(), data)
    }

    pub fn join(&self, query:SetQuery, with:&Self) -> Self {
        match query {
            SetQuery::Union         => self.union(with),
            SetQuery::Diff          => self.diff(with),
            SetQuery::Intersection  => self.inter(with),
            _ => unimplemented!()
        }
    }

    pub fn find_all(&self, filter:&CmOp) -> Self {
        let mut data =  Vec::new();
        let mut rows = 0;

        if self.col_count() >0 {
            let value = filter.rhs.as_ref();
            let col = filter.lhs;
            let apply = filter.get_fn();

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

    pub fn sorted(&mut self, asc:bool, pos:usize) -> Self {
        self.clone()
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