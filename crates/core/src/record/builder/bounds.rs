use super::{
    BuildError, Entry, MAX_NODES,
    encoding::{header_size, unsigned_size},
};
use crate::record::{Key, KeyRef, MAX_DEPTH, MAX_RECORD_BYTES};

pub(super) fn validate_limits(
    max_bytes: usize,
    max_nodes: usize,
    max_depth: usize,
) -> Result<(), BuildError> {
    if max_bytes == 0 || max_bytes > MAX_RECORD_BYTES {
        return Err(BuildError::Limit("bytes"));
    }
    if max_nodes == 0 || max_nodes > MAX_NODES {
        return Err(BuildError::Limit("nodes"));
    }
    if max_depth == 0 || max_depth > MAX_DEPTH {
        return Err(BuildError::Limit("depth"));
    }
    Ok(())
}

pub(super) fn owned_key(key: Option<KeyRef<'_>>) -> Result<Option<Key>, BuildError> {
    match key {
        Some(KeyRef::Field(field)) => Ok(Some(Key::Field(field))),
        Some(KeyRef::Name(name)) => {
            let mut owned = String::new();
            owned.try_reserve_exact(name.len())?;
            owned.push_str(name);
            Ok(Some(Key::Name(owned)))
        }
        None => Ok(None),
    }
}

pub(super) fn key_size_ref(key: Option<KeyRef<'_>>) -> Result<usize, BuildError> {
    match key {
        Some(KeyRef::Name(name)) => name
            .len()
            .checked_add(header_size(name.len()))
            .ok_or(BuildError::Limit("buffer")),
        Some(KeyRef::Field(field)) => Ok(unsigned_size(field.value())),
        None => Ok(0),
    }
}

pub(super) fn key_size(key: Option<&Key>) -> Result<usize, BuildError> {
    match key {
        Some(Key::Name(name)) => name
            .len()
            .checked_add(header_size(name.len()))
            .ok_or(BuildError::Limit("buffer")),
        Some(Key::Field(field)) => Ok(unsigned_size(field.value())),
        None => Ok(0),
    }
}

pub(super) fn entry_buffer_size(entries: &[Entry]) -> Result<usize, BuildError> {
    entries.iter().try_fold(0_usize, |sum, entry| {
        let key = key_size(entry.key.as_ref())?;
        sum.checked_add(entry.bytes.len())
            .and_then(|length| length.checked_add(key))
            .ok_or(BuildError::Limit("buffer"))
    })
}
