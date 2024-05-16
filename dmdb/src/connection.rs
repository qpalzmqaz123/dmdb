use crate::{
    utils::{cstring::CString, error::error_check},
    Error, Params, Result, Row, Statement, Transaction,
};

pub struct Connection {
    server: String,
    user: String,
    password: String,
    conn: Option<InternalConnection>,
}

macro_rules! require_conn {
    ($self:expr) => {{
        if $self.conn.is_none() {
            let conn = InternalConnection::connect(&$self.server, &$self.user, &$self.password)?;
            $self.conn = Some(conn);
        }

        $self
            .conn
            .as_ref()
            .ok_or(Error::Internal("Connection is none".into()))?
    }};
}

macro_rules! drop_conn_on_error {
    ($self:expr, $res:expr) => {{
        let res = $res;
        if let Err(Error::Connection(_)) = res.as_ref() {
            let conn_opt_ptr = &$self.conn as *const _ as *mut Option<InternalConnection>;
            unsafe {
                (*conn_opt_ptr).take();
            }
        }

        res
    }};
}

impl Connection {
    pub fn connect(server: &str, user: &str, pwd: &str) -> Result<Self> {
        let instance = Self {
            server: server.into(),
            user: user.into(),
            password: pwd.into(),
            conn: None,
        };

        Ok(instance)
    }

    pub fn prepare(&mut self, sql: &str) -> Result<Statement<'_>> {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.prepare(sql))
    }

    pub fn execute<P: Params>(&mut self, sql: &str, params: P) -> Result<()> {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.execute(sql, params))
    }

    pub fn query_row<P, F, T>(&mut self, sql: &str, params: P, map: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(Row<'_, '_, '_>) -> Result<T>,
    {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.query_row(sql, params, map))
    }

    pub fn ident_current(&mut self, table: &str) -> Result<u64> {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.ident_current(table))
    }

    pub fn last_insert_id(&mut self) -> Result<u64> {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.last_insert_id())
    }

    pub fn transaction(&mut self) -> Result<Transaction<'_>> {
        let conn = require_conn!(self);
        drop_conn_on_error!(self, conn.transaction())
    }
}

pub struct InternalConnection {
    henv: dmdb_sys::dhenv,
    hcon: dmdb_sys::dhcon,
}

unsafe impl Send for InternalConnection {}

impl InternalConnection {
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

            let rt = dmdb_sys::dpi_set_con_attr(
                hcon,
                dmdb_sys::DSQL_ATTR_LOCAL_CODE as _,
                dmdb_sys::PG_UTF8 as _,
                0,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, hcon, msg => Error::Connection(msg));

            let rt = dmdb_sys::dpi_set_con_attr(
                hcon,
                dmdb_sys::DSQL_ATTR_AUTOCOMMIT as _,
                dmdb_sys::DSQL_AUTOCOMMIT_ON as _,
                0,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, hcon, msg => Error::Connection(msg));

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

            let rt = dmdb_sys::dpi_set_stmt_attr(
                hstmt,
                dmdb_sys::DSQL_ATTR_SQL_CHARSET as _,
                dmdb_sys::PG_UTF8 as _,
                0,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, hstmt, msg => Error::Prepare(msg));

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

    pub fn last_insert_id(&self) -> Result<u64> {
        let id = self.query_row(&format!("SELECT @@IDENTITY"), [], |row| {
            row.get::<Option<u64>>(1)
        })?;

        Ok(id.unwrap_or(0))
    }

    pub fn transaction(&self) -> Result<Transaction<'_>> {
        Transaction::new(self)
    }

    pub fn set_autocommit(&self, mode: bool) -> Result<()> {
        let mode = if mode {
            dmdb_sys::DSQL_AUTOCOMMIT_ON
        } else {
            dmdb_sys::DSQL_AUTOCOMMIT_OFF
        };

        unsafe {
            let rt = dmdb_sys::dpi_set_con_attr(
                self.hcon,
                dmdb_sys::DSQL_ATTR_AUTOCOMMIT as _,
                mode as _,
                0,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, self.hcon, msg => Error::Connection(msg));
        }

        Ok(())
    }
}

impl Drop for InternalConnection {
    fn drop(&mut self) {
        unsafe {
            dmdb_sys::dpi_logout(self.hcon);
            dmdb_sys::dpi_free_con(self.hcon);
            dmdb_sys::dpi_free_env(self.henv);
        }
    }
}
