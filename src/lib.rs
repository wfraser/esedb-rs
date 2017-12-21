#![allow(non_upper_case_globals)]
#![allow(unknown_lints, useless_transmute)]

#[macro_use] extern crate log;

extern crate winapi;
extern crate esent;

// re-export the constants and types for esent-sys
pub use winapi::esent::*;

#[macro_use] mod macros;

mod database;
pub use database::*;

mod error;
pub use error::*;

mod instance;
pub use instance::*;

mod session;
pub use session::*;

mod wide_string;
pub use wide_string::*;

mod table;
pub use table::*;

mod util;

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
