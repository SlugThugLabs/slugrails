//! Foundation module for clean sample fixture.

/// Sanitizes input strings by trimming whitespace.
pub fn sanitize(input: &str) -> String {
    input.trim().to_string()
}
