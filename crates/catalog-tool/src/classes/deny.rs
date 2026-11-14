use super::{DenyReason, OperationFacts};
use thiserror::Error;

/// A canonical must-deny syntax failure, with a one-based line number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("must-deny line {line}: {kind:?}")]
pub struct ParseError {
    /// One-based offending line; whole-file size errors use line one.
    pub line: usize,
    /// Machine-readable cause.
    pub kind: ParseErrorKind,
}

/// Rejections of noncanonical or excessive deny-list input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// Keys must be ascending, ignoring comment lines.
    Unsorted,
    /// A key must occur only once, even with different reasons.
    Duplicate,
    /// Empty lines are forbidden.
    BlankLine,
    /// An entry needs a tab separator.
    MissingTab,
    /// Reasons are exactly the five documented lowercase labels.
    UnknownReason,
    /// Every nonempty file ends in LF.
    MissingFinalNewline,
    /// CR, including CRLF, is forbidden.
    CarriageReturn,
    /// Expected an ASCII operation identifier or service/operation key.
    InvalidKey,
    /// Exceeds 1 MiB, 4,096 entries, or allocation capacity.
    Limit,
}

#[derive(Debug)]
pub(crate) struct Entry<'a> {
    pub key: &'a str,
    pub reason: DenyReason,
    pub line: usize,
}

impl Entry<'_> {
    pub fn matches(&self, facts: &OperationFacts<'_>) -> bool {
        match self.key.split_once('/') {
            Some((service, name)) => service == facts.service && name == facts.name,
            None => self.key == facts.name,
        }
    }
}

/// Validated, borrowed entries; parsing allocates only the bounded entry vector.
#[derive(Debug)]
pub struct DenyList<'a>(pub(crate) Vec<Entry<'a>>);

impl<'a> DenyList<'a> {
    /// Parse strict LF text, ignoring `#` comments; empty text is an empty list.
    /// # Errors
    /// Returns the one-based offending line and typed syntax/resource cause.
    pub fn parse(text: &'a str) -> Result<Self, ParseError> {
        validate_text(text)?;
        text.split_terminator('\n')
            .enumerate()
            .try_fold(Self(Vec::new()), |mut list, (i, row)| {
                list.add_line(row, i.saturating_add(1))?;
                Ok(list)
            })
    }

    /// Number of rules, excluding comments; at most 4,096.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn reason(&self, facts: &OperationFacts<'_>) -> Option<DenyReason> {
        self.0
            .iter()
            .filter(|e| e.matches(facts))
            .max_by_key(|e| e.key.contains('/'))
            .map(|e| e.reason)
    }

    fn add_line(&mut self, row: &'a str, line: usize) -> Result<(), ParseError> {
        if row.contains('\r') {
            return Err(failure(line, ParseErrorKind::CarriageReturn));
        }
        if row.starts_with('#') {
            return Ok(());
        }
        let entry = parse_entry(row, line)?;
        self.check_order(&entry)?;
        if self.len() == 4096 {
            return Err(failure(line, ParseErrorKind::Limit));
        }
        self.0
            .try_reserve(1)
            .map_err(|_| failure(line, ParseErrorKind::Limit))?;
        self.0.push(entry);
        Ok(())
    }

    fn check_order(&self, entry: &Entry<'_>) -> Result<(), ParseError> {
        if self.0.iter().any(|e| e.key == entry.key) {
            return Err(failure(entry.line, ParseErrorKind::Duplicate));
        }
        if self.0.last().is_some_and(|e| entry.key < e.key) {
            return Err(failure(entry.line, ParseErrorKind::Unsorted));
        }
        Ok(())
    }
}

fn validate_text(text: &str) -> Result<(), ParseError> {
    if text.len() > 1_048_576 {
        return Err(failure(1, ParseErrorKind::Limit));
    }
    if !text.is_empty() && !text.ends_with('\n') {
        return Err(failure(
            text.lines().count(),
            ParseErrorKind::MissingFinalNewline,
        ));
    }
    Ok(())
}

fn parse_entry(row: &str, line: usize) -> Result<Entry<'_>, ParseError> {
    if row.is_empty() {
        return Err(failure(line, ParseErrorKind::BlankLine));
    }
    let (key, reason) = row
        .split_once('\t')
        .ok_or_else(|| failure(line, ParseErrorKind::MissingTab))?;
    if !valid_key(key) {
        return Err(failure(line, ParseErrorKind::InvalidKey));
    }
    Ok(Entry {
        key,
        reason: parse_reason(reason, line)?,
        line,
    })
}

fn valid_key(key: &str) -> bool {
    let identifier = |s: &str| {
        s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    };
    match key.split_once('/') {
        Some((service, name)) => {
            !service.is_empty()
                && service
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
                && identifier(name)
        }
        None => identifier(key),
    }
}

fn parse_reason(reason: &str, line: usize) -> Result<DenyReason, ParseError> {
    match reason {
        "content" => Ok(DenyReason::Content),
        "secret" => Ok(DenyReason::Secret),
        "paid" => Ok(DenyReason::Paid),
        "mutating" => Ok(DenyReason::Mutating),
        "reference-data" => Ok(DenyReason::ReferenceData),
        _ => Err(failure(line, ParseErrorKind::UnknownReason)),
    }
}

const fn failure(line: usize, kind: ParseErrorKind) -> ParseError {
    ParseError { line, kind }
}
