use std::mem::{size_of, transmute};
use std::slice;

pub unsafe fn slice_transmute<'a, T, U>(src: &'a [T]) -> &'a [U] {
    assert_eq!(0, src.len() % size_of::<U>());
    let ptr_of_u: *const U = transmute(src.as_ptr());
    slice::from_raw_parts(ptr_of_u, src.len() / size_of::<U>())
}
