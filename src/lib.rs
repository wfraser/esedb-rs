#![allow(non_upper_case_globals)]

extern crate winapi;
extern crate esent;

#[macro_use] mod macros;

mod errors;
pub use errors::*;

mod instance;
pub use instance::*;

mod strings;
pub use strings::*;

#[cfg(test)]
mod test {
    use esent::*;
    use winapi::esent::*;
    use super::*;

    #[test]
    fn test_error_macro() {
        assert_eq!(JET_errInvalidSesid, unsafe { JetCloseTable(0, 0) }); // JET_errInvalidSesid "Invalid session handle"

        let e = unsafe { jetcall!(JetCloseTable(0, 0)) };
        assert!(e.is_err());
        assert_eq!(-1104, e.as_ref().err().unwrap().code);
        assert_eq!("Invalid session handle", e.as_ref().err().unwrap().text);
    }
}
