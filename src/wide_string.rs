use std::ffi::{OsStr, OsString};
use std::fmt;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

pub struct WideString {
    ucs2: Vec<u16>,
}

impl WideString {
    pub fn as_ptr(&self) -> *const u16 {
        self.ucs2.as_ptr()
    }
}

impl fmt::Display for WideString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", OsString::from(self).to_string_lossy())
    }
}

impl fmt::Debug for WideString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", OsString::from(self))
    }
}

impl<'a> From<&'a OsStr> for WideString {
    fn from(s: &OsStr) -> WideString {
        WideString {
            ucs2: s.encode_wide().chain(Some(0).into_iter()).collect()
        }
    }
}

impl From<OsString> for WideString {
    fn from(s: OsString) -> WideString {
        WideString::from(s.as_os_str())
    }
}

impl<'a> From<&'a str> for WideString {
    fn from(s: &str) -> WideString {
        WideString::from(OsStr::new(s))
    }
}

impl From<String> for WideString {
    fn from(s: String) -> WideString {
        WideString::from(OsStr::new(&s))
    }
}

impl<'a> From<&'a WideString> for OsString {
    fn from(s: &WideString) -> OsString {
        OsString::from_wide(&s.ucs2[0..s.ucs2.len() - 1]) // remove the trailing NUL
    }
}
