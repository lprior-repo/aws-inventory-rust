/// Mask non-code content (strings, comments, literals) with spaces.
use crate::HarnessError;

fn source_location(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset];
    let line = before
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        .saturating_add(1);
    let line_start = before
        .rfind('\n')
        .map(|i| i.saturating_add(1))
        .unwrap_or(0);
    let column = offset.saturating_sub(line_start).saturating_add(1);
    (line, column)
}

pub fn mask_noncode(path: &str, text: &str) -> Result<String, HarnessError> {
    let mut output = text.as_bytes().to_vec();
    let bytes = text.as_bytes();
    let mut position = 0;

    while position < bytes.len() {
        let current = match bytes.get(position) {
            Some(b) => *b,
            None => break,
        };
        let next_pos = position.saturating_add(1);
        let next_byte = match bytes.get(next_pos) {
            Some(b) => *b,
            None => {
                position = next_pos;
                continue;
            }
        };
        // Line comment
        if current == b'/' && next_byte == b'/' {
            let remaining = &text[position..];
            let line_end = remaining.find('\n');
            let end = match line_end {
                Some(offset) => position.saturating_add(offset),
                None => bytes.len(),
            };
            mask_range(&mut output, position, end);
            position = end;
        }
        // Block comment
        else if current == b'/' && next_byte == b'*' {
            let (ok, end) = mask_block_comment(&mut output, text, position);
            if !ok {
                let (line, col) = source_location(text, position);
                return Err(HarnessError::invalid(format!(
                    "{}:{}:{}: unterminated Rust block comment",
                    path, line, col
                )));
            }
            position = end;
        }
        // String literal
        else if current == b'"' {
            let (ok, end) = mask_string_literal(&mut output, text, position);
            if !ok {
                let (line, col) = source_location(text, position);
                return Err(HarnessError::invalid(format!(
                    "{}:{}:{}: unterminated Rust string",
                    path, line, col
                )));
            }
            position = end;
        }
        // Raw string
        else if current == b'r' {
            let (ok, end) = mask_raw_string(&mut output, text, position);
            if ok {
                position = end;
            } else {
                position = next_pos;
            }
        }
        // Byte string
        else if current == b'b' {
            let (ok, end) = mask_byte_string(&mut output, text, position);
            if ok {
                position = end;
            } else {
                position = next_pos;
            }
        }
        // Character literal
        else if current == b'\'' {
            let (ok, end) = mask_char_literal(&mut output, text, position);
            if !ok {
                // Not a char literal (e.g. lifetime parameter)
                position = next_pos;
            } else {
                position = end;
            }
        } else {
            position = next_pos;
        }
    }

    String::from_utf8(output).map_err(|e| HarnessError::invalid(format!(
        "invalid UTF-8 in masked source: {}", e
    )))
}

fn mask_range(output: &mut [u8], start: usize, end: usize) {
    let slice = output.get_mut(start..end);
    if let Some(s) = slice {
        for b in s.iter_mut() {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
}

fn mask_string_literal(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    let mut cursor = position.saturating_add(1);
    while let Some(b) = bytes.get(cursor) {
        if *b == b'\\' {
            cursor = cursor.saturating_add(2);
        } else if *b == b'"' {
            mask_range(output, position, cursor.saturating_add(1));
            cursor = cursor.saturating_add(1);
            return (true, cursor);
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    let found_close = bytes.get(cursor.saturating_sub(1)) == Some(&b'"');
    (found_close, cursor)
}

fn mask_raw_string(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    // Count # symbols
    let mut hash_count: usize = 0;
    let mut cursor = position.saturating_add(1);
    while bytes.get(cursor) == Some(&b'#') {
        hash_count = hash_count.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }
    // Find opening quote
    if bytes.get(cursor) != Some(&b'"') {
        return (false, position.saturating_add(1));
    }
    cursor = cursor.saturating_add(1);
    // Find closing quote followed by # symbols
    while let Some(b) = bytes.get(cursor) {
        if *b == b'"' {
            let mut hash_cursor = cursor.saturating_add(1);
            let mut closing_hash: usize = 0;
            while bytes.get(hash_cursor) == Some(&b'#') {
                closing_hash = closing_hash.saturating_add(1);
                hash_cursor = hash_cursor.saturating_add(1);
            }
            if closing_hash == hash_count {
                mask_range(output, position, hash_cursor);
                return (true, hash_cursor);
            }
        }
        cursor = cursor.saturating_add(1);
    }
    (false, cursor)
}

fn mask_byte_string(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    // Check for b" or b'
    let next = position.saturating_add(1);
    let quote = bytes.get(next);
    if quote != Some(&b'"') && quote != Some(&b'\'') {
        return (false, next);
    }
    if quote == Some(&b'"') {
        let (ok, end) = mask_string_literal(output, text, next);
        if ok {
            if let Some(b) = output.get_mut(position) {
                *b = b' ';
            }
            return (true, end);
        }
    }
    // Byte character literal
    if quote == Some(&b'\'') {
        let (ok, end) = mask_byte_char(output, text, position);
        if ok {
            return (true, end);
        }
    }
    (false, next)
}

fn mask_byte_char(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    let start = position.saturating_add(2);
    if bytes.get(start).is_none() {
        return (false, position.saturating_add(1));
    }
    let mut cursor = start;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor = cursor.saturating_add(2);
    } else {
        cursor = cursor.saturating_add(1);
    }
    if bytes.get(cursor) == Some(&b'\'') {
        mask_range(output, position, cursor.saturating_add(1));
        return (true, cursor.saturating_add(1));
    }
    (false, position.saturating_add(1))
}

fn mask_char_literal(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    let mut cursor = position.saturating_add(1);
    if bytes.get(cursor).is_none() {
        return (false, position.saturating_add(1));
    }

    // Check for escape sequence or single character
    if bytes.get(cursor) == Some(&b'\\') {
        // Escape sequence
        let next = cursor.saturating_add(1);
        if bytes.get(next) == Some(&b'x') {
            cursor = cursor.saturating_add(4);
        } else if bytes.get(next) == Some(&b'u') {
            let inner_start = cursor.saturating_add(3);
            let mut inner = inner_start;
            while bytes.get(inner) != Some(&b'}') {
                inner = inner.saturating_add(1);
            }
            if bytes.get(inner).is_some() {
                cursor = inner.saturating_add(1);
            }
        } else {
            cursor = cursor.saturating_add(2);
        }
    } else {
        // Single character
        cursor = cursor.saturating_add(1);
    }

    // Check for closing quote
    if bytes.get(cursor) == Some(&b'\'') {
        mask_range(output, position, cursor.saturating_add(1));
        cursor = cursor.saturating_add(1);
        return (true, cursor);
    }

    // Not a valid character literal (e.g., lifetime parameter)
    (false, position.saturating_add(1))
}

fn mask_block_comment(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let mut cursor = position.saturating_add(2);
    let mut depth: usize = 1;
    let bytes = text.as_bytes();
    while depth > 0 {
        if bytes.get(cursor) == Some(&b'/') && bytes.get(cursor.saturating_add(1)) == Some(&b'*') {
            depth = depth.saturating_add(1);
            if depth > 64 {
                return (false, cursor);
            }
            cursor = cursor.saturating_add(2);
        } else if bytes.get(cursor) == Some(&b'*') && bytes.get(cursor.saturating_add(1)) == Some(&b'/') {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                mask_range(output, position, cursor.saturating_add(2));
                cursor = cursor.saturating_add(2);
            }
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    (true, cursor)
}