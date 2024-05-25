use dmdb::{params, Connection};

const INIT_SQL: &'static str = r#"
DROP TABLE IF EXISTS dmdb_test;

CREATE TABLE dmdb_test (
    id INTEGER PRIMARY KEY IDENTITY(1,1),
    nil INTEGER,
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
    u DATETIME,
    v BLOB
);
"#;

#[derive(Debug, PartialEq)]
struct Test {
    id: u64,
    nil: Option<i32>,
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
    u: (u16, u8, u8, u8, u8, u8, u32),
    v: Vec<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::connect("127.0.0.1:5236", "SYSDBA", "SYSDBA001")?;

    let text_data = (0..1000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let blob_data: Vec<u8> = (0..8000).map(|v| v as u8).collect();

    // Init
    for sql in INIT_SQL.split(";") {
        if !sql.trim().is_empty() {
            conn.execute(sql, [])?;
        }
    }

    // Insert
    let mut stmt = conn.prepare(
        "INSERT INTO dmdb_test (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )?;
    #[rustfmt::skip]
    stmt.execute(params![
        1, 2, 3, 4, 5, 6, 7, 8.1, true, "jj", "kkk中文", "ll", "m", 13.1, 14.1, 15.1, 16.1, 17.1, text_data, "t", (2021u16, 3u8, 1u8, 15u8, 38u8, 0u8, 123456u32), blob_data,
    ])?;
    drop(stmt);
    let id = conn.ident_current(&"dmdb_test".to_uppercase())?;
    assert_eq!(id, 1);
    let id = conn.last_insert_id()?;
    assert_eq!(id, 1);

    // Get
    let tuple = conn.query_row(
        "SELECT id, nil, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v FROM dmdb_test",
        [],
        |row| {
            println!("cols: {:?}", row.columns());

            Ok(Test {
                id: row.get(1)?,
                nil: row.get(2)?,
                a: row.get(3)?,
                b: row.get(4)?,
                c: row.get(5)?,
                d: row.get(6)?,
                e: row.get(7)?,
                f: row.get(8)?,
                g: row.get(9)?,
                h: row.get(10)?,
                i: row.get(11)?,
                j: row.get(12)?,
                k: row.get(13)?,
                l: row.get(14)?,
                m: row.get(15)?,
                n: row.get(16)?,
                o: row.get(17)?,
                p: row.get(18)?,
                q: row.get(19)?,
                r: row.get(20)?,
                s: row.get(21)?,
                t: row.get(22)?,
                u: row.get(23)?,
                v: row.get(24)?,
            })
        },
    )?;

    // Check
    assert_eq!(
        tuple,
        Test {
            id,
            nil: None,
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
            k: "kkk中文".into(),
            l: "ll ".into(),
            m: "m".into(),
            n: 13.1,
            o: 14.1,
            p: 15.1,
            q: 16.1,
            r: 17.1,
            s: text_data,
            t: "t".into(),
            u: (2021, 3, 1, 15, 38, 0, 123456),
            v: blob_data
        }
    );

    Ok(())
}
