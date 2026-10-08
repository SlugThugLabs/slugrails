//! String, character, and raw string literal scanning for code analysis.

use super::find_byte_from;

/// Whether the source looks like Rust. Used only to disambiguate `'…'`, which is
/// a character/lifetime in Rust and a string in Python/JS.
pub(super) fn looks_rust_like(source: &str) -> bool {
    source.contains("fn ")
        || source.contains("impl ")
        || source.contains("#[")
        || source.contains("pub ")
}

/// Recognizes a quoted literal at `index`, returning `(index_after, body_start, body_end)`.
pub(super) fn scan_quoted(bytes: &[u8], index: usize, rust_like: bool) -> Option<(usize, usize, usize)> {
    match bytes.get(index).copied()? {
        b'"' => scan_double_quoted(bytes, index),
        b'`' => {
            let body_start = index + 1;
            let body_end = find_byte_from(bytes, body_start, b'`')?;
            Some((body_end + 1, body_start, body_end))
        }
        b'\'' => scan_single_quoted(bytes, index, rust_like),
        b'r' | b'b' => scan_raw_string(bytes, index),
        _ => None,
    }
}

/// Double-quoted string, including Python's triple-quoted form.
fn scan_double_quoted(bytes: &[u8], index: usize) -> Option<(usize, usize, usize)> {
    if bytes.get(index + 1) == Some(&b'"') && bytes.get(index + 2) == Some(&b'"') {
        let body_start = index + 3;
        let mut probe = body_start;
        while probe + 2 < bytes.len() {
            if bytes[probe] == b'"' && bytes[probe + 1] == b'"' && bytes[probe + 2] == b'"' {
                return Some((probe + 3, body_start, probe));
            }
            probe += 1;
        }
        return None;
    }
    let body_start = index + 1;
    let mut probe = body_start;
    let mut escaped = false;
    while probe < bytes.len() {
        let byte = bytes[probe];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some((probe + 1, body_start, probe));
        }
        probe += 1;
    }
    None
}

/// Rust raw string `r"…"` / `r#"…"#` / `br#"…"#`.
fn scan_raw_string(bytes: &[u8], index: usize) -> Option<(usize, usize, usize)> {
    let mut probe = index;
    if bytes.get(probe) == Some(&b'b') {
        probe += 1;
    }
    if bytes.get(probe) != Some(&b'r') {
        return None;
    }
    probe += 1;
    let mut hashes = 0usize;
    while bytes.get(probe) == Some(&b'#') {
        hashes += 1;
        probe += 1;
    }
    if bytes.get(probe) != Some(&b'"') {
        return None;
    }
    let body_start = probe + 1;
    let mut scan_at = body_start;
    while scan_at < bytes.len() {
        if bytes[scan_at] == b'"' {
            let terminated = (0..hashes).all(|h| bytes.get(scan_at + 1 + h) == Some(&b'#'));
            if terminated {
                return Some((scan_at + 1 + hashes, body_start, scan_at));
            }
        }
        scan_at += 1;
    }
    None
}

/// Single-quoted literal: a Rust character (`'x'`, `'\n'`) or a Python/JS string.
/// Rust lifetimes carry no closing quote on the line and are therefore skipped.
fn scan_single_quoted(
    bytes: &[u8],
    index: usize,
    rust_like: bool,
) -> Option<(usize, usize, usize)> {
    if bytes.get(index + 1) == Some(&b'\\') {
        let mut probe = index + 2;
        let mut escaped = true;
        while probe < bytes.len() {
            let byte = bytes[probe];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\'' {
                return Some((probe + 1, index + 1, probe));
            } else if byte == b'\n' {
                return None;
            }
            probe += 1;
        }
        return None;
    }
    if bytes.get(index + 2) == Some(&b'\'') && bytes.get(index + 1).is_some_and(|b| *b != b'\n') {
        return Some((index + 3, index + 1, index + 2));
    }
    if rust_like {
        return None;
    }
    let line_end = find_byte_from(bytes, index + 1, b'\n').unwrap_or(bytes.len());
    let mut probe = index + 1;
    let mut escaped = false;
    while probe < line_end {
        let byte = bytes[probe];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'\'' {
            return Some((probe + 1, index + 1, probe));
        }
        probe += 1;
    }
    None
}
