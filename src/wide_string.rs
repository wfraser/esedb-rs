use std::ffi::{OsStr, OsString};
use std::fmt;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

#[derive(Clone)]
pub struct WideString {
    ucs2: Vec<u16>,
}

impl WideString {
    pub fn as_ptr(&self) -> *const u16 {
        self.ucs2.as_ptr()
    }
    pub fn is_empty(&self) -> bool {
        self.ucs2.is_empty()
    }
    pub fn len(&self) -> usize {
        self.ucs2.len()
    }
    pub fn as_ucs2_slice(&self) -> &[u16] {
        self.ucs2.as_slice()
    }
    pub fn to_string_lossy(&self) -> String {
        OsString::from(self).to_string_lossy().into_owned()
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
            ucs2: s.encode_wide().chain(Some(0).into_iter()).collect()  // add trailing NUL
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

impl From<Vec<u16>> for WideString {
    fn from(ucs2: Vec<u16>) -> WideString {
        assert_eq!(0, ucs2[ucs2.len() - 1]);
        WideString {
            ucs2,
        }
    }
}

impl<'a> From<&'a WideString> for OsString {
    fn from(s: &WideString) -> OsString {
        OsString::from_wide(&s.ucs2[0..s.ucs2.len() - 1]) // remove the trailing NUL
    }
}
