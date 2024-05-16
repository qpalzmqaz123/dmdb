use std::mem::size_of_val;

pub fn get_err_msg(handle_type: dmdb_sys::sdint2, handle: dmdb_sys::dhandle) -> String {
    let mut error_code: dmdb_sys::sdint4 = 0;
    let mut msg_len: dmdb_sys::sdint2 = 0;
    let mut err_msg: [u8; 128] = [0; 128];

    unsafe {
        let rv = dmdb_sys::dpi_get_diag_rec(
            handle_type,
            handle,
            1,
            &mut error_code as *mut dmdb_sys::sdint4,
            &mut err_msg as *mut u8 as *mut i8,
            size_of_val(&err_msg) as dmdb_sys::sdint2,
            &mut msg_len as *mut dmdb_sys::sdint2,
        );
        if rv == 0 {
            let s = std::str::from_utf8_unchecked(&err_msg[0..msg_len as usize]);
            s.into()
        } else {
            "".into()
        }
    }
}

macro_rules! error_check {
    ($rt:expr, $hdl_ty:expr, $hdl:expr, $msg:ident => $err:expr) => {
        if $rt != dmdb_sys::DSQL_SUCCESS as dmdb_sys::DPIRETURN
            && $rt != dmdb_sys::DSQL_SUCCESS_WITH_INFO as dmdb_sys::DPIRETURN
        {
            let $msg = $crate::utils::error::get_err_msg($hdl_ty as dmdb_sys::sdint2, $hdl);
            return Err($err);
        }
    };
}
pub(crate) use error_check;
