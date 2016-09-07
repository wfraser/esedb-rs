use esent::*;

use std::ffi::OsString;
use std::marker::PhantomData;
use std::mem::{size_of, transmute, uninitialized};
use std::ptr::{null, null_mut};

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
    pub fn move_prev_key(&self) -> Result<(), JetError> {
        self.move_internal(JET_MovePrevious, true)
    }
    pub fn move_last(&self) -> Result<(), JetError> {
        self.move_internal(JET_MoveLast, false)
    }

    pub fn retrieve_column_bytes<T: Copy>(&self, column_id: JET_COLUMNID)
            -> Result<Vec<T>, JetError> {
        let mut data: Vec<T> = vec![];
        let mut nbytes = 0u32;
        unsafe {
            match jetcall!(JetRetrieveColumn(
                    self.sesid, self.tableid, column_id,
                    null_mut(), 0, &mut nbytes, JET_bitNil, null_mut())) {
                Err(e) => match e.code {
                    JET_wrnBufferTruncated => (),
                    // TODO: maybe this should return Result<Option<Vec... instead
                    JET_wrnColumnNull => return Ok(data), // return empty vector
                    _ => return Err(e),
                },
                Ok(()) => panic!("expected this call to fail"),
            }
            // T is inappropriate if it doesn't evenly divide the number of bytes in the column.
            assert_eq!(0, nbytes as usize % size_of::<T>());
            data.reserve_exact(nbytes as usize / size_of::<T>());
            jettry!(JetRetrieveColumn(self.sesid, self.tableid, column_id,
                    transmute(data.as_mut_ptr()), nbytes, null_mut(), JET_bitNil, null_mut()));
            data.set_len(nbytes as usize / size_of::<T>());
        }
        Ok(data)
    }

    pub fn retrieve_wstring(&self, column_id: JET_COLUMNID) -> Result<WideString, JetError> {
        let ucs2: Vec<u16> = try!(self.retrieve_column_bytes(column_id));
        Ok(WideString::from(ucs2))
    }

    pub fn retrieve_string(&self, column_id: JET_COLUMNID) -> Result<OsString, JetError> {
        self.retrieve_wstring(column_id).map(|x| OsString::from(&x))
    }

    pub fn retrieve<T: Copy>(&self, column_id: JET_COLUMNID) -> Result<T, JetError> {
        unsafe {
            let mut data: T = uninitialized();
            let mut actual_bytes = 0;
            jettry!(JetRetrieveColumn(self.sesid, self.tableid, column_id, transmute(&mut data),
                    size_of::<T>() as u32, &mut actual_bytes, JET_bitNil, null_mut()));
            assert_eq!(size_of::<T>() as u32, actual_bytes);
            Ok(data)
        }
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

    pub fn select_index(&self, index_name: &WideString) -> Result<(), JetError> {
        unsafe { jettry!(JetSetCurrentIndexW(self.sesid, self.tableid, index_name.as_ptr())); }
        Ok(())
    }

    fn seek_internal(&self, seek_type: SeekType, data: &[u8]) -> Result<bool, JetError> {
        let seek_grbit = match seek_type {
            SeekType::Equal => JET_bitSeekEQ,
            SeekType::EqualOnly => JET_bitSeekEQ | JET_bitSetIndexRange,
            SeekType::EqualOrGreater => JET_bitSeekGE,
            SeekType::EqualOrLesser => JET_bitSeekLE,
            SeekType::ClosestGreater => JET_bitSeekGT,
            SeekType::ClosestLesser => JET_bitSeekLT,
        };

        unsafe {
            jettry!(JetMakeKey(self.sesid, self.tableid, transmute(data.as_ptr()),
                    data.len() as u32, JET_bitNewKey));
            match JetSeek(self.sesid, self.tableid, seek_grbit) {
                JET_errSuccess => Ok(true),
                JET_wrnSeekNotEqual => Ok(false),
                other => Err(JetError::from(other)),
            }
        }
    }

    pub fn seek<T: Copy>(&self, seek_type: SeekType, data: &T)
            -> Result<bool, JetError> {
        self.seek_internal(seek_type, byte_slice(data))
    }

    pub fn seek_slice<T: Copy>(&self, seek_type: SeekType, slice: &[T])
            -> Result<bool, JetError> {
        self.seek_internal(seek_type, slice_transmute(slice))
    }

    pub fn seek_wstr(&self, seek_type: SeekType, wstr: &WideString)
            -> Result<bool, JetError> {
        self.seek_internal(seek_type, slice_transmute(wstr.as_ucs2_slice()))
    }

    fn update_internal(&self, column_id: JET_COLUMNID, data: &[u8]) -> Result<(), JetError> {
        unsafe {
            jetcall!(JetSetColumn(self.sesid, self.tableid, column_id, transmute(data.as_ptr()),
                    data.len() as u32, JET_bitNil, null()))
        }
    }

    pub fn update<T: Copy>(&self, column_id: JET_COLUMNID, data: &T)
            -> Result<(), JetError> {
        self.update_internal(column_id, byte_slice(data))
    }

    pub fn update_slice<T: Copy>(&self, column_id: JET_COLUMNID, slice: &[T])
            -> Result<(), JetError> {
        self.update_internal(column_id, slice_transmute(slice))
    }

    pub fn update_wstr(&self, column_id: JET_COLUMNID, wstr: &WideString)
            -> Result<(), JetError> {
        self.update_internal(column_id, slice_transmute(wstr.as_ucs2_slice()))
    }
}

pub enum SeekType {
    Equal,
    EqualOnly,      // also sets the index range to only match the specified key
    EqualOrGreater,
    EqualOrLesser,
    ClosestGreater,
    ClosestLesser,
}

impl<'a> Drop for JetTable<'a> {
    fn drop(&mut self) {
        debug!("closing JetTable {:x}", self.tableid);
        unsafe { jetcall!(JetCloseTable(self.sesid, self.tableid)).unwrap() }
    }
}
