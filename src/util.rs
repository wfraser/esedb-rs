// Extensible Storage Engine database library for Rust
// Copyright 2016-2019 by William R. Fraser

use std::mem::size_of;
use std::slice;

pub fn slice_transmute<T: Copy, U: Copy>(src: &[T]) -> &[U] {
    // types are inappropriate if one doesn't fit evenly inside the other.
    assert_eq!(0, (src.len() * size_of::<T>()) % size_of::<U>());
    unsafe {
        slice::from_raw_parts(
            src.as_ptr() as *const U,
            (src.len() * size_of::<T>()) / size_of::<U>())
    }
}

pub fn byte_slice<T: Copy>(data: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(data as *const T as *const u8, size_of::<T>()) }
}
