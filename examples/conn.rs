use dmdb::{params, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect("127.0.0.1:5236", "SYSDBA", "SYSDBA")?;
    let mut stmt = conn.prepare("SELECT * FROM test1 WHERE a = ?")?;
    let mut rows = stmt.query(params![1])?;
    while let Some(row) = rows.next()? {
        let a = row.get::<u32>(1)?;
        let b = row.get::<String>(2)?;
        println!("a: {:?}, b: {:?}", a, b);
    }

    Ok(())
}
