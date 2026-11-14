//! Append-only field-ID registry: names become stable 1-based line numbers.
//!
//! The registry is UTF-8, one non-empty valid name per LF-terminated line.
//! IDs are the 1-based line numbers, never reused. Bootstrap sorts names
//! bytewise. Regeneration preserves existing order and appends only unseen
//! names in sorted order. Previous registries must be prefixes of new ones.

use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU32;

#[cfg(test)]
mod tests;

/// Failures from registry parsing, validation, or prefix checks.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The text is non-empty but does not end with a newline.
    #[error("registry is missing its final newline")]
    MissingFinalNewline,
    /// A line contains a carriage return.
    #[error("carriage return on registry line {line}")]
    CarriageReturn { line: usize },
    /// A name contains invalid characters.
    #[error("invalid member name on registry line {line}: {name:?}")]
    InvalidName { line: usize, name: String },
    /// The same name appears more than once.
    #[error("duplicate member name on registry line {line}: {name:?}")]
    Duplicate { line: usize, name: String },
    /// The line count exceeds u32.
    #[error("registry exceeds the nonzero u32 ID range")]
    IdOverflow,
    /// A previous name is missing or replaced at a specific line.
    #[error("registry prefix changed on line {line}: expected {expected:?}, found {found:?}")]
    PrefixChanged {
        line: usize,
        expected: String,
        found: Option<String>,
    },
}

/// A validated, append-only registry of field names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registry {
    names: Vec<String>,
    ids: HashMap<String, NonZeroU32>,
}

impl Registry {
    /// Parse a registry from a UTF-8 string.
    ///
    /// Empty text is the empty registry. Otherwise the text must end with `\n`,
    /// and every line must be a valid Smithy member identifier.
    ///
    /// # Errors
    /// Rejects malformed names, duplicate lines, invalid line endings, and ID overflow.
    pub fn parse(text: &str) -> Result<Self, Error> {
        if text.is_empty() {
            return Ok(Self::default());
        }
        let text = text.strip_suffix('\n').ok_or(Error::MissingFinalNewline)?;
        text.split('\n')
            .try_fold(Self::default(), |mut registry, name| {
                registry.push_name(name.to_owned())?;
                Ok(registry)
            })
    }

    /// The number of names in this registry.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether this registry is empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Look up the stable 1-based ID of a name.
    pub fn id(&self, name: &str) -> Option<NonZeroU32> {
        self.ids.get(name).copied()
    }

    /// Return the name at a 1-based ID.
    pub fn name(&self, id: NonZeroU32) -> Option<&str> {
        let index = usize::try_from(id.get()).ok()?.checked_sub(1)?;
        self.names.get(index).map(String::as_str)
    }

    /// Append unseen names in deterministic sorted order.
    ///
    /// Existing names keep their positions. Returns the new registry.
    ///
    /// # Errors
    /// Rejects invalid names and more than `u32::MAX` distinct entries.
    pub fn extend(self, names: impl IntoIterator<Item = String>) -> Result<Registry, Error> {
        self.unseen_names(names)?
            .into_iter()
            .try_fold(self, |mut registry, name| {
                registry.push_name(name)?;
                Ok(registry)
            })
    }

    fn unseen_names(
        &self,
        names: impl IntoIterator<Item = String>,
    ) -> Result<BTreeSet<String>, Error> {
        names
            .into_iter()
            .try_fold(BTreeSet::new(), |mut unseen, name| {
                if self.ids.contains_key(&name) {
                    return Ok(unseen);
                }
                validate_name(&name, next_line(self.len())?)?;
                unseen.insert(name);
                let count = self
                    .len()
                    .checked_add(unseen.len())
                    .ok_or(Error::IdOverflow)?;
                u32::try_from(count).map_err(|_| Error::IdOverflow)?;
                Ok(unseen)
            })
    }

    /// Check whether `next` is a valid append-only successor.
    ///
    /// Every name in `previous` must appear at the same position in `next`.
    /// The first line that is absent or different gives an error.
    ///
    /// # Errors
    /// Returns `PrefixChanged` at the first removed, replaced, or reordered line.
    pub fn verify_prefix(previous: &Registry, next: &Registry) -> Result<(), Error> {
        previous
            .names
            .iter()
            .enumerate()
            .try_for_each(|(index, name)| {
                let found = next.names.get(index);
                if found == Some(name) {
                    return Ok(());
                }
                Err(Error::PrefixChanged {
                    line: next_line(index)?,
                    expected: name.clone(),
                    found: found.cloned(),
                })
            })
    }

    /// Render the registry to its canonical UTF-8 form.
    pub fn render(&self) -> String {
        self.names.iter().fold(String::new(), |mut text, name| {
            text.push_str(name);
            text.push('\n');
            text
        })
    }

    fn push_name(&mut self, name: String) -> Result<(), Error> {
        let line = next_line(self.len())?;
        validate_name(&name, line)?;
        let id = NonZeroU32::new(u32::try_from(line).map_err(|_| Error::IdOverflow)?)
            .ok_or(Error::IdOverflow)?;
        if self.ids.contains_key(&name) {
            return Err(Error::Duplicate { line, name });
        }
        self.ids.insert(name.clone(), id);
        self.names.push(name);
        Ok(())
    }
}

fn next_line(length: usize) -> Result<usize, Error> {
    let line = length.checked_add(1).ok_or(Error::IdOverflow)?;
    u32::try_from(line).map_err(|_| Error::IdOverflow)?;
    Ok(line)
}

fn validate_name(name: &str, line: usize) -> Result<(), Error> {
    if name.contains('\r') {
        return Err(Error::CarriageReturn { line });
    }
    let mut bytes = name.bytes();
    let valid_first = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if !valid_first || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return Err(Error::InvalidName {
            line,
            name: name.to_owned(),
        });
    }
    Ok(())
}
