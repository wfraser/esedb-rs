#[macro_export]
macro_rules! jetcall {
    ($call:expr) => {
        match $call {
            JET_errSuccess => Ok(()),
            err => Err(JetError::from(err)),
        }
    }
}
