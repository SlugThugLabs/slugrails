//! Root library module for clean sample fixture.

#![forbid(unsafe_code)]

pub mod foundation;
pub mod core;

/// Adds two positive numbers safely with overflow check.
pub fn safe_add(first: u32, second: u32) -> Result<u32, &'static str> {
    first.checked_add(second).ok_or("Arithmetic overflow")
}
