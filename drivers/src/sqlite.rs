use super::*;
use rusqlite::{Connection, Error};

struct SqliteDb {
    conn:Connection,
}

impl From<Error> for DbErr {
    fn from(err: Error) -> Self {
        DbErr::EngineError(format!("{}", err))
    }
}

fn select(of:&SqliteDb, sql:&str, params:Option<DbRow>) -> RowResult {
    let mut stmt = of.conn.prepare(sql)?;

    let result = stmt.query_map(&[], |row| {
        let mut result = DbRow::new();

        result.insert("1".to_string(), Scalar::None);

        result
    })?;

    let mut rows = Vec::new();
    for name_result in result {
        rows.push(name_result?);
    }

    Ok(rows)
}

impl Rdbms for SqliteDb {
    fn select(&self, sql:&str, params:Option<DbRow>) -> RowResult {
        select(&self, sql, params)
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
