//! Numeric literal scanner for code analysis.

/// Recognizes a numeric literal starting at `index`, returning the index after it.
pub(super) fn scan_number_at(bytes: &[u8], index: usize) -> Option<usize> {
    let byte = bytes.get(index).copied()?;
    let negative = byte == b'-' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
    if !byte.is_ascii_digit() && !negative {
        return None;
    }
    if negative {
        if !is_sign_boundary(bytes, index) {
            return None;
        }
    } else {
        let previous = if index == 0 {
            None
        } else {
            bytes.get(index - 1).copied()
        };
        if previous.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.') {
            return None;
        }
    }
    Some(consume_number(bytes, index))
}

/// Consumes sign, radix prefix, digits, fraction, exponent and suffix.
fn consume_number(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start;
    if bytes.get(cursor) == Some(&b'-') {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'0')
        && matches!(
            bytes.get(cursor + 1).copied(),
            Some(b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
        )
    {
        cursor += 2;
        while bytes
            .get(cursor)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            cursor += 1;
        }
        return cursor;
    }
    while bytes
        .get(cursor)
        .is_some_and(|b| b.is_ascii_digit() || *b == b'_')
    {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'.') && bytes.get(cursor + 1).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
        while bytes
            .get(cursor)
            .is_some_and(|b| b.is_ascii_digit() || *b == b'_')
        {
            cursor += 1;
        }
    }
    if matches!(bytes.get(cursor).copied(), Some(b'e' | b'E')) {
        let mut probe = cursor + 1;
        if matches!(bytes.get(probe).copied(), Some(b'+' | b'-')) {
            probe += 1;
        }
        let digits_start = probe;
        while bytes.get(probe).is_some_and(u8::is_ascii_digit) {
            probe += 1;
        }
        if probe > digits_start {
            cursor = probe;
        }
    }
    if bytes.get(cursor).is_some_and(u8::is_ascii_alphabetic) {
        while bytes
            .get(cursor)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            cursor += 1;
        }
    }
    cursor
}

/// Whether the character before a signed number is an operator, bracket or start
/// of line — matching Python's `-?\d` behaviour without treating `x-1` as `-1`.
fn is_sign_boundary(bytes: &[u8], index: usize) -> bool {
    let mut probe = index;
    while probe > 0 {
        probe -= 1;
        let byte = bytes[probe];
        if byte.is_ascii_whitespace() {
            continue;
        }
        return matches!(
            byte,
            b'=' | b'('
                | b'['
                | b'{'
                | b','
                | b':'
                | b';'
                | b'+'
                | b'-'
                | b'*'
                | b'/'
                | b'<'
                | b'>'
                | b'&'
                | b'|'
                | b'!'
                | b'?'
        );
    }
    true
}
