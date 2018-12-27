use super::*;
use rusqlite::{Connection, Error};
use rusqlite::types::Value;

struct SqliteDb {
    conn:Connection,
}

impl From<Error> for DbErr {
    fn from(err: Error) -> Self {
        DbErr::EngineError(format!("{}", err))
    }
}

impl From<Value> for Scalar {
    fn from(value: Value) -> Self {
        match value {
            Value::Null     => Scalar::None,
            Value::Integer(x) => Scalar::I64(x),
            Value::Real(x)  => Scalar::F64(x),
            Value::Text(x)  => Scalar::UTF8(x),
            Value::Blob(x)  => Scalar::Blob(x),
        }
    }
}

fn select(of:&SqliteDb, sql:&str, _params:&Option<DbRow>) -> RowResult {
    let mut stmt = of.conn.prepare(sql)?;

    let names = stmt.column_names().into_iter().map(String::from).collect::<Vec<_>>();
    let mut rows = stmt.query(&[]).unwrap();

    let mut values = Vec::new();
    while let Some(row) = rows.next() {
        let row = row.unwrap();
        let mut d = DbRow::new();

        for name in names.iter() {
            let value = Scalar::from(row.get::<_, Value>(name.as_ref()));
            d.insert(name.clone(), value);
        }

        values.push(d);
    }
    Ok(values)
}

impl Rdbms for SqliteDb {
    fn select(&self, sql:&str, params:Option<DbRow>) -> RowResult {
        select(&self, sql, &params)
    }
}

impl SqliteDb {
    fn open_memory() -> Result<Connection, DbErr> {
        let con  = Connection::open_in_memory()?;

        Ok(con)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries() {
        let con = SqliteDb::open_memory().unwrap();

        let db: SqliteDb = SqliteDb {
            conn:con
        };

        let rows = db.select("SELECT 1", None).unwrap();

        println!("Row0:  {:?}", rows);
    }
}
