//! Negative fixture: Prohibited .unwrap() in production code.

pub fn parse_number(val: &str) -> i32 {
    val.parse::<i32>().unwrap()
}
