use super::{Key, KeyRef, MAX_DEPTH, Record, RecordError, ValueKind, ValueRef};
use std::collections::TryReserveError;

mod bounds;
mod encoding;
use bounds::{entry_buffer_size, key_size, key_size_ref, owned_key, validate_limits};

#[cfg(test)]
mod tests;

const MAX_NODES: usize = 65_536;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error("record builder allocation failed")]
    Allocation(#[from] TryReserveError),
    #[error("record builder limit exceeded: {0}")]
    Limit(&'static str),
    #[error("record builder state error: {0}")]
    State(&'static str),
}

struct Entry {
    key: Option<Key>,
    bytes: Vec<u8>,
}

struct Frame {
    map: bool,
    parent_key: Option<Key>,
    entries: Vec<Entry>,
}

pub struct RecordBuilder {
    max_bytes: usize,
    max_nodes: usize,
    max_depth: usize,
    buffer_limit: usize,
    buffered: usize,
    nodes: usize,
    frames: Vec<Frame>,
    root: Option<Vec<u8>>,
    failed: bool,
}

impl RecordBuilder {
    pub fn new(max_bytes: usize, max_nodes: usize, max_depth: usize) -> Result<Self, BuildError> {
        validate_limits(max_bytes, max_nodes, max_depth)?;
        let buffer_limit = max_bytes
            .checked_mul(2)
            .ok_or(BuildError::Limit("buffer"))?;
        Ok(Self {
            max_bytes,
            max_nodes,
            max_depth,
            buffer_limit,
            buffered: 0,
            nodes: 0,
            frames: Vec::new(),
            root: None,
            failed: false,
        })
    }

    pub fn begin_map(&mut self, key: Option<KeyRef<'_>>) -> Result<(), BuildError> {
        let result = self.begin_container(key, true);
        self.terminal(result)
    }

    pub fn begin_list(&mut self, key: Option<KeyRef<'_>>) -> Result<(), BuildError> {
        let result = self.begin_container(key, false);
        self.terminal(result)
    }

    pub fn scalar(
        &mut self,
        key: Option<KeyRef<'_>>,
        value: ValueRef<'_>,
    ) -> Result<(), BuildError> {
        let result = self.add_scalar(key, value);
        self.terminal(result)
    }

    pub fn end(&mut self) -> Result<(), BuildError> {
        let result = self.close_container();
        self.terminal(result)
    }

    pub fn finish(self) -> Result<Record, BuildError> {
        if self.failed {
            return Err(BuildError::State("builder is terminally failed"));
        }
        if !self.frames.is_empty() {
            return Err(BuildError::State("containers remain open"));
        }
        let bytes = self.root.ok_or(BuildError::State("root map is missing"))?;
        Ok(Record::from_bytes(bytes)?)
    }

    fn terminal<T>(&mut self, result: Result<T, BuildError>) -> Result<T, BuildError> {
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn begin_container(&mut self, key: Option<KeyRef<'_>>, map: bool) -> Result<(), BuildError> {
        self.ensure_active()?;
        self.validate_position(key, map)?;
        self.check_depth()?;
        let key_bytes = key_size_ref(key)?;
        let next_buffered = self.check_buffer(key_bytes)?;
        let parent_key = owned_key(key)?;
        self.count_node()?;
        self.frames.try_reserve(1)?;
        self.buffered = next_buffered;
        self.frames.push(Frame {
            map,
            parent_key,
            entries: Vec::new(),
        });
        Ok(())
    }

    fn add_scalar(
        &mut self,
        key: Option<KeyRef<'_>>,
        value: ValueRef<'_>,
    ) -> Result<(), BuildError> {
        self.ensure_active()?;
        if matches!(value.kind(), ValueKind::Map | ValueKind::List) {
            return Err(BuildError::State("container passed as scalar"));
        }
        self.validate_position(key, false)?;
        self.check_value_depth(value)?;
        let key_bytes = key_size_ref(key)?;
        self.check_buffer(key_bytes)?;
        self.count_node()?;
        let owned = owned_key(key)?;
        let existing = self
            .buffered
            .checked_add(key_bytes)
            .ok_or(BuildError::Limit("buffer"))?;
        let bytes = encoding::scalar(value, self.max_bytes, self.buffer_limit, existing)?;
        self.attach_entry(Entry { key: owned, bytes })
    }
    fn close_container(&mut self) -> Result<(), BuildError> {
        self.ensure_active()?;
        let frame = self
            .frames
            .pop()
            .ok_or(BuildError::State("no container to close"))?;
        let Frame {
            map,
            parent_key,
            entries,
        } = frame;
        let key_bytes = key_size(parent_key.as_ref())?;
        let encoded = self.encode_frame(Frame {
            map,
            parent_key: None,
            entries,
        })?;
        self.buffered = self
            .buffered
            .checked_sub(key_bytes)
            .ok_or(BuildError::State("buffer accounting"))?;
        self.attach_entry(Entry {
            key: parent_key,
            bytes: encoded,
        })
    }

    fn ensure_active(&self) -> Result<(), BuildError> {
        if self.failed {
            return Err(BuildError::State("builder is terminally failed"));
        }
        Ok(())
    }

    fn validate_position(&self, key: Option<KeyRef<'_>>, root_map: bool) -> Result<(), BuildError> {
        match self.frames.last() {
            Some(frame) if frame.map && key.is_none() => {
                Err(BuildError::State("map child requires a key"))
            }
            Some(frame) if !frame.map && key.is_some() => {
                Err(BuildError::State("list child cannot have a key"))
            }
            Some(_) => Ok(()),
            None if self.root.is_some() => Err(BuildError::State("root already completed")),
            None if key.is_some() => Err(BuildError::State("root cannot have a key")),
            None if !root_map => Err(BuildError::State("root must be a map")),
            None => Ok(()),
        }
    }

    fn check_depth(&self) -> Result<(), BuildError> {
        self.frames
            .len()
            .checked_add(1)
            .filter(|depth| *depth <= self.max_depth && *depth <= MAX_DEPTH)
            .ok_or(BuildError::Limit("depth"))?;
        Ok(())
    }

    fn check_value_depth(&self, value: ValueRef<'_>) -> Result<(), BuildError> {
        if matches!(value, ValueRef::Time(_)) {
            self.check_depth()?;
        }
        Ok(())
    }

    fn check_buffer(&self, added: usize) -> Result<usize, BuildError> {
        self.buffered
            .checked_add(added)
            .filter(|size| *size <= self.buffer_limit)
            .ok_or(BuildError::Limit("buffer"))
    }

    fn count_node(&mut self) -> Result<(), BuildError> {
        let next = self
            .nodes
            .checked_add(1)
            .ok_or(BuildError::Limit("nodes"))?;
        if next > MAX_NODES || next > self.max_nodes {
            return Err(BuildError::Limit("nodes"));
        }
        self.nodes = next;
        Ok(())
    }

    fn attach_entry(&mut self, entry: Entry) -> Result<(), BuildError> {
        if let Some(frame) = self.frames.last() {
            Self::validate_entry(frame, &entry)?;
        }
        let added = entry
            .bytes
            .len()
            .checked_add(key_size(entry.key.as_ref())?)
            .ok_or(BuildError::Limit("buffer"))?;
        let next = self.check_buffer(added)?;
        if let Some(frame) = self.frames.last_mut() {
            frame.entries.try_reserve(1)?;
            frame.entries.push(entry);
            self.buffered = next;
            return Ok(());
        }
        self.store_root(entry)
    }
    fn validate_entry(frame: &Frame, entry: &Entry) -> Result<(), BuildError> {
        if frame.map != entry.key.is_some() {
            return Err(BuildError::State("child key does not match container"));
        }
        if entry.key.as_ref().is_some_and(|key| {
            frame
                .entries
                .iter()
                .any(|old| old.key.as_ref() == Some(key))
        }) {
            return Err(BuildError::State("duplicate map key"));
        }
        Ok(())
    }

    fn store_root(&mut self, entry: Entry) -> Result<(), BuildError> {
        if entry.key.is_some() || self.root.is_some() {
            return Err(BuildError::State("invalid root value"));
        }
        if entry.bytes.len() > self.max_bytes {
            return Err(BuildError::Limit("bytes"));
        }
        self.root = Some(entry.bytes);
        Ok(())
    }

    fn encode_frame(&mut self, mut frame: Frame) -> Result<Vec<u8>, BuildError> {
        if frame.map {
            frame
                .entries
                .sort_unstable_by(|left, right| left.key.cmp(&right.key));
        }
        let bytes = encoding::container(&frame, self.max_bytes, self.buffer_limit, self.buffered)?;
        let removed = entry_buffer_size(&frame.entries)?;
        self.buffered = self
            .buffered
            .checked_sub(removed)
            .ok_or(BuildError::State("buffer accounting"))?;
        Ok(bytes)
    }
}
