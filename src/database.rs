use winapi::*;
use esent::*;

use std::ptr::null;

use super::*;

pub struct JetDatabase<'a> {
    session: &'a JetSession<'a>,
    sesid: JET_SESID,
    dbid: JET_DBID,
}

impl<'a> JetDatabase<'a> {
    pub fn new(session: &'a JetSession<'a>, dbid: JET_DBID) -> JetDatabase<'a> {
        let sesid = unsafe { session.raw() };
        JetDatabase {
            session: session,
            sesid: sesid,
            dbid: dbid,
        }
    }

    pub fn open_table<'b>(&'b self, tablename: &WideString) -> Result<JetTable<'b>, JetError> {
        let mut tableid = JET_tableidNil;
        unsafe { try!(jetcall!(JetOpenTableW(self.sesid, self.dbid, tablename.as_ptr(), null(), 0, JET_bitNil, &mut tableid))); }
        Ok(JetTable::new(self.session, self, tableid))
    }
}

impl<'a> Drop for JetDatabase<'a> {
    fn drop(&mut self) {
        debug!("dropping JetDatabase {}", self.dbid);
        unsafe { jetcall!(JetCloseDatabase(self.sesid, self.dbid, JET_bitNil)).unwrap() }
    }
}
