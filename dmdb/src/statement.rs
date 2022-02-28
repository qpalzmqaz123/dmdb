use std::mem::size_of_val;

use crate::{utils::error::error_check, Connection, Error, Params, Result, Rows, Value};

#[derive(Debug)]
pub(crate) struct ColumnInfo {
    #[allow(unused)]
    pub name: String,
    pub length: usize,
    pub sql_type: dmdb_sys::sdint2,
}

pub struct Statement<'conn> {
    pub(crate) hstmt: dmdb_sys::dhstmt,
    pub(crate) values: Vec<Box<Value>>,
    _conn: &'conn Connection,
}

impl<'conn> Statement<'conn> {
    pub(crate) fn new(hstmt: dmdb_sys::dhstmt, conn: &'conn Connection) -> Self {
        Self {
            hstmt,
            values: vec![],
            _conn: conn,
        }
    }

    pub(crate) fn get_column_info(&self, index: usize) -> Result<ColumnInfo> {
        let mut name: [u8; 64] = [0; 64];
        let mut name_len: dmdb_sys::sdint2 = 0;
        let mut sql_type: dmdb_sys::sdint2 = 0;
        let mut length: dmdb_sys::ulength = 0;
        unsafe {
            let rt = dmdb_sys::dpi_desc_column(
                self.hstmt,
                index as dmdb_sys::sdint2,
                &mut name as *mut u8 as *mut dmdb_sys::sdbyte,
                size_of_val(&name) as dmdb_sys::sdint2,
                &mut name_len,
                &mut sql_type,
                &mut length,
                0 as *mut dmdb_sys::sdint2,
                0 as *mut dmdb_sys::sdint2,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.hstmt, msg => Error::Statement(format!("Get column info `{}` failed: {}", index, msg)));
        }

        Ok(ColumnInfo {
            name: String::from_utf8_lossy(&name[..name_len as usize]).to_string(),
            length: length as usize,
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
        params.bind(self)?;

        unsafe {
            let rt = dmdb_sys::dpi_exec(self.hstmt);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.hstmt, msg => Error::Statement(msg));
        }

        Ok(Rows::new(self)?)
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        unsafe {
            dmdb_sys::dpi_free_stmt(self.hstmt);
        }
    }
}
