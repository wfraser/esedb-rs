use esent::*;

use super::*;

use std::marker::PhantomData;
use std::ptr::null;

#[derive(Debug)]
pub struct JetSession<'a> {
    _lifetime: PhantomData<&'a JetInstance>,
    sesid: JET_SESID,
}

#[derive(Debug, Copy, Clone)]
pub enum DatabaseAccessMode {
    ReadOnly,
    ReadWrite,
}

impl<'a> JetSession<'a> {
    pub fn new(_instance: &'a JetInstance, sesid: JET_SESID) -> JetSession<'a> {
        assert!(sesid != JET_sesidNil);
        JetSession {
            _lifetime: PhantomData,
            sesid,
        }
    }

    pub fn open_database<'b>(&'b mut self, path: &WideString, mode: DatabaseAccessMode)
            -> Result<JetDatabase<'b>, JetError> {
        debug!("attaching+opening JetDatabase from {:?}", path);
        let mut dbid = JET_dbidNil;
        unsafe {
            let bit = match mode {
                DatabaseAccessMode::ReadOnly => JET_bitDbReadOnly,
                DatabaseAccessMode::ReadWrite => JET_bitNil,
            };
            jettry!(JetAttachDatabaseW(self.sesid, path.as_ptr(), bit));
            jettry!(JetOpenDatabaseW(self.sesid, path.as_ptr(), null(), &mut dbid, JET_bitNil));
        }
        debug!("opened JetDatabase {:?} = {:x}", path, dbid);
        Ok(JetDatabase::new(self, dbid))
    }

    pub unsafe fn raw(&self) -> JET_SESID {
        self.sesid
    }
}

impl<'a> Drop for JetSession<'a> {
    fn drop(&mut self) {
        debug!("ending JetSession {:x}", self.sesid);
        unsafe { jetcall!(JetEndSession(self.sesid, JET_bitNil)).unwrap(); }
    }
}
