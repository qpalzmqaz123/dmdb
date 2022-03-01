use std::mem::size_of;

use crate::{utils::error::error_check, Error, FromValue, Result, Rows, Value, ValueType};

pub struct Row<'conn, 'stmt, 'row> {
    rows: &'row Rows<'conn, 'stmt>,
}

impl<'conn, 'stmt, 'row> Row<'conn, 'stmt, 'row> {
    pub(crate) fn new(rows: &'row Rows<'conn, 'stmt>) -> Self {
        Self { rows }
    }

    pub fn get<T: FromValue>(&self, index: usize) -> Result<T> {
        Ok(T::from_value(self.get_value(index)?)?)
    }

    pub fn get_value(&self, index: usize) -> Result<Value> {
        if index == 0 {
            return Err(Error::Index("Index must not 0".into()));
        }

        // Get info
        let info = self
            .rows
            .col_infos
            .get(index - 1)
            .ok_or(Error::Index(format!("Index `{}` out of range", index)))?;

        // Get value buffer info
        let (buf_len, ctype, value_type) = match info.sql_type as u32 {
            #[rustfmt::skip]
            dmdb_sys::DSQL_CHAR | dmdb_sys::DSQL_VARCHAR => {
                (info.length, dmdb_sys::DSQL_C_NCHAR, ValueType::Text)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_BIT | dmdb_sys::DSQL_TINYINT | dmdb_sys::DSQL_SMALLINT | dmdb_sys::DSQL_INT | dmdb_sys::DSQL_BIGINT => {
                (size_of::<i64>(), dmdb_sys::DSQL_C_SBIGINT, ValueType::Integer)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_FLOAT | dmdb_sys::DSQL_DOUBLE | dmdb_sys::DSQL_DEC => {
                (size_of::<f64>(), dmdb_sys::DSQL_C_DOUBLE, ValueType::Float)
            },
            _ => {
                return Err(Error::Internal(format!(
                    "Unsupport sql type: {}",
                    info.sql_type
                )))
            }
        };

        // Alloc buffer
        let mut buf = vec![0u8; buf_len];
        let mut val_len: dmdb_sys::slength = 0;

        // Get column data
        unsafe {
            let rt = dmdb_sys::dpi_get_data(
                self.rows.stmt.hstmt,
                index as dmdb_sys::udint2,
                ctype as dmdb_sys::sdint2,
                buf.as_mut_ptr() as dmdb_sys::dpointer,
                buf_len as dmdb_sys::slength,
                &mut val_len,
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.rows.stmt.hstmt, msg => Error::Statement(format!("Get column data `{}` failed: {}", index, msg)));
        }

        // Parse column data to value
        let value = match value_type {
            ValueType::Null => Value::Null,
            ValueType::Integer => {
                let n = unsafe { *(buf.as_ptr() as *const i64) };
                Value::Integer(n)
            }
            ValueType::Float => {
                let n = unsafe { *(buf.as_ptr() as *const f64) };
                Value::Float(n)
            }
            ValueType::Text => {
                let a = std::str::from_utf8(&buf[0..val_len as usize])
                    .map_err(|e| Error::Internal(format!("Parse result to string failed: {}", e)))?
                    .to_string();
                Value::Text(a)
            }
        };

        Ok(value)
    }
}
