// Extensible Storage Engine database library for Rust
// Copyright 2016-2019 by William R. Fraser

#[macro_export]
macro_rules! jetcall {
    ($call:expr) => {
        match $call {
            JET_errSuccess => Ok(()),
            err => Err(JetError::from(err)),
        }
    }
}

#[macro_export]
macro_rules! jettry {
    ($func:ident($($args:expr),*)) => {
        match jetcall!($func($($args),*)) {
            Ok(x) => x,
            Err(e) => {
                error!("{} failed: {}", stringify!($func), e);
                return Err(e);
            }
        }
    }
}
