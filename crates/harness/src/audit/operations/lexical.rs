use crate::HarnessError;
use std::collections::HashMap;

pub const MAX_TOKENS: usize = 500_000;

pub struct Token {
    pub text: String,
    pub start: usize,
}

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

    Ok(String::from_utf8(output).map_err(|e| HarnessError::invalid(format!(
        "invalid UTF-8 in masked source: {}", e
    )))?)
}

fn mask_range(output: &mut [u8], start: usize, end: usize) {
    for index in start..end {
        if output[index] != b'\n' {
            output[index] = b' ';
        }
    }
}

fn mask_string_literal(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    let mut cursor = position + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
            cursor += 2;
        } else if bytes[cursor] == b'"' {
            mask_range(output, position, cursor + 1);
            cursor = cursor.saturating_add(1);
            break;
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    (cursor < bytes.len() && bytes[cursor - 1] == b'"', cursor)
}

fn mask_raw_string(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    // Count # symbols
    let mut hash_count = 0;
    let mut cursor = position + 1;
    while cursor < bytes.len() && bytes[cursor] == b'#' {
        hash_count += 1;
        cursor = cursor.saturating_add(1);
    }
    // Find opening quote
    if cursor >= bytes.len() || bytes[cursor] != b'"' {
        return (false, position + 1);
    }
    let _start_quote = cursor;
    cursor = cursor.saturating_add(1);
    // Find closing quote followed by # symbols
    while cursor < bytes.len() {
        if bytes[cursor] == b'"' {
            let mut hash_cursor = cursor + 1;
            let mut closing_hash = 0;
            while hash_cursor < bytes.len() && bytes[hash_cursor] == b'#' {
                closing_hash += 1;
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
    if position + 1 >= bytes.len() {
        return (false, position + 1);
    }
    let quote = bytes[position + 1];
    if quote != b'"' && quote != b'\'' {
        return (false, position + 1);
    }
    if quote == b'"' {
        let (ok, end) = mask_string_literal(output, text, position + 1);
        if ok {
            output[position] = b' ';
            return (true, end);
        }
    }
    // Byte character literal
    if quote == b'\'' {
        let (ok, end) = mask_byte_char(output, text, position);
        return ok.then_some(end).map(|e| (true, e)).unwrap_or((false, position + 1));
    }
    (false, position + 1)
}

fn mask_byte_char(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    if position + 2 >= bytes.len() {
        return (false, position + 1);
    }
    let mut cursor = position + 2;
    if bytes[cursor] == b'\\' {
        cursor += 2;
    } else {
        cursor = cursor.saturating_add(1);
    }
    if cursor < bytes.len() && bytes[cursor] == b'\'' {
        mask_range(output, position, cursor + 1);
        return (true, cursor + 1);
    }
    (false, position + 1)
}

fn mask_char_literal(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let bytes = text.as_bytes();
    if position + 1 >= bytes.len() {
        return (false, position + 1);
    }

    let mut cursor = position + 1;

    // Check for escape sequence or single character
    if bytes[cursor] == b'\\' && cursor + 1 < bytes.len() {
        // Escape sequence
        if bytes[cursor + 1] == b'x' && cursor + 3 < bytes.len() {
            cursor += 4;
        } else if bytes[cursor + 1] == b'u' && cursor + 2 < bytes.len() && bytes[cursor + 2] == b'{' {
            let mut inner = cursor + 3;
            while inner < bytes.len() && bytes[inner] != b'}' {
                inner += 1;
            }
            if inner < bytes.len() {
                cursor = inner + 1;
            }
        } else {
            cursor += 2;
        }
    } else {
        // Single character
        cursor = cursor.saturating_add(1);
    }

    // Check for closing quote
    if cursor < bytes.len() && bytes[cursor] == b'\'' {
        mask_range(output, position, cursor + 1);
        cursor = cursor.saturating_add(1);
        return (true, cursor);
    }

    // Not a valid character literal (e.g., lifetime parameter)
    (false, position + 1)
}

fn mask_block_comment(
    output: &mut [u8],
    text: &str,
    position: usize,
) -> (bool, usize) {
    let mut cursor = position + 2;
    let mut depth: usize = 1;
    let bytes = text.as_bytes();
    while cursor < bytes.len() && depth > 0 {
        if bytes[cursor] == b'/' && cursor + 1 < bytes.len() && bytes[cursor + 1] == b'*' {
            depth = depth.saturating_add(1);
            if depth > 64 {
                return (false, cursor);
            }
            cursor += 2;
        } else if bytes[cursor] == b'*' && cursor + 1 < bytes.len() && bytes[cursor + 1] == b'/' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                mask_range(output, position, cursor + 2);
                cursor += 2;
            }
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    (depth == 0, cursor)
}

fn tokenize(text: &str, max_tokens: usize) -> Result<Vec<Token>, HarnessError> {
    let mut tokens = Vec::new();
    let mut position = 0;
    let bytes = text.as_bytes();

    while position < bytes.len() {
        let current = match bytes.get(position) {
            Some(b) => *b,
            None => break,
        };
        // Skip whitespace
        if current.is_ascii_whitespace() {
            position = position.saturating_add(1);
            continue;
        }
        // Alphanumeric identifier
        if current.is_ascii_alphanumeric() || current == b'_' {
            let start = position;
            while position < bytes.len() {
                let byte = match bytes.get(position) {
                    Some(b) => *b,
                    None => break,
                };
                if !(byte.is_ascii_alphanumeric() || byte == b'_') {
                    break;
                }
                position = position.saturating_add(1);
            }
            tokens.push(Token {
                text: text[start..position].to_string(),
                start,
            });
        }
        // Lifetime parameter: 'name
        else if current == b'\'' {
            let start = position;
            let next_pos = position.saturating_add(1);
            if next_pos < bytes.len() {
                let next_byte = bytes[next_pos];
                if next_byte.is_ascii_alphanumeric() {
                    position = next_pos;
                    while position < bytes.len() {
                        let byte = match bytes.get(position) {
                            Some(b) => *b,
                            None => break,
                        };
                        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
                            break;
                        }
                        position = position.saturating_add(1);
                    }
                    tokens.push(Token {
                        text: text[start..position].to_string(),
                        start,
                    });
                } else {
                    position = position.saturating_add(1);
                    tokens.push(Token {
                        text: text[start..position].to_string(),
                        start,
                    });
                }
            } else {
                position = position.saturating_add(1);
                tokens.push(Token {
                    text: text[start..position].to_string(),
                    start,
                });
            }
        }
        // Single character token (delimiters, operators, punctuation)
        // Multi-byte UTF-8 characters are treated as a single token
        else {
            let start = position;
            // Find the end of the UTF-8 character
            if current >= 0x80 {
                // Multi-byte character: find the end
                let mut end = position.saturating_add(1);
                while end < bytes.len() {
                    let byte = match bytes.get(end) {
                        Some(b) => *b,
                        None => break,
                    };
                    if (byte & 0xC0) != 0x80 {
                        break;
                    }
                    end = end.saturating_add(1);
                }
                position = end;
            } else {
                position = position.saturating_add(1);
            }
            tokens.push(Token {
                text: text[start..position].to_string(),
                start,
            });
        }
        if tokens.len() > max_tokens {
            return Err(HarnessError::invalid(
                "Rust token budget exceeded",
            ));
        }
    }

    Ok(tokens)
}

pub fn tokens_and_pairs(path: &str, text: &str) -> Result<(Vec<Token>, HashMap<usize, usize>), HarnessError> {
    tokens_and_pairs_with_budget(path, text, MAX_TOKENS)
}

pub fn tokens_and_pairs_with_budget(path: &str, text: &str, max_tokens: usize) -> Result<(Vec<Token>, HashMap<usize, usize>), HarnessError> {
    let tokens = tokenize(text, max_tokens)?;
    let mut stack: Vec<usize> = Vec::new();
    let mut pairs = HashMap::new();
    let mut position = 0;

    while position < tokens.len() {
        let token = &tokens[position];
        if token.text == "(" || token.text == "[" || token.text == "{" {
            stack.push(position);
        } else if token.text == ")" || token.text == "]" || token.text == "}" {
            let expected = match token.text.as_str() {
                ")" => "(",
                "]" => "[",
                _ => "{",
            };
            if stack.is_empty() {
                let (line, column) = source_location(text, token.start);
                return Err(HarnessError::invalid(format!(
                    "{}:{}:{}: unexpected closing Rust delimiter {}",
                    path, line, column, token.text
                )));
            }
            if let Some(open) = stack.pop() {
                if tokens[open].text != expected {
                    let (line, column) = source_location(text, token.start);
                    let (open_line, open_column) = source_location(text, tokens[open].start);
                    return Err(HarnessError::invalid(format!(
                        "{}:{}:{}: unbalanced Rust delimiters: {} closes {} opened at {}:{}; expected {}",
                        path, line, column, token.text, tokens[open].text,
                        open_line, open_column,
                        match expected {
                            "(" => ")",
                            "[" => "]",
                            _ => "}",
                        }
                    )));
                }
                pairs.insert(position, open);
                pairs.insert(open, position);
            }
        }
        position = position.saturating_add(1);
    }

    if let Some(open) = stack.last() {
        let open_token = match tokens.get(*open) {
            Some(t) => t,
            None => {
                return Err(HarnessError::invalid(format!(
                    "{}: unclosed Rust delimiter; stack corruption at token index {}",
                    path, open
                )));
            }
        };
        let (line, column) = source_location(text, open_token.start);
        let expected = match open_token.text.as_str() {
            "(" => ")",
            "[" => "]",
            _ => "}",
        };
        return Err(HarnessError::invalid(format!(
            "{}:{}:{}: unclosed Rust delimiter '{}'; expected '{}'",
            path, line, column, open_token.text, expected
        )));
    }

    Ok((tokens, pairs))
}
