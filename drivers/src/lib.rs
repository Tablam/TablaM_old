extern crate rusqlite;

use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub enum DbErr {
    EngineError(String),
    Connection,
    Query,
}

#[derive(Debug, Clone)]
pub enum Scalar {
    None,
    BOOL(bool),
    I64(i64),
    UTF8(String),
}

type DbRow = HashMap<String, Scalar>;
type RowResult = Result<Vec<DbRow>, DbErr>;

pub trait Rdbms {
    fn select(&self, sql:&str, params:Option<DbRow>) -> RowResult;
}

mod sqlite;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
