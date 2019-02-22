#![allow(unused_imports)]
use std::fmt;
use std::hash::{Hash, Hasher};

use super::ndarray::*;
use super::types::*;
use super::dsl::*;
//
//impl Rows for Vec<Scalar>
//{
//    fn next (&mut self) -> Option<Scalar> {
//        Some(self[0].clone())
//    }
//    fn row_count(&self) -> usize {1}
//    fn col_count(&self) -> usize {1}
//    fn get_row(&self, _pos:usize) -> Option<Col> { Some([].to_vec())}
//}
//
//impl <'a> Rel <'a> {
//    pub fn exact_schema_size(self) -> Option<usize> {
//        match self.data {
//            RScalar::BTree(_) => Some(2),
//            RScalar::Range(_) => Some(1),
//            _ => None,
//        }
//    }
//
//    pub fn max_schema_size(self) -> usize {
//        match self.data {
//            RScalar::BTree(_) => 2,
//            RScalar::Range(_) => 1,
//            _ => std::usize::MAX,
//        }
//    }
//
//
//    pub fn empty(schema:Schema, kind:DataType) -> Self {
//        match kind {
//            DataType::Rows => {
//                let data = RScalar::Rows(NDArray::new(0, 0, [].to_vec()));
//                Rel {
//                    schema,
//                    data
//                }
//            },
//            _ => unreachable!(),
//        }
//    }
//
//    fn schema(&self) -> &Schema {
//        &self.schema
//    }
//
//    fn len(&self) -> usize {
//        match &self.data {
//            RScalar::Rows(data) => data.rows * data.cols,
//            _ => 1,
//        }
//    }
//
//    fn row_count(&self) -> usize {
//        match &self.data {
//            RScalar::Rows(data) => data.rows,
//            RScalar::BTree(data) => data.row_count(),
//            RScalar::Range(data) => (data.end - data.start) as usize,
//            RScalar::Iter(data) => data.data.row_count(),
//        }
//    }
//
//    fn col_count(&self) -> usize {
//        match &self.data {
//            RScalar::Rows(data) => data.cols,
//            RScalar::BTree(data) => data.col_count(),
//            RScalar::Range(_) => 1,
//            RScalar::Iter(data) => data.data.col_count(),
//        }
//    }
//
//    fn is_empty(&self) -> bool { self.len() == 0}
//
//    fn from_vector(schema:Schema, rows:usize, cols:usize, vector:Col) -> Self {
//        match vector.len() {
//            0 => Self::empty(schema, DataType::Rows),
//            _ => {
//                let data = RScalar::Rows(NDArray::new(rows, cols, vector));
//
//                Rel {
//                    schema: schema.clone(),
//                    data
//                }
//            },
//        }
//    }
//
//    fn to_ndarray(&self) -> NDArray {
//        match &self.data {
//            RScalar::Rows(data) => data.clone(),
//            _ => NDArray::new(1, 1, [].to_vec()),
//        }
//    }
//
//    fn to_array(&self) -> Col {
//        let rows = self.row_count();
//        let cols = self.col_count();
//
//        let mut array = Vec::with_capacity(cols * rows);
////
////        match &self.data {
////            RScalar::Rows(data) => data.into_vec(),
////            RScalar::BTree(data) => data.,
////            RScalar::Range(mut data) => data.read(row, col),
////            RScalar::Iter(mut data) => data.read(row, col),
////        }
////
////        for mut value in self.data.rows_iter() {
////            data.append(&mut value);
////        }
//
//        array
//    }
//
//    fn value(&self, row:usize, col:usize) -> &Scalar {
//        &Scalar::None
//    }
//
//
//    fn get_row(&self, pos:usize) -> Option<Col> {
//        match &self.data {
//            RScalar::Rows(data) =>  {
//                if pos < data.rows {
//                    let value = unsafe {data.row_unchecked(pos).raw_slice().to_vec()};
//
//                    Some(value)
//                } else {
//                    None
//                }
//            },
//            RScalar::BTree(data) => data.get_row(pos),
//            RScalar::Range(data) => Some(vec![data.start.into()]),
//            RScalar::Iter(data) => data.data.get_row(pos),
//        }
//    }
//
//    fn rows_iter(&self) -> RelIter<'_, Self> {
//        RelIter {
//            pos: 0,
//            rel: self,
//        }
//    }
//
//    fn col_iter(&self, col: usize) -> ColIter<'_, Self> {
//        ColIter {
//            pos: 0,
//            col,
//            rel:self
//        }
//    }
//
////    fn rows_pos(&self, pick: Pos) -> Self;
////
////    fn cmp_cols(&self, row: usize, cols: &[usize], tuple: &[RScalar], apply: &BoolExpr) -> bool
////    {
////        let values = cols.iter().zip(tuple.iter());
////
////        for (col, value) in values {
////            let old = self.value(row, *col);
////            if !apply(old, value) {
////                return false;
////            }
////        }
////        true
////    }
////
//    fn find(&self, row:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Option<usize>
//    {
//        if apply(value, self.value(row, col)) {
//            Some(row)
//        } else {
//            None
//        }
//    }
//
//    fn find_all(&self, row:usize, col:usize, value:&Scalar, apply: &BoolExpr ) -> Vec<usize>
//    {
//        let mut pos = Vec::new();
//
//        for next in row..self.row_count() {
//            if apply(value, self.value(next, col)) {
//                pos.push(next);
//            }
//        }
//
//        pos
//    }
//
////    fn row_only(&self, row: usize, cols: &[usize]) -> Col {
////        let mut data = Vec::with_capacity(cols.len());
////
////        for row in self.rows_iter() {
////            for i in cols {
////                data.push(row[*i]);
////            }
////        }
////        data
////    }
////
////    fn materialize_raw(&self, pos:&BitVec, null_count:usize, keep_null:bool) -> Col {
////        let rows = pos.len();
////        let cols = self.col_count();
////        let total_rows = if keep_null {rows} else {rows - null_count};
////
////        let mut data = vec![RScalar::None; cols * total_rows];
////        println!("Raw r:{:?}", pos);
////
////        let positions:Vec<(usize, bool)> =  pos.iter()
////            .enumerate()
////            .filter(|(_, x)| *x || keep_null).collect();
////        println!("Raw r:{:?}", positions);
////
////        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());
////
////        for (new_row, (row, found)) in positions.into_iter().enumerate() {
////            for col in 0..cols {
////                let _pos = index( cols, total_rows, new_row, col);
////                if found {
////                    data[_pos] = self.value(row, col).clone();
////                }
////            }
////        }
////
////        data
////    }
////
////    fn materialize_data(&self, pos:&BitVec, keep_null:bool) -> NDArray {
////        let rows = pos.len();
////        let cols = self.col_count();
////        let positions:Vec<(usize, bool)> =  pos.iter()
////            .enumerate()
////            .filter(|(_, x)| *x || keep_null).collect();
////        println!("Raw rpos:{:?}", positions);
////
////        let total_rows = if keep_null {rows} else { positions.len()};
////
////        let mut data = vec![RScalar::None; cols * total_rows];
////        println!("Raw r:{:?}", pos);
////
////        println!("Raw r:{} c:{} n:{} total: {} {}", rows, cols, keep_null, total_rows, positions.len());
////
////        for (new_row, (row, found)) in positions.into_iter().enumerate() {
////            for col in 0..cols {
////                if found {
////                    let _pos = index(cols, total_rows, new_row, col);
////                    data[_pos] = self.value(row, col).clone();
////                }
////            }
////        }
////
////        NDArray::new(total_rows, cols, data)
////    }
////
////    fn find_all_rows(&self, col:usize, value:&RScalar, apply: &BoolExpr ) -> Self;
////
////
////    fn union<T:Relation>(&self, to:&T) -> Self;
//}