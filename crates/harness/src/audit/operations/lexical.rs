/// Rust tokenizer for AWS SDK call analysis.
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
    let line_start = before.rfind('\n').map(|i| i.saturating_add(1)).unwrap_or(0);
    let column = offset.saturating_sub(line_start).saturating_add(1);
    (line, column)
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
            if let Some(next_byte) = bytes.get(next_pos) {
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
            return Err(HarnessError::invalid("Rust token budget exceeded"));
        }
    }

    Ok(tokens)
}

/// Tokenize Rust source and pair matching delimiters.
pub fn tokens_and_pairs(
    path: &str,
    text: &str,
) -> Result<(Vec<Token>, HashMap<usize, usize>), HarnessError> {
    tokens_and_pairs_with_budget(path, text, MAX_TOKENS)
}

/// Tokenize with explicit token budget.
pub fn tokens_and_pairs_with_budget(
    path: &str,
    text: &str,
    max_tokens: usize,
) -> Result<(Vec<Token>, HashMap<usize, usize>), HarnessError> {
    let tokens = tokenize(text, max_tokens)?;
    let mut stack: Vec<usize> = Vec::new();
    let mut pairs = HashMap::new();
    let mut position = 0;

    while let Some(token) = tokens.get(position) {
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
                let open_token = match tokens.get(open) {
                    Some(t) => t,
                    None => {
                        return Err(HarnessError::invalid(format!(
                            "{}: unbalanced Rust delimiters; stack corruption at token index {}",
                            path, open
                        )));
                    }
                };
                if open_token.text != expected {
                    let (line, column) = source_location(text, token.start);
                    let (open_line, open_column) = source_location(text, open_token.start);
                    return Err(HarnessError::invalid(format!(
                        "{}:{}:{}: unbalanced Rust delimiters: {} closes {} opened at {}:{}; expected {}",
                        path,
                        line,
                        column,
                        token.text,
                        open_token.text,
                        open_line,
                        open_column,
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
