use crate::{
    utils::{cstring::CString, error::error_check},
    Error, Result,
};

pub struct Connection {}

impl Connection {
    pub fn connect(server: &str, user: &str, pwd: &str) -> Result<Self> {
        let mut henv: dmdb_sys::dhenv = std::ptr::null_mut();
        let mut hcon: dmdb_sys::dhcon = std::ptr::null_mut();

        unsafe {
            let rt = dmdb_sys::dpi_alloc_env(&mut henv);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_ENV, henv, msg => Error::Connection(msg));

            let rt = dmdb_sys::dpi_alloc_con(henv, &mut hcon);
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, hcon, msg => Error::Connection(msg));

            let rt = dmdb_sys::dpi_login(
                hcon,
                CString::new(server).as_ptr_mut(),
                CString::new(user).as_ptr_mut(),
                CString::new(pwd).as_ptr_mut(),
            );
            error_check!(rt, dmdb_sys::DSQL_HANDLE_DBC, hcon, msg => Error::Connection(msg));
        }

        Ok(Self {})
    }
}
