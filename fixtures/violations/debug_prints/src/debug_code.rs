//! Negative fixture: Leftover dbg!() macro in library code.

pub fn compute_value(input: i32) -> i32 {
    let intermediate = input * 2;
    dbg!(intermediate);
    intermediate + 1
}
