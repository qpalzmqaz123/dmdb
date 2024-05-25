use std::vec;

use crate::{
    statement::ColumnInfo, utils::error::error_check, Error, FromValue, Result, Rows, Value,
    ValueType,
};

pub struct Row<'conn, 'stmt, 'row> {
    rows: &'row Rows<'conn, 'stmt>,
}

impl<'conn, 'stmt, 'row> Row<'conn, 'stmt, 'row> {
    pub(crate) fn new(rows: &'row Rows<'conn, 'stmt>) -> Self {
        Self { rows }
    }

    pub fn columns(&self) -> &[ColumnInfo] {
        self.rows.columns()
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
        let (ctype, value_type) = match info.sql_type() as u32 {
            #[rustfmt::skip]
            dmdb_sys::DSQL_CHAR | dmdb_sys::DSQL_VARCHAR | dmdb_sys::DSQL_CLOB => {
                (dmdb_sys::DSQL_C_CHAR, ValueType::Text)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_BIT | dmdb_sys::DSQL_TINYINT | dmdb_sys::DSQL_SMALLINT | dmdb_sys::DSQL_INT | dmdb_sys::DSQL_BIGINT => {
                (dmdb_sys::DSQL_C_SBIGINT, ValueType::Integer)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_FLOAT | dmdb_sys::DSQL_DOUBLE | dmdb_sys::DSQL_DEC => {
                (dmdb_sys::DSQL_C_DOUBLE, ValueType::Float)
            },
            #[rustfmt::skip]
            dmdb_sys::DSQL_BLOB => {
                (dmdb_sys::DSQL_C_BINARY, ValueType::Blob)
            },
            dmdb_sys::DSQL_TIMESTAMP => (dmdb_sys::DSQL_C_TIMESTAMP, ValueType::DateTime),
            _ => {
                return Err(Error::Internal(format!(
                    "Unsupport sql type: {}",
                    info.sql_type()
                )))
            }
        };

        // Get raw data
        let Some(buf) = self.recevie_data(index as dmdb_sys::udint2, ctype as dmdb_sys::sdint2)? else {
            // Value is null
            return Ok(Value::Null)
        };

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
                let string = String::from_utf8(buf)
                    .map_err(|e| Error::FromValue(format!("Parse text value failed: {e}")))?;
                Value::Text(string)
            }
            ValueType::Blob => Value::Blob(buf),
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

    fn recevie_data(
        &self,
        index: dmdb_sys::udint2,
        ctype: dmdb_sys::sdint2,
    ) -> Result<Option<Vec<u8>>> {
        const BUF_LEN: usize = 4096;

        let mut buf = vec![0u8; BUF_LEN];
        let mut out_buf = vec![];

        loop {
            let mut val_len: dmdb_sys::slength = 0;
            let rt = unsafe {
                dmdb_sys::dpi_get_data(
                    self.rows.stmt.hstmt,
                    index,
                    ctype,
                    buf.as_mut_ptr() as dmdb_sys::dpointer,
                    BUF_LEN as dmdb_sys::slength,
                    &mut val_len,
                )
            };

            // Value is null
            if val_len < 0 {
                return Ok(None);
            }

            let tmp_data_size = std::cmp::min(val_len as usize, BUF_LEN);
            let tmp_data = buf.get(0..tmp_data_size).ok_or(Error::Statement(format!(
                "Get column data `{}` out of range",
                index
            )))?;

            if rt == dmdb_sys::DSQL_SUCCESS as dmdb_sys::DPIRETURN
                || rt == dmdb_sys::DSQL_SUCCESS_WITH_INFO as dmdb_sys::DPIRETURN
            {
                out_buf.extend(tmp_data);
            } else if rt == dmdb_sys::DSQL_NO_DATA as dmdb_sys::DPIRETURN {
                out_buf.extend(tmp_data);
                break;
            } else {
                error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.rows.stmt.hstmt, msg => Error::Statement(format!("Get column data `{}` failed: {}", index, msg)));
            }
        }

        Ok(Some(out_buf))
    }
}
