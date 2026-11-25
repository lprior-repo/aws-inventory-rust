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
        if current.is_ascii_whitespace() {
            position = position.saturating_add(1);
            continue;
        }
        if current.is_ascii_alphanumeric() || current == b'_' {
            let start = position;
            skip_word_bytes(bytes, &mut position);
            tokens.push(token_between(text, start, position));
        } else if current == b'\'' {
            scan_quote_token(text, bytes, &mut tokens, &mut position);
        } else {
            let start = position;
            scan_single_char(bytes, current, &mut position);
            tokens.push(token_between(text, start, position));
        }
        if tokens.len() > max_tokens {
            return Err(HarnessError::invalid("Rust token budget exceeded"));
        }
    }

    Ok(tokens)
}

fn token_between(text: &str, start: usize, end: usize) -> Token {
    Token {
        text: text[start..end].to_string(),
        start,
    }
}

fn skip_word_bytes(bytes: &[u8], position: &mut usize) {
    while *position < bytes.len() {
        let byte = match bytes.get(*position) {
            Some(b) => *b,
            None => break,
        };
        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
            break;
        }
        *position = position.saturating_add(1);
    }
}

fn scan_quote_token(text: &str, bytes: &[u8], tokens: &mut Vec<Token>, position: &mut usize) {
    let start = *position;
    let next_pos = position.saturating_add(1);
    *position = next_pos;
    if bytes.get(next_pos).is_some_and(u8::is_ascii_alphanumeric) {
        skip_word_bytes(bytes, position);
    }
    tokens.push(token_between(text, start, *position));
}

fn scan_single_char(bytes: &[u8], current: u8, position: &mut usize) {
    if current >= 0x80 {
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
        *position = end;
    } else {
        *position = position.saturating_add(1);
    }
}

pub fn tokens_and_pairs(
    path: &str,
    text: &str,
) -> Result<(Vec<Token>, HashMap<usize, usize>), HarnessError> {
    tokens_and_pairs_with_budget(path, text, MAX_TOKENS)
}

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
            pair_closing_delimiter(path, text, &tokens, &mut stack, &mut pairs, position, token)?;
        }
        position = position.saturating_add(1);
    }

    reject_unclosed_delimiter(path, text, &tokens, &stack)?;
    Ok((tokens, pairs))
}

fn pair_closing_delimiter(
    path: &str,
    text: &str,
    tokens: &[Token],
    stack: &mut Vec<usize>,
    pairs: &mut HashMap<usize, usize>,
    position: usize,
    token: &Token,
) -> Result<(), HarnessError> {
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
    Ok(())
}

fn reject_unclosed_delimiter(
    path: &str,
    text: &str,
    tokens: &[Token],
    stack: &[usize],
) -> Result<(), HarnessError> {
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
    Ok(())
}
