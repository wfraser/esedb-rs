use winapi::*;
use esent::*;

use super::*;

use std::ptr::null;

pub struct JetSession<'a> {
    _instance: &'a JetInstance,
    sesid: JET_SESID,
}

pub enum DatabaseAccessMode {
    ReadOnly,
    ReadWrite,
}

impl<'a> JetSession<'a> {
    pub fn new(instance: &'a JetInstance, sesid: JET_SESID) -> JetSession<'a> {
        assert!(sesid != JET_sesidNil);
        JetSession {
            _instance: instance,
            sesid: sesid,
        }
    }

    pub fn open_database<'b>(&'b mut self, path: &WideString, mode: DatabaseAccessMode)
            -> Result<JetDatabase<'b>, JetError> {
        println!("{:?}", path);
        let mut dbid = JET_dbidNil;
        unsafe {
            let bit = match mode {
                DatabaseAccessMode::ReadOnly => JET_bitDbReadOnly,
                DatabaseAccessMode::ReadWrite => JET_bitNil,
            };
            try!(jetcall!(JetAttachDatabaseW(self.sesid, path.as_ptr(), bit)));
            try!(jetcall!(JetOpenDatabaseW(self.sesid, path.as_ptr(), null(), &mut dbid, JET_bitNil)));
        }
        Ok(JetDatabase::new(self, dbid))
    }

    pub unsafe fn raw(&self) -> JET_SESID {
        self.sesid
    }
}

impl<'a> Drop for JetSession<'a> {
    fn drop(&mut self) {
        debug!("dropping JetSession {}", self.sesid);
        unsafe { jetcall!(JetEndSession(self.sesid, JET_bitNil)).unwrap(); }
    }
}
