use super::DecodeError;
use std::borrow::Cow;

#[derive(Default)]
struct Scanner {
    quoted: bool,
    escaped: bool,
    quoted_item: bool,
    date_comma: bool,
}

impl Scanner {
    fn separator(&mut self, character: char, date: bool) -> bool {
        if self.escaped {
            self.escaped = false;
            return false;
        }
        if self.quoted && character == '\\' {
            self.escaped = true;
            return false;
        }
        if character == '"' {
            self.quoted = !self.quoted;
            self.quoted_item = true;
            return false;
        }
        if character != ',' || self.quoted {
            return false;
        }
        if date && !self.quoted_item && !self.date_comma {
            self.date_comma = true;
            return false;
        }
        self.date_comma = false;
        self.quoted_item = false;
        true
    }
}

pub(super) fn visit<F>(text: &str, date: bool, mut visit: F) -> Result<(), DecodeError>
where
    F: FnMut(&str) -> Result<(), DecodeError>,
{
    let mut scanner = Scanner::default();
    let mut start = 0usize;
    for (index, character) in text.char_indices() {
        if scanner.separator(character, date) {
            let item = unquote(text.get(start..index).ok_or_else(invalid)?.trim())?;
            visit(item.as_ref())?;
            start = index.checked_add(1).ok_or_else(invalid)?;
        }
    }
    if scanner.quoted || scanner.escaped {
        return Err(invalid());
    }
    let item = unquote(text.get(start..).ok_or_else(invalid)?.trim())?;
    visit(item.as_ref())
}

fn unquote(item: &str) -> Result<Cow<'_, str>, DecodeError> {
    if !item.starts_with('"') && !item.ends_with('"') {
        return Ok(Cow::Borrowed(item));
    }
    if !item.starts_with('"') || !item.ends_with('"') || item.len() < 2 {
        return Err(invalid());
    }
    let end = item.len().checked_sub(1).ok_or_else(invalid)?;
    let contents = item.get(1..end).ok_or_else(invalid)?;
    if !contents.contains('\\') {
        return Ok(Cow::Borrowed(contents));
    }
    let mut decoded = String::new();
    decoded.try_reserve_exact(contents.len())?;
    let mut chars = contents.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            decoded.push(chars.next().ok_or_else(invalid)?);
        } else {
            decoded.push(character);
        }
    }
    Ok(Cow::Owned(decoded))
}

fn invalid() -> DecodeError {
    DecodeError::Invalid("HTTP header list")
}
