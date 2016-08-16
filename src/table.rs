use winapi::*;
use esent::*;

use std::mem::{size_of, transmute, uninitialized};
use std::ptr::null_mut;

use super::*;

pub struct JetTable<'a> {
    _database: &'a JetDatabase<'a>,
    sesid: JET_SESID,
    tableid: JET_TABLEID,
}

impl<'a> JetTable<'a> {
    pub fn new<'b>(session: &'b JetSession<'b>, database: &'a JetDatabase<'a>, tableid: JET_TABLEID)
            -> JetTable<'a> {
        let sesid = unsafe { session.raw() };
        JetTable {
            _database: database,
            sesid: sesid,
            tableid: tableid,
        }
    }

    pub unsafe fn raw(&self) -> JET_TABLEID {
        self.tableid
    }

    fn move_internal(&self, offset: i32, next_key: bool) -> Result<(), JetError> {
        let grbit = if next_key {
            JET_bitMoveKeyNE
        } else {
            JET_bitNil
        };
        unsafe { jetcall!(JetMove(self.sesid, self.tableid, offset, grbit)) }
    }

    pub fn move_first(&self) -> Result<(), JetError> {
        self.move_internal(JET_MoveFirst, false)
    }
    pub fn move_next(&self) -> Result<(), JetError> {
        self.move_internal(JET_MoveNext, false)
    }
    pub fn move_next_key(&self) -> Result<(), JetError> {
        self.move_internal(JET_MoveNext, true)
    }
    pub fn move_prev(&self) -> Result<(), JetError> {
        self.move_internal(JET_MovePrevious, false)
    }
    pub fn move_pref_key(&self) -> Result<(), JetError> {
        self.move_internal(JET_MovePrevious, true)
    }
    pub fn move_last(&self) -> Result<(), JetError> {
        self.move_internal(JET_MoveLast, false)
    }

    pub fn retrieve_column(&self, column_id: JET_COLUMNID) -> Result<Vec<u8>, JetError> {
        let mut data: Vec<u8> = vec![];
        let mut len = 0u32;
        unsafe {
            match jetcall!(JetRetrieveColumn(
                    self.sesid, self.tableid, column_id,
                    null_mut(), 0, &mut len, JET_bitNil, null_mut())) {
                Err(e) => if e.code != JET_wrnBufferTruncated {
                    return Err(e);
                },
                Ok(()) => panic!("expected this call to fail"),
            }
            data.reserve_exact(len as usize);
            try!(jetcall!(JetRetrieveColumn(self.sesid, self.tableid, column_id,
                    transmute(data.as_mut_ptr()), len, &mut len, JET_bitNil, null_mut())));
            data.set_len(len as usize);
        }
        Ok(data)
    }

    pub fn get_column_id(&self, column_name: &WideString) -> Result<JET_COLUMNID, JetError> {
        unsafe {
            let mut info: JET_COLUMNDEF = uninitialized();
            info.cbStruct = size_of::<JET_COLUMNDEF>() as u32;
            try!(jetcall!(JetGetTableColumnInfoW(self.sesid, self.tableid, column_name.as_ptr(), transmute(&mut info), info.cbStruct, JET_ColInfo)));
            Ok(info.columnid)
        }
    }
}

impl<'a> Drop for JetTable<'a> {
    fn drop(&mut self) {
        debug!("dropping table {}", self.tableid);
        unsafe { jetcall!(JetCloseTable(self.sesid, self.tableid)).unwrap() }
    }
}
