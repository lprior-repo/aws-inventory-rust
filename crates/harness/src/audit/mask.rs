//! Bounded lexical masking for Rust sources.
//!
//! Strings, characters and nested comments are blanked without moving any
//! newline, so token offsets still refer to the original source. Every failure
//! carries the original character offset for path/line/column reporting.

use crate::HarnessError;

/// Maximum nested block-comment depth.
const MAX_COMMENT_DEPTH: usize = 64;

/// A bounded lexical failure with its original character offset.
pub struct LexError {
    /// Operator-facing reason.
    pub message: String,
    /// Character offset in the original source.
    pub offset: usize,
}

impl LexError {
    pub(crate) fn new(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

/// Line and column of a character offset, both one-based.
pub fn location(source: &[char], offset: usize) -> (usize, usize) {
    let limit = offset.min(source.len());
    let mut line = 1_usize;
    let mut column = 1_usize;
    for character in source.iter().take(limit) {
        if *character == '\n' {
            line = line.saturating_add(1);
            column = 1;
        } else {
            column = column.saturating_add(1);
        }
    }
    (line, column)
}

/// Render a lexical failure with its path, line and column.
pub fn source_error(path: &str, source: &[char], offset: usize, message: &str) -> HarnessError {
    let (line, column) = location(source, offset);
    HarnessError::invalid(format!("{path}:{line}:{column}: {message}"))
}

pub(crate) fn begins(chars: &[char], position: usize, text: &str) -> bool {
    text.chars().enumerate().all(|(index, expected)| {
        chars
            .get(position.saturating_add(index))
            .map(|value| *value == expected)
            .unwrap_or(false)
    })
}

fn find_from(chars: &[char], from: usize, needle: &[char]) -> Option<usize> {
    let mut position = from;
    while position.saturating_add(needle.len()) <= chars.len() {
        if needle.iter().enumerate().all(|(index, expected)| {
            chars
                .get(position.saturating_add(index))
                .map(|value| value == expected)
                .unwrap_or(false)
        }) {
            return Some(position);
        }
        position = position.saturating_add(1);
    }
    None
}

/// Content start and hash count of a raw-string prefix at `position`.
fn raw_prefix(chars: &[char], position: usize) -> Option<(usize, usize)> {
    let byte_prefix = matches!(chars.get(position), Some('b' | 'c'))
        && chars.get(position.saturating_add(1)) == Some(&'r');
    let prefix = if byte_prefix {
        2_usize
    } else if chars.get(position) == Some(&'r') {
        1_usize
    } else {
        return None;
    };
    let mut cursor = position.saturating_add(prefix);
    let mut hashes = 0_usize;
    while chars.get(cursor) == Some(&'#') {
        hashes = hashes.saturating_add(1);
        cursor = cursor.saturating_add(1);
    }
    if hashes > 255 || chars.get(cursor) != Some(&'"') {
        return None;
    }
    Some((cursor.saturating_add(1), hashes))
}

fn is_hex(character: char) -> bool {
    character.is_ascii_hexdigit()
}

fn hex_escape_end(chars: &[char], escaped: usize) -> Option<usize> {
    let high = *chars.get(escaped.saturating_add(1))?;
    let low = *chars.get(escaped.saturating_add(2))?;
    if is_hex(high) && is_hex(low) {
        Some(escaped.saturating_add(3))
    } else {
        Some(escaped.saturating_add(1))
    }
}

fn unicode_escape_end(chars: &[char], marker: usize) -> Option<usize> {
    let brace = *chars.get(marker.saturating_add(1))?;
    if brace != '{' {
        return Some(marker.saturating_add(1));
    }
    let digits_start = marker.saturating_add(2);
    let mut end = digits_start;
    while chars.get(end).map(|value| is_hex(*value) || *value == '_') == Some(true) {
        end = end.saturating_add(1);
    }
    let digits = end.saturating_sub(digits_start);
    if (1..=6).contains(&digits) && chars.get(end) == Some(&'}') {
        Some(end.saturating_add(1))
    } else {
        Some(marker.saturating_add(1))
    }
}

/// Cursor position after the escape sequence starting at `backslash`.
fn escape_end(chars: &[char], backslash: usize) -> Option<usize> {
    let escaped = backslash.saturating_add(1);
    match *chars.get(escaped)? {
        'x' => hex_escape_end(chars, escaped),
        'u' => unicode_escape_end(chars, escaped),
        '\n' => None,
        _ => Some(escaped.saturating_add(1)),
    }
}

/// End offset of a character literal starting at `position`.
fn character_end(chars: &[char], position: usize) -> Option<usize> {
    let mut cursor = position.saturating_add(1);
    match chars.get(cursor) {
        Some('\\') => cursor = escape_end(chars, cursor)?,
        Some(character) if *character != '\\' && *character != '\'' && *character != '\n' => {
            cursor = cursor.saturating_add(1);
        }
        _ => return None,
    }
    if chars.get(cursor) == Some(&'\'') {
        Some(cursor.saturating_add(1))
    } else {
        None
    }
}

fn line_comment_end(chars: &[char], position: usize) -> usize {
    let mut cursor = position;
    while cursor < chars.len() && chars.get(cursor) != Some(&'\n') {
        cursor = cursor.saturating_add(1);
    }
    cursor
}

fn comment_depth(depth: usize, cursor: usize) -> Result<usize, LexError> {
    let next = depth.saturating_add(1);
    if next > MAX_COMMENT_DEPTH {
        return Err(LexError::new(
            "Rust comment nesting exceeds audit bound",
            cursor,
        ));
    }
    Ok(next)
}

fn block_comment_end(chars: &[char], position: usize) -> Result<usize, LexError> {
    let mut depth = 1_usize;
    let mut cursor = position.saturating_add(2);
    while cursor < chars.len() && depth > 0 {
        if begins(chars, cursor, "/*") {
            depth = comment_depth(depth, cursor)?;
            cursor = cursor.saturating_add(2);
        } else if begins(chars, cursor, "*/") {
            depth = depth.saturating_sub(1);
            cursor = cursor.saturating_add(2);
        } else {
            cursor = cursor.saturating_add(1);
        }
    }
    if depth > 0 {
        return Err(LexError::new("unterminated Rust block comment", position));
    }
    Ok(cursor)
}

fn guarded(chars: &[char], position: usize) -> bool {
    position == 0
        || !chars
            .get(position.saturating_sub(1))
            .map(|value| value.is_alphanumeric() || *value == '_')
            .unwrap_or(false)
}

fn raw_string_end(chars: &[char], position: usize) -> Result<Option<usize>, LexError> {
    let Some((content, hashes)) = raw_prefix(chars, position) else {
        return Ok(None);
    };
    let mut ending = vec!['"'];
    ending.extend(std::iter::repeat_n('#', hashes));
    let found = find_from(chars, content, &ending)
        .ok_or_else(|| LexError::new("unterminated Rust raw string", position))?;
    Ok(Some(found.saturating_add(ending.len())))
}

fn string_end(chars: &[char], position: usize) -> Result<usize, LexError> {
    let mut cursor = position.saturating_add(1);
    while cursor < chars.len() {
        match chars.get(cursor) {
            Some('\\') => cursor = cursor.saturating_add(2),
            Some('"') => return Ok(cursor.saturating_add(1)),
            Some(_) => cursor = cursor.saturating_add(1),
            None => break,
        }
    }
    Err(LexError::new("unterminated Rust string", position))
}

/// End offset of the literal starting at `position`, if there is one.
fn literal_end(chars: &[char], position: usize) -> Result<Option<usize>, LexError> {
    if guarded(chars, position)
        && let Some(end) = raw_string_end(chars, position)?
    {
        return Ok(Some(end));
    }
    match chars.get(position) {
        Some('"') => Ok(Some(string_end(chars, position)?)),
        Some('\'') => Ok(character_end(chars, position)),
        _ => Ok(None),
    }
}

/// End offset of the masked region starting at `position`.
fn region(chars: &[char], position: usize) -> Result<Option<usize>, LexError> {
    if begins(chars, position, "//") {
        return Ok(Some(line_comment_end(chars, position)));
    }
    if begins(chars, position, "/*") {
        return block_comment_end(chars, position).map(Some);
    }
    literal_end(chars, position)
}

fn blank(output: &mut [char], start: usize, finish: usize) {
    for index in start..finish {
        if let Some(slot) = output.get_mut(index)
            && *slot != '\n'
        {
            *slot = ' ';
        }
    }
}

/// Blank strings, characters and nested comments while preserving offsets.
pub fn mask(chars: &[char]) -> Result<Vec<char>, LexError> {
    let mut output = chars.to_vec();
    let mut position = 0_usize;
    while position < chars.len() {
        let start = position;
        match region(chars, position)? {
            None => position = position.saturating_add(1),
            Some(finish) => {
                blank(&mut output, start, finish);
                position = finish;
            }
        }
    }
    Ok(output)
}
