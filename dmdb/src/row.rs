use std::{ffi::CStr, mem::size_of};

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
            dmdb_sys::DSQL_CHAR | dmdb_sys::DSQL_VARCHAR | dmdb_sys::DSQL_CLOB => {
                (info.length + 1, dmdb_sys::DSQL_C_NCHAR, ValueType::Text)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_BIT | dmdb_sys::DSQL_TINYINT | dmdb_sys::DSQL_SMALLINT | dmdb_sys::DSQL_INT | dmdb_sys::DSQL_BIGINT => {
                (size_of::<i64>(), dmdb_sys::DSQL_C_SBIGINT, ValueType::Integer)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_FLOAT | dmdb_sys::DSQL_DOUBLE | dmdb_sys::DSQL_DEC => {
                (size_of::<f64>(), dmdb_sys::DSQL_C_DOUBLE, ValueType::Float)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_BLOB => {
                (info.length, dmdb_sys::DSQL_C_BINARY, ValueType::Blob)
            },
            dmdb_sys::DSQL_TIMESTAMP => (
                size_of::<dmdb_sys::dpi_timestamp_t>(),
                dmdb_sys::DSQL_C_TIMESTAMP,
                ValueType::DateTime,
            ),
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

        // Value is null
        if val_len <= 0 {
            return Ok(Value::Null);
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
                let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) };
                Value::Text(cstr.to_owned())
            }
            ValueType::Blob => Value::Blob(buf[..val_len as usize].to_vec()),
            ValueType::DateTime => {
                let ptr = buf.as_ptr() as *const dmdb_sys::dpi_timestamp_t;
                unsafe {
                    let ts = &*ptr;
                    Value::DateTime(
                        ts.year as _,
                        ts.month as _,
                        ts.day as _,
                        ts.hour as _,
                        ts.minute as _,
                        ts.second as _,
                        ts.fraction.wrapping_div(1000),
                    )
                }
            }
        };

        Ok(value)
    }
}
