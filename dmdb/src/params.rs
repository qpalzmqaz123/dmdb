use std::mem::{size_of, size_of_val};

use crate::{utils::error::error_check, Error, Result, Statement, ToValue, Value};

pub trait Params {
    fn bind(&self, stmt: &mut Statement) -> Result<()>;
}

impl Params for [&dyn ToValue; 0] {
    #[inline]
    fn bind(&self, _: &mut Statement) -> Result<()> {
        Ok(())
    }
}

impl Params for &[&dyn ToValue] {
    #[inline]
    fn bind(&self, stmt: &mut Statement) -> Result<()> {
        for (index, param) in self.iter().enumerate() {
            let value = Box::new(param.to_value());
            let iparam = index as dmdb_sys::udint2 + 1;
            let ctype = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(_) => dmdb_sys::DSQL_C_SBIGINT,
                Value::Float(_) => dmdb_sys::DSQL_C_DOUBLE,
                Value::Text(_) => dmdb_sys::DSQL_C_NCHAR,
                Value::Blob(_) => dmdb_sys::DSQL_C_BINARY,
                Value::DateTime(..) => dmdb_sys::DSQL_C_TIMESTAMP,
            } as dmdb_sys::sdint2;
            let dtype = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(_) => dmdb_sys::DSQL_BIGINT,
                Value::Float(_) => dmdb_sys::DSQL_DOUBLE,
                Value::Text(_) => dmdb_sys::DSQL_CLOB,
                Value::Blob(_) => dmdb_sys::DSQL_BLOB,
                Value::DateTime(..) => dmdb_sys::DSQL_TIMESTAMP,
            } as dmdb_sys::sdint2;
            let precision = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(_) => 20,
                Value::Float(_) => 20,
                Value::Text(s) => s.as_bytes_with_nul().len(),
                Value::Blob(v) => v.len(),
                Value::DateTime(..) => size_of::<dmdb_sys::dpi_timestamp_t>(),
            };
            let scale = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(_) => 0,
                Value::Float(_) => 3, // TODO
                Value::Text(_) => 0,
                Value::Blob(_) => 0,
                Value::DateTime(..) => 0,
            };
            let buf = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(i) => i as *const _ as *const u8,
                Value::Float(f) => f as *const _ as *const u8,
                Value::Text(s) => s.as_c_str().as_ptr() as *const u8,
                Value::Blob(v) => v.as_ptr(),
                Value::DateTime(y, m, d, h, i, s, us) => {
                    let ts = Box::new(dmdb_sys::dpi_timestamp_t {
                        year: *y as _,
                        month: *m as _,
                        day: *d as _,
                        hour: *h as _,
                        minute: *i as _,
                        second: *s as _,
                        fraction: (*us).wrapping_mul(1000) as _,
                    });
                    let buf = ts.as_ref() as *const _ as *const u8;
                    stmt.timestampes.push(ts);

                    buf
                }
            };
            let buf_len = match value.as_ref() {
                Value::Null => return Err(Error::Connection("Cannot bind null parameter".into())),
                Value::Integer(i) => size_of_val(i),
                Value::Float(f) => size_of_val(f),
                Value::Text(s) => s.as_bytes_with_nul().len(),
                Value::Blob(v) => v.len(),
                Value::DateTime(..) => size_of::<dmdb_sys::dpi_timestamp_t>(),
            };

            unsafe {
                let rt = dmdb_sys::dpi_bind_param(
                    stmt.hstmt,
                    iparam,
                    dmdb_sys::DSQL_PARAM_INPUT as dmdb_sys::sdint2,
                    ctype,
                    dtype,
                    precision as dmdb_sys::ulength,
                    scale as dmdb_sys::sdint2,
                    buf as dmdb_sys::dpointer,
                    buf_len as dmdb_sys::slength,
                    0 as *mut dmdb_sys::slength,
                );
                error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, stmt.hstmt, msg => Error::Parameter(msg));
            }

            stmt.values.push(value);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! params {
    () => {
        &[] as &[&dyn $crate::ToValue]
    };
    ($($param:expr),+ $(,)?) => {
        &[$(&$param as &dyn $crate::ToValue),+] as &[&dyn $crate::ToValue]
    };
}
