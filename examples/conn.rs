use dmdb::{params, Connection};

const INIT_SQL: &'static str = r#"
DROP TABLE IF EXISTS dmdb_test;

CREATE TABLE dmdb_test (
    id INTEGER PRIMARY KEY IDENTITY(1,1),
    a INTEGER,
    b INT,
    c BIGINT,
    d TINYINT,
    e BYTE,
    f SMALLINT,
    g NUMBER(10, 0),
    h NUMBER(10, 2)
);
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect("127.0.0.1:5236", "SYSDBA", "SYSDBA")?;

    // Init
    for sql in INIT_SQL.split(";") {
        if !sql.trim().is_empty() {
            conn.execute(sql, [])?;
        }
    }

    // Insert
    let mut stmt = conn.prepare(
        "INSERT INTO dmdb_test (a, b, c, d, e, f, g, h) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )?;
    stmt.execute(params![1, 2, 3, 4, 5, 6, 7, 8.1])?;

    // Get
    let tuple = conn.query_row("SELECT a, b, c, d, e, f, g, h FROM dmdb_test", [], |row| {
        Ok((
            row.get::<i32>(1)?,
            row.get::<i32>(2)?,
            row.get::<i64>(3)?,
            row.get::<i8>(4)?,
            row.get::<i8>(5)?,
            row.get::<i16>(6)?,
            row.get::<i32>(7)?,
            row.get::<f64>(8)?,
        ))
    })?;

    // Check
    assert_eq!(tuple, (1, 2, 3, 4, 5, 6, 7, 8.1));

    Ok(())
}
