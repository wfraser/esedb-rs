use winapi::*;
use esent::*;
use super::strings::*;
use super::errors::*;

use std::ptr::null;

pub struct JetInstance {
    instance: JET_INSTANCE,
}

impl JetInstance {
    pub fn new() -> JetInstance {
        JetInstance {
            instance: JET_instanceNil,
        }
    }

    pub fn init_engine(&mut self, name: &WideString) -> Result<(), JetError> {
        unsafe { jetcall!(JetCreateInstance2W(&mut self.instance, name.as_ptr(), null(), JET_bitNil)) }
    }

    pub fn set_string_parameter(&mut self, param: u32, s: &WideString) -> Result<(), JetError> {
        unsafe { jetcall!(JetSetSystemParameterW(&mut self.instance, 0, param, 0, s.as_ptr())) }
    }

    pub fn set_int_parameter(&mut self, param: u32, i: usize) -> Result<(), JetError> {
        unsafe { jetcall!(JetSetSystemParameterW(&mut self.instance, 0, param, i, null())) }
    }

    pub fn init(&mut self) -> Result<(), JetError> {
        unsafe { jetcall!(JetInit(&mut self.instance)) }
    }
}

impl Into<JET_INSTANCE> for JetInstance {
    fn into(self) -> JET_INSTANCE {
        self.instance
    }
}

impl Drop for JetInstance {
    fn drop(&mut self) {
        if self.instance != JET_instanceNil {
            unsafe {
                if jetcall!(JetTerm2(self.instance, JET_bitTermComplete)).is_err() {
                    jetcall!(JetTerm2(self.instance, JET_bitTermDirty)).unwrap();
                }
            }
        }
    }
}
