//! Negative fixture: Prohibited unsafe code in production.

pub fn perform_dereference(ptr: *const i32) -> i32 {
    unsafe {
        *ptr
    }
}
