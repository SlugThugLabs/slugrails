//! Negative fixture: Function with 6 parameters (exceeds limit of 5).

pub fn excessive_parameters(
    param_one: i32,
    param_two: i32,
    param_three: i32,
    param_four: i32,
    param_five: i32,
    param_six: i32,
) -> i32 {
    param_one + param_two + param_three + param_four + param_five + param_six
}
