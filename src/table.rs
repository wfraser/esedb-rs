use winapi::*;
use esent::*;

use std::ffi::OsString;
use std::marker::PhantomData;
use std::mem::{size_of, transmute, uninitialized};
use std::ptr::null_mut;
use std::os::windows::ffi::OsStringExt;

use super::*;
use super::util::*;

pub struct JetTable<'a> {
    _lifetime: PhantomData<&'a JetDatabase<'a>>,
    sesid: JET_SESID,
    tableid: JET_TABLEID,
}

impl<'a> JetTable<'a> {
    pub fn new<'b>(session: &'b JetSession<'b>, _database: &'a JetDatabase<'a>, tableid: JET_TABLEID)
            -> JetTable<'a> {
        let sesid = unsafe { session.raw() };
        JetTable {
            _lifetime: PhantomData,
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

    pub fn retrieve_column_bytes(&self, column_id: JET_COLUMNID) -> Result<Vec<u8>, JetError> {
        let mut data: Vec<u8> = vec![];
        let mut len = 0u32;
        unsafe {
            match jetcall!(JetRetrieveColumn(
                    self.sesid, self.tableid, column_id,
                    null_mut(), 0, &mut len, JET_bitNil, null_mut())) {
                Err(e) => match e.code {
                    JET_wrnBufferTruncated => (),
                    // TODO: maybe this should return Result<Option<Vec... instead
                    JET_wrnColumnNull => return Ok(data), // return empty vector
                    _ => return Err(e),
                },
                Ok(()) => panic!("expected this call to fail"),
            }
            data.reserve_exact(len as usize);
            jettry!(JetRetrieveColumn(self.sesid, self.tableid, column_id,
                    transmute(data.as_mut_ptr()), len, &mut len, JET_bitNil, null_mut()));
            data.set_len(len as usize);
        }
        Ok(data)
    }

    pub fn retrieve_string(&self, column_id: JET_COLUMNID) -> Result<OsString, JetError> {
        let bytes: Vec<u8> = try!(self.retrieve_column_bytes(column_id));
        let ucs2: &[u16] = unsafe { slice_transmute(&bytes) };
        let osstring = OsString::from_wide(&ucs2[0..ucs2.len() - 1]); // remove the trailing NUL
        Ok(osstring)
    }

    pub fn retrieve_primitive<T: Copy>(&self, column_id: JET_COLUMNID) -> Result<T, JetError> {
        let bytes: Vec<u8> = try!(self.retrieve_column_bytes(column_id));
        let of_t: &[T] = unsafe { slice_transmute(&bytes) };
        assert_eq!(1, of_t.len()); // if there's more than one, then the size is wrong
        Ok(of_t[0])
    }

    pub fn get_column_id(&self, column_name: &WideString) -> Result<JET_COLUMNID, JetError> {
        unsafe {
            let mut info: JET_COLUMNDEF = uninitialized();
            info.cbStruct = size_of::<JET_COLUMNDEF>() as u32;
            jettry!(JetGetTableColumnInfoW(self.sesid, self.tableid, column_name.as_ptr(),
                    transmute(&mut info), info.cbStruct, JET_ColInfo));
            Ok(info.columnid)
        }
    }
}

impl<'a> Drop for JetTable<'a> {
    fn drop(&mut self) {
        debug!("closing JetTable {:x}", self.tableid);
        unsafe { jetcall!(JetCloseTable(self.sesid, self.tableid)).unwrap() }
    }
}
