//! Negative fixture: Prohibited panic!() in production code.

pub fn validate_positive(num: i32) {
    if num < 0 {
        panic!("Negative numbers are strictly disallowed");
    }
}
