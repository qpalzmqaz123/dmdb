use crate::{
    utils::{cstring::CString, error::error_check},
    Error, Params, Result, Row, Statement,
};

pub struct Connection {
    henv: dmdb_sys::dhenv,
    hcon: dmdb_sys::dhcon,
}

unsafe impl Send for Connection {}

impl Connection {
    pub fn connect(server: &str, user: &str, pwd: &str) -> Result<Self> {
        let mut henv: dmdb_sys::dhenv = std::ptr::null_mut();
        let mut hcon: dmdb_sys::dhcon = std::ptr::null_mut();

        unsafe {
            let rt = dmdb_sys::dpi_alloc_env(&mut henv);
            if rt != 0 {
                return Err(Error::Connection(
                    "Allocate environment handle failed.".into(),
                ));
            }

            let rt = dmdb_sys::dpi_alloc_con(henv, &mut hcon);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_ENV, henv, msg => Error::Connection(msg));

            let rt = dmdb_sys::dpi_login(
                hcon,
                CString::new(server).as_ptr_mut(),
                CString::new(user).as_ptr_mut(),
                CString::new(pwd).as_ptr_mut(),
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, hcon, msg => Error::Connection(msg));
        }

        Ok(Self { henv, hcon })
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>> {
        let mut hstmt: dmdb_sys::dhstmt = std::ptr::null_mut();

        unsafe {
            let rt = dmdb_sys::dpi_alloc_stmt(self.hcon, &mut hstmt);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, self.hcon, msg => Error::Connection(msg));

            let rt = dmdb_sys::dpi_prepare(hstmt, CString::new(sql).as_ptr_mut());
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, hstmt, msg => Error::Prepare(msg));
        }

        Ok(Statement::new(hstmt, self))
    }

    pub fn execute<P: Params>(&self, sql: &str, params: P) -> Result<()> {
        let mut stmt = self.prepare(sql)?;
        stmt.execute(params)
    }

    pub fn query_row<P, F, T>(&self, sql: &str, params: P, map: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(Row<'_, '_, '_>) -> Result<T>,
    {
        let mut stmt = self.prepare(sql)?;
        stmt.query_row(params, map)
    }

    pub fn ident_current(&self, table: &str) -> Result<u64> {
        let id = self.query_row(&format!("SELECT IDENT_CURRENT('{}')", table), [], |row| {
            row.get(1)
        })?;

        Ok(id)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            dmdb_sys::dpi_logout(self.hcon);
            dmdb_sys::dpi_free_con(self.hcon);
            dmdb_sys::dpi_free_env(self.henv);
        }
    }
}
