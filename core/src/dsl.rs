#![allow(unused_imports)]
extern crate bit_vec;
use self::bit_vec::BitVec;

use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use super::types::*;

pub fn decode<T: From<Scalar>>(values: &[Scalar]) -> Vec<T> {
    values.iter().map(move |x| From::from(x.clone())).collect()
}

pub fn to_columns(of: Col) -> Vec<Col> {
    let mut rows = Vec::with_capacity(of.len());
    for x in of {
        rows.push(vec![x]);
    }
    rows
}

pub fn field(name: &str, kind: DataType) -> Field {
    Field::new(name, kind)
}

pub fn schema(names: &[(&str, DataType)]) -> Schema {
    let fields = names
        .into_iter()
        .map(|(name, kind)| Field::new(name, *kind))
        .collect();

    Schema::new(fields)
}

pub fn schema_single(name: &str, kind: DataType) -> Schema {
    Schema::new_single(name, kind)
}
pub fn schema_it(kind: DataType) -> Schema {
    schema_single("it", kind)
}

pub fn schema_build(names: &[(&str, DataType)]) -> Schema {
    let fields = names
        .into_iter()
        .map(|(name, kind)| Field::new(name, *kind))
        .collect();

    Schema::new(fields)
}

pub fn schema_kv(types: [DataType; 2]) -> Schema {
    let key = field("key", types[0]);
    let value = field("value", types[1]);

    Schema::new(vec![key, value])
}

pub fn colp(pos: usize) -> ColumnName {
    ColumnName::Pos(pos)
}
pub fn coln(name: &str) -> ColumnName {
    ColumnName::Name(name.to_string())
}

pub fn infer_type(of: &Col) -> DataType {
    if of.is_empty() {
        DataType::None
    } else {
        of[0].kind()
    }
}

pub fn infer_types(of: &Col) -> Vec<DataType> {
    of.iter().map(|x| x.kind()).collect()
}

pub fn value<T>(x: T) -> Scalar
where
    T: From<Scalar>,
    Scalar: From<T>,
{
    Scalar::from(x)
}

pub fn rvalue<T>(x: T) -> Rc<Scalar>
where
    T: From<Scalar>,
    Scalar: From<T>,
{
    Rc::new(Scalar::from(x))
}

pub fn int(x: i32) -> Scalar {
    value::<i32>(x)
}

pub fn int64(x: i64) -> Scalar {
    value::<i64>(x)
}

pub fn bool(x: bool) -> Scalar {
    value::<bool>(x)
}

pub fn str(x: &str) -> Scalar {
    value::<String>(x.to_string())
}

pub fn col<T>(x: &[T]) -> Vec<Scalar>
where
    T: From<Scalar>,
    Scalar: From<T>,
    T: Clone,
{
    x.iter().map(|v| value(v.clone())).collect()
}

pub fn rcol_t<T>(name: &str, kind: DataType, of: &[T]) -> Vector
where
    T: From<Scalar>,
    Scalar: From<T>,
    T: Clone,
{
    let data = col(of);

    Vector::new(schema_single(name, kind), data)
}

pub fn rcol<T>(name: &str, of: &[T]) -> Vector
where
    T: From<Scalar>,
    Scalar: From<T>,
    T: Clone,
{
    let data = col(of);
    let kind = infer_type(&data);

    Vector::new(schema_single(name, kind), data)
}

pub fn array<T>(of: &[T]) -> Vector
where
    T: From<Scalar>,
    Scalar: From<T>,
    T: Clone,
{
    rcol("it", of)
}

pub fn array_empty(kind: DataType) -> Vector {
    Vector::empty(kind)
}

pub fn reverse(of: Col) -> Col {
    let mut col = of.clone();
    col.reverse();
    col
}

pub fn none() -> Scalar {
    Scalar::default()
}

pub fn vector(of: &[Scalar]) -> Vector {
    Vector::new_scalars(of)
}
