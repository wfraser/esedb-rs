use std::mem::{size_of, transmute};
use std::slice;

pub fn slice_transmute<T: Copy, U: Copy>(src: &[T]) -> &[U] {
    // types are inappropriate if one doesn't fit evenly inside the other.
    assert_eq!(0, (src.len() * size_of::<T>()) % size_of::<U>());
    unsafe {
        slice::from_raw_parts(
            transmute(src.as_ptr()),
            (src.len() * size_of::<T>()) / size_of::<U>())
    }
}

pub fn byte_slice<T: Copy>(data: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(transmute(&data), size_of::<T>()) }
}
