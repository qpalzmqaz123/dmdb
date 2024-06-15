use crate::{utils::error::error_check, ColumnInfo, Error, Result, Row, Statement};

pub struct Rows<'conn, 'stmt> {
    pub(crate) stmt: &'stmt mut Statement<'conn>,
    pub(crate) col_infos: Vec<ColumnInfo>,
}

impl<'conn, 'stmt> Rows<'conn, 'stmt> {
    pub(crate) fn new(stmt: &'stmt mut Statement<'conn>) -> Result<Self> {
        let mut columns = vec![];
        for i in 0..stmt.get_column_count()? {
            columns.push(stmt.get_column_info(i + 1)?);
        }

        Ok(Self {
            stmt,
            col_infos: columns,
        })
    }

    pub fn next(&mut self) -> Result<Option<Row<'conn, 'stmt, '_>>> {
        let mut _row_num: dmdb_sys::ulength = 0;

        unsafe {
            let rt = dmdb_sys::dpi_fetch(self.stmt.hstmt, &mut _row_num);
            if rt == dmdb_sys::DSQL_NO_DATA as dmdb_sys::DPIRETURN {
                return Ok(None);
            }
            error_check!(rt, dmdb_sys::DSQL_HANDLE_STMT, self.stmt.hstmt, msg => Error::Statement(format!("Next error: {}", msg)));
        }

        Ok(Some(Row::new(self)?))
    }

    pub fn columns(&self) -> &[ColumnInfo] {
        &self.col_infos
    }
}
