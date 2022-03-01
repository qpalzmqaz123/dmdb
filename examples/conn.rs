use dmdb::{params, Connection, Value};

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
    h NUMBER(10, 2),
    i BIT,
    j CHAR(2),
    k VARCHAR(10),
    l CHARACTER(3),
    m VARCHAR2(10),
    n DECIMAL(10, 2),
    o FLOAT(2),
    p DOUBLE(2),
    q DOUBLE,
    r DOUBLE PRECISION(2),
    s TEXT,
    t CLOB,
    u DATETIME
);
"#;

#[derive(Debug, PartialEq)]
struct Test {
    a: i32,
    b: i32,
    c: i64,
    d: i8,
    e: i8,
    f: i16,
    g: i32,
    h: f64,
    i: bool,
    j: String,
    k: String,
    l: String,
    m: String,
    n: f64,
    o: f32,
    p: f64,
    q: f64,
    r: f64,
    s: String,
    t: String,
    u: Value,
}

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
        "INSERT INTO dmdb_test (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?;
    #[rustfmt::skip]
    stmt.execute(params![
        1, 2, 3, 4, 5, 6, 7, 8.1, true, "jj", "kkk", "ll", "m", 13.1, 14.1, 15.1, 16.1, 17.1, "s", "t", Value::DateTime(2021, 3, 1, 15, 38, 0, 123456)
    ])?;

    // Get
    let tuple = conn.query_row(
        "SELECT a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u FROM dmdb_test",
        [],
        |row| {
            Ok(Test {
                a: row.get(1)?,
                b: row.get(2)?,
                c: row.get(3)?,
                d: row.get(4)?,
                e: row.get(5)?,
                f: row.get(6)?,
                g: row.get(7)?,
                h: row.get(8)?,
                i: row.get(9)?,
                j: row.get(10)?,
                k: row.get(11)?,
                l: row.get(12)?,
                m: row.get(13)?,
                n: row.get(14)?,
                o: row.get(15)?,
                p: row.get(16)?,
                q: row.get(17)?,
                r: row.get(18)?,
                s: row.get(19)?,
                t: row.get(20)?,
                u: row.get_value(21)?,
            })
        },
    )?;

    // Check
    assert_eq!(
        tuple,
        Test {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            g: 7,
            h: 8.1,
            i: true,
            j: "jj".into(),
            k: "kkk".into(),
            l: "ll ".into(),
            m: "m".into(),
            n: 13.1,
            o: 14.1,
            p: 15.1,
            q: 16.1,
            r: 17.1,
            s: "s".into(),
            t: "t".into(),
            u: Value::DateTime(2021, 3, 1, 15, 38, 0, 123456),
        }
    );

    Ok(())
}
