//! Field-ID path exclusions for content hashing.
use super::{FieldId, MAX_DEPTH, RecordError};

/// A set of field-ID paths to exclude from content hashing.
/// Paths are sorted and de-duplicated.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exclusions(Vec<Box<[FieldId]>>);

impl Exclusions {
    /// An empty exclusion set.
    pub fn none() -> Self {
        Self(Vec::new())
    }

    /// Build exclusions from field-ID paths.
    /// Each path must be 1..=32 IDs.
    /// # Errors
    /// Returns `ExclusionPath` for a path outside the valid length range.
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
        Ok(Self(paths))
    }

    /// Return the sorted, de-duplicated exclusion paths.
    pub(super) fn paths(&self) -> &[Box<[FieldId]>] {
        &self.0
    }
}
