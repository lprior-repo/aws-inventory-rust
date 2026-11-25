use super::{FieldId, RecordError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemberExclusions([Option<FieldId>; 8]);

impl MemberExclusions {
    pub fn new(fields: impl IntoIterator<Item = FieldId>) -> Result<Self, RecordError> {
        fields
            .into_iter()
            .try_fold(Self::default(), |mut result, field| {
                if result.contains(field) {
                    return Ok(result);
                }
                let slot = result
                    .0
                    .iter_mut()
                    .find(|slot| slot.is_none())
                    .ok_or(RecordError::SizeLimit)?;
                *slot = Some(field);
                Ok(result)
            })
    }

    pub fn contains(self, field: FieldId) -> bool {
        self.0.contains(&Some(field))
    }
}
