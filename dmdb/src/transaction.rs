use crate::{connection::InternalConnection, Params, Result, Row, Statement};

pub struct Transaction<'conn> {
    conn: &'conn InternalConnection,
}

impl<'conn> Transaction<'conn> {
    pub(crate) fn new(conn: &'conn InternalConnection) -> Result<Self> {
        // Unset autocommit
        conn.set_autocommit(false)?;

        Ok(Self { conn })
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>> {
        self.conn.prepare(sql)
    }

    pub fn execute<P: Params>(&self, sql: &str, params: P) -> Result<()> {
        self.conn.execute(sql, params)
    }

    pub fn query_row<P, F, T>(&self, sql: &str, params: P, map: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(Row<'_, '_, '_>) -> Result<T>,
    {
        self.conn.query_row(sql, params, map)
    }

    pub fn ident_current(&self, table: &str) -> Result<u64> {
        self.conn.ident_current(table)
    }

    pub fn last_insert_id(&self) -> Result<u64> {
        self.conn.last_insert_id()
    }

    pub fn commit(mut self) -> Result<()> {
        self.commit_ref()
    }

    pub fn rollback(mut self) -> Result<()> {
        self.rollback_ref()
    }

    pub fn commit_ref(&mut self) -> Result<()> {
        self.conn.execute("COMMIT", [])
    }

    pub fn rollback_ref(&mut self) -> Result<()> {
        self.conn.execute("ROLLBACK", [])
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // Roll back the transaction. This will have no effect if already committed or rolled back
        self.rollback_ref().ok();

        // Set autocommit
        self.conn.set_autocommit(true).ok();
    }
}
