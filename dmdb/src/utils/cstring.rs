pub struct CString {
    s: std::ffi::CString,
}

impl CString {
    #[inline]
    pub fn new<S: Into<Vec<u8>>>(s: S) -> CString {
        CString {
            s: std::ffi::CString::new(s).unwrap_or(std::ffi::CString::new("").unwrap()),
        }
    }

    #[inline]
    pub fn as_ptr_mut(&mut self) -> *mut i8 {
        self.s.as_ptr() as *mut i8
    }
}
