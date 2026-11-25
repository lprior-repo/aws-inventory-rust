use super::{FieldId, MAX_DEPTH, RecordError};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapKeyExclusion {
    path: Box<[FieldId]>,
    key: Box<str>,
}

impl MapKeyExclusion {
    pub fn new(path: Vec<FieldId>, key: impl Into<Box<str>>) -> Result<Self, RecordError> {
        if path.is_empty() || path.len() > MAX_DEPTH {
            return Err(RecordError::ExclusionPath { length: path.len() });
        }
        let key = key.into();
        if key.is_empty() || key.len() > 4096 {
            return Err(RecordError::ExclusionKey { length: key.len() });
        }
        Ok(Self {
            path: path.into_boxed_slice(),
            key,
        })
    }

    pub fn path(&self) -> &[FieldId] {
        &self.path
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exclusions {
    paths: Option<Arc<[Box<[FieldId]>]>>,
    map_keys: Option<Arc<[MapKeyExclusion]>>,
}

impl Exclusions {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn new(paths: impl IntoIterator<Item = Vec<FieldId>>) -> Result<Self, RecordError> {
        let mut paths = paths
            .into_iter()
            .map(|path| match path.len() {
                1..=MAX_DEPTH => Ok(path.into_boxed_slice()),
                length => Err(RecordError::ExclusionPath { length }),
            })
            .collect::<Result<Vec<_>, RecordError>>()?;
        paths.sort_unstable();
        paths.dedup();
        Ok(Self {
            paths: (!paths.is_empty()).then(|| Arc::from(paths)),
            map_keys: None,
        })
    }

    pub fn paths(&self) -> &[Box<[FieldId]>] {
        match &self.paths {
            Some(paths) => paths,
            None => &[],
        }
    }

    pub fn with_map_keys(
        mut self,
        mut map_keys: Vec<MapKeyExclusion>,
    ) -> Result<Self, RecordError> {
        if map_keys.len() > 64 {
            return Err(RecordError::ExclusionCount {
                length: map_keys.len(),
            });
        }
        map_keys.sort_unstable();
        if map_keys.windows(2).any(|pair| pair.first() == pair.last()) {
            return Err(RecordError::DuplicateExclusion);
        }
        self.map_keys = (!map_keys.is_empty()).then(|| Arc::from(map_keys));
        Ok(self)
    }

    pub fn map_keys(&self) -> &[MapKeyExclusion] {
        match &self.map_keys {
            Some(map_keys) => map_keys,
            None => &[],
        }
    }
}
