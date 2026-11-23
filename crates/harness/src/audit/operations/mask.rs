/// Mask non-code content (strings, comments, literals) with spaces.
use crate::HarnessError;

fn source_location(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset];
    let line = before
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        .saturating_add(1);
    let line_start = before.rfind('\n').map(|i| i.saturating_add(1)).unwrap_or(0);
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
        position = mask_step(path, text, &mut output, position, current, next_byte)?;
    }

    String::from_utf8(output)
        .map_err(|e| HarnessError::invalid(format!("invalid UTF-8 in masked source: {}", e)))
}

fn mask_step(
    path: &str,
    text: &str,
    output: &mut [u8],
    position: usize,
    current: u8,
    next_byte: u8,
) -> Result<usize, HarnessError> {
    // Line comment
    if current == b'/' && next_byte == b'/' {
        return Ok(mask_line_comment(output, text, position));
    }
    // Block comment
    if current == b'/' && next_byte == b'*' {
        let (ok, end) = mask_block_comment(output, text, position);
        if !ok {
            let (line, col) = source_location(text, position);
            return Err(HarnessError::invalid(format!(
                "{}:{}:{}: unterminated Rust block comment",
                path, line, col
            )));
        }
        return Ok(end);
    }
    // String literal
    if current == b'"' {
        let (ok, end) = mask_string_literal(output, text, position);
        if !ok {
            let (line, col) = source_location(text, position);
            return Err(HarnessError::invalid(format!(
                "{}:{}:{}: unterminated Rust string",
                path, line, col
            )));
        }
        return Ok(end);
    }
    Ok(mask_literal_step(output, text, position, current))
}

fn mask_line_comment(output: &mut [u8], text: &str, position: usize) -> usize {
    let bytes = text.as_bytes();
    let remaining = &text[position..];
    let line_end = remaining.find('\n');
    let end = match line_end {
        Some(offset) => position.saturating_add(offset),
        None => bytes.len(),
    };
    mask_range(output, position, end);
    end
}

fn mask_literal_step(output: &mut [u8], text: &str, position: usize, current: u8) -> usize {
    let next_pos = position.saturating_add(1);
    // Raw string
    if current == b'r' {
        let (ok, end) = mask_raw_string(output, text, position);
        if ok { end } else { next_pos }
    }
    // Byte string
    else if current == b'b' {
        let (ok, end) = mask_byte_string(output, text, position);
        if ok { end } else { next_pos }
    }
    // Character literal
    else if current == b'\'' {
        let (ok, end) = mask_char_literal(output, text, position);
        if !ok {
            // Not a char literal (e.g. lifetime parameter)
            next_pos
        } else {
            end
        }
    } else {
        next_pos
    }
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

fn mask_string_literal(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
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

fn mask_raw_string(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
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

fn mask_byte_string(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
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

fn mask_byte_char(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
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

fn mask_char_literal(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
    let bytes = text.as_bytes();
    let mut cursor = position.saturating_add(1);
    let Some(first) = bytes.get(cursor) else {
        return (false, cursor);
    };
    let after = cursor.saturating_add(1);
    if *first == b'\\' && bytes.get(after).is_some() {
        let escape = bytes.get(after).copied();
        if escape == Some(b'x') && bytes.get(cursor.saturating_add(3)).is_some() {
            cursor = cursor.saturating_add(4);
        } else if escape == Some(b'u') && bytes.get(cursor.saturating_add(2)) == Some(&b'{') {
            let mut inner = cursor.saturating_add(3);
            while bytes.get(inner).is_some_and(|byte| *byte != b'}') {
                inner = inner.saturating_add(1);
            }
            if bytes.get(inner).is_some() {
                cursor = inner.saturating_add(1);
            }
        } else {
            cursor = cursor.saturating_add(2);
        }
    } else {
        cursor = cursor.saturating_add(1);
    }
    if bytes.get(cursor) == Some(&b'\'') {
        let end = cursor.saturating_add(1);
        mask_range(output, position, end);
        return (true, end);
    }
    (false, position.saturating_add(1))
}

fn mask_block_comment(output: &mut [u8], text: &str, position: usize) -> (bool, usize) {
    let mut cursor = position.saturating_add(2);
    let mut depth: usize = 1;
    let bytes = text.as_bytes();
    while cursor < bytes.len() && depth > 0 {
        let pair = (
            bytes.get(cursor).copied(),
            bytes.get(cursor.saturating_add(1)).copied(),
        );
        if pair == (Some(b'/'), Some(b'*')) {
            depth = depth.saturating_add(1);
            if depth > 64 {
                return (false, cursor);
            }
            cursor = cursor.saturating_add(2);
        } else if pair == (Some(b'*'), Some(b'/')) {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                cursor = cursor.saturating_add(2);
                mask_range(output, position, cursor);
            }
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    (depth == 0, cursor)
}
