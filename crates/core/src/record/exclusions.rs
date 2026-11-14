use super::{FieldId, MAX_DEPTH, RecordError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exclusions(Vec<Box<[FieldId]>>);

impl Exclusions {
    pub fn none() -> Self {
        Self(Vec::new())
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
        Ok(Self(paths))
    }

    pub(super) fn paths(&self) -> &[Box<[FieldId]>] {
        &self.0
    }
}
