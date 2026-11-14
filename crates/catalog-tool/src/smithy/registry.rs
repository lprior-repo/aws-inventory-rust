use inventory_core::record::FieldId;
use std::collections::{BTreeSet, HashMap};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("registry is missing its final newline")]
    MissingFinalNewline,

    #[error("carriage return on registry line {line}")]
    CarriageReturn { line: usize },

    #[error("invalid member name on registry line {line}: {name:?}")]
    InvalidName { line: usize, name: String },

    #[error("duplicate member name on registry line {line}: {name:?}")]
    Duplicate { line: usize, name: String },

    #[error("registry exceeds the nonzero u32 ID range")]
    IdOverflow,

    #[error("registry prefix changed on line {line}: expected {expected:?}, found {found:?}")]
    PrefixChanged {
        line: usize,
        expected: String,
        found: Option<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registry {
    names: Vec<String>,
    ids: HashMap<String, FieldId>,
}

impl Registry {
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

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn id(&self, name: &str) -> Option<FieldId> {
        self.ids.get(name).copied()
    }

    pub fn name(&self, id: FieldId) -> Option<&str> {
        let index = usize::try_from(id.value()).ok()?.checked_sub(1)?;
        self.names.get(index).map(String::as_str)
    }

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
        let id = FieldId::new(u32::try_from(line).map_err(|_| Error::IdOverflow)?)
            .map_err(|_| Error::IdOverflow)?;
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
