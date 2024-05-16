use dmdb::{params, Connection};

const INIT_SQL: &'static str = r#"
DROP TABLE IF EXISTS dmdb_test;

CREATE TABLE dmdb_test (
    id INTEGER PRIMARY KEY IDENTITY(1,1),
    a INTEGER
);
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::connect("127.0.0.1:5236", "SYSDBA", "SYSDBA001")?;

    // Init
    for sql in INIT_SQL.split(";") {
        if !sql.trim().is_empty() {
            conn.execute(sql, [])?;
        }
    }

    // Insert 1 ok
    {
        let tx = conn.transaction()?;
        tx.execute("INSERT INTO dmdb_test (a) VALUES (?)", params![1])?;
        assert_eq!(tx.last_insert_id()?, 1);
        tx.commit()?;
    }

    // Insert 2 fail
    {
        let tx = conn.transaction()?;
        tx.execute("INSERT INTO dmdb_test (a) VALUES (?)", params![2])?;
        assert_eq!(tx.last_insert_id()?, 2);
        tx.rollback()?;
    }

    // Insert 3 fail
    {
        let tx = conn.transaction()?;
        tx.execute("INSERT INTO dmdb_test (a) VALUES (?)", params![3])?;
        assert_eq!(tx.last_insert_id()?, 3);
    }

    // Insert 4 ok
    {
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO dmdb_test (a) VALUES (?)")?;
        stmt.execute(params![4])?;
        assert_eq!(tx.last_insert_id()?, 4);
        drop(stmt);
        tx.commit()?;
    }

    // Insert 5 fail
    {
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO dmdb_test (a) VALUES (?)")?;
        stmt.execute(params![5])?;
        assert_eq!(tx.last_insert_id()?, 5);
        drop(stmt);
    }

    let mut stmt = conn.prepare("SELECT a FROM dmdb_test")?;
    let mut rows = stmt.query([])?;
    let mut values = vec![];
    while let Some(row) = rows.next()? {
        values.push(row.get::<u32>(1)?);
    }

    assert_eq!(values, vec![1, 4]);

    Ok(())
}
