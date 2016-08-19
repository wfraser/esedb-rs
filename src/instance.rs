use esent::*;
use super::*;

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
        unsafe { jettry!(JetCreateInstance2W(&mut self.instance, name.as_ptr(), null(), JET_bitNil)); }
        Ok(())
    }

    pub fn set_string_parameter(&mut self, param: u32, s: &WideString) -> Result<(), JetError> {
        unsafe { jettry!(JetSetSystemParameterW(&mut self.instance, 0, param, 0, s.as_ptr())); }
        Ok(())
    }

    pub fn set_int_parameter(&mut self, param: u32, i: usize) -> Result<(), JetError> {
        unsafe { jettry!(JetSetSystemParameterW(&mut self.instance, 0, param, i, null())); }
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), JetError> {
        debug!("initializing JetInstance");
        let mut instance = self.instance;
        match unsafe { jetcall!(JetInit(&mut instance)) } {
            Ok(()) => {
                debug!("initialized JetInstance {:x}", instance);
                self.instance = instance;
                Ok(())
            },
            Err(e) => {
                // The documentation is fuzzy here, but it looks like if JetInit fails, then
                // JetTerm is not to be used (in practice it returns JET_errInvalidInstance).
                // So we set it to Nil as a sentinel to prevent calling JetTerm on drop.
                self.instance = JET_instanceNil;
                error!("JetInit failed: {}", e);
                Err(e)
            }
        }
    }

    pub fn create_session<'a>(&'a self) -> Result<JetSession<'a>, JetError> {
        debug!("creating new JetSession");
        assert!(self.instance != JET_instanceNil);
        let mut sesid = JET_sesidNil;
        unsafe { jettry!(JetBeginSessionW(self.instance, &mut sesid, null(), null())); }
        debug!("created JetSession {:x}", sesid);
        Ok(JetSession::new(self, sesid))
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
            debug!("terminating JetInstance {:x}", self.instance);
            unsafe {
                if jetcall!(JetTerm2(self.instance, JET_bitTermComplete)).is_err() {
                    jetcall!(JetTerm2(self.instance, JET_bitTermDirty)).unwrap();
                }
            }
        }
    }
}
