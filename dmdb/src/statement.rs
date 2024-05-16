use std::mem::size_of_val;

use crate::{
    utils::error::error_check, Error, InternalConnection, Params, Result, Row, Rows, Value,
};

#[derive(Debug)]
pub(crate) struct ColumnInfo {
    #[allow(unused)]
    pub name: String,
    pub sql_type: dmdb_sys::sdint2,
}

pub struct Statement<'conn> {
    pub(crate) hstmt: dmdb_sys::dhstmt,
    /// Temorary save the values for each bind parameter
    pub(crate) values: Vec<Box<Value>>,
    /// Temorary save the timestamp values for each bind parameter
    pub(crate) timestampes: Vec<Box<dmdb_sys::dpi_timestamp_t>>,
    /// Temorary save the buffer size for each value
    pub(crate) bind_ind_vec: Vec<Box<dmdb_sys::slength>>,
    _conn: &'conn InternalConnection,
}

impl<'conn> Statement<'conn> {
    pub(crate) fn new(hstmt: dmdb_sys::dhstmt, conn: &'conn InternalConnection) -> Self {
        Self {
            hstmt,
            values: vec![],
            timestampes: vec![],
            bind_ind_vec: vec![],
            _conn: conn,
        }
    }

    pub(crate) fn get_column_info(&self, index: usize) -> Result<ColumnInfo> {
        let mut name: [u8; 64] = [0; 64];
        let mut name_len: dmdb_sys::sdint2 = 0;
        let mut sql_type: dmdb_sys::sdint2 = 0;
        unsafe {
            let rt = dmdb_sys::dpi_desc_column(
                self.hstmt,
                index as dmdb_sys::sdint2,
                &mut name as *mut u8 as *mut dmdb_sys::sdbyte,
                size_of_val(&name) as dmdb_sys::sdint2,
                &mut name_len,
                &mut sql_type,
                0 as *mut dmdb_sys::ulength,
                0 as *mut dmdb_sys::sdint2,
                0 as *mut dmdb_sys::sdint2,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.hstmt, msg => Error::Statement(format!("Get column info `{}` failed: {}", index, msg)));
        }

        Ok(ColumnInfo {
            name: String::from_utf8_lossy(&name[..name_len as usize]).to_string(),
            sql_type,
        })
    }

    pub(crate) fn get_column_count(&self) -> Result<usize> {
        let mut col_cnt: dmdb_sys::sdint2 = 0;
        unsafe {
            let rt = dmdb_sys::dpi_number_columns(self.hstmt, &mut col_cnt);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.hstmt, msg => Error::Statement(format!("Get column count error: {}", msg)));
        }

        Ok(col_cnt as usize)
    }

    pub fn query<P: Params>(&mut self, params: P) -> Result<Rows<'conn, '_>> {
        self.execute(params)?;

        Ok(Rows::new(self)?)
    }

    pub fn query_row<P, F, T>(&mut self, params: P, map: F) -> Result<T>
    where
        P: Params,
        F: FnOnce(Row<'conn, '_, '_>) -> Result<T>,
    {
        let mut rows = self.query(params)?;
        if let Some(row) = rows.next()? {
            Ok(map(row)?)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    pub fn execute<P: Params>(&mut self, params: P) -> Result<()> {
        params.bind(self)?;

        unsafe {
            let rt = dmdb_sys::dpi_exec(self.hstmt);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.hstmt, msg => Error::Statement(msg));
        }

        Ok(())
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        unsafe {
            dmdb_sys::dpi_free_stmt(self.hstmt);
        }
    }
}
