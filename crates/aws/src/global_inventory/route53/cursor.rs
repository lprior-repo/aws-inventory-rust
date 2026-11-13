//! cursor responsibilities for route53.
use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct RecordCursor {
    pub(super) name: String,
    pub(super) record_type: String,
    pub(super) identifier: Option<String>,
}

impl RecordCursor {
    pub(super) fn validate(&self) -> Result<(), AwsScanError> {
        checked_text(&self.name, "record cursor name")?;
        checked_text(&self.record_type, "record cursor type")?;
        if self.name.len() > 1_024
            || self.record_type.len() > 32
            || self.identifier.as_ref().is_some_and(|id| {
                id.is_empty()
                    || id.len() > 512
                    || id.chars().take(129).count() > 128
                    || id.contains('\0')
            })
        {
            return Err(AwsScanError::InvalidData(
                "record cursor exceeds field limits".into(),
            ));
        }
        Ok(())
    }
    pub(super) fn decode(token: Option<&str>) -> Result<Option<Self>, AwsScanError> {
        let Some(token) = token else {
            return Ok(None);
        };
        if token.len() > 2_048 {
            return Err(AwsScanError::InvalidData(
                "record cursor exceeds 2 KiB".into(),
            ));
        }
        let cursor: Self = serde_json::from_str(token)
            .map_err(|_| AwsScanError::InvalidData("invalid record cursor JSON".into()))?;
        cursor.validate()?;
        Ok(Some(cursor))
    }
}

pub(super) fn next_record_cursor(
    output: &ListResourceRecordSetsOutput,
) -> Result<Option<String>, AwsScanError> {
    if !output.is_truncated() {
        return Ok(None);
    }
    let cursor = RecordCursor {
        name: required(output.next_record_name(), "next record name")?,
        record_type: output
            .next_record_type()
            .map(|v| v.as_str().to_owned())
            .ok_or_else(|| AwsScanError::InvalidData("truncated records lack next type".into()))?,
        identifier: output.next_record_identifier().map(str::to_owned),
    };
    cursor.validate()?;
    serde_json::to_string(&cursor)
        .map(Some)
        .map_err(|_| AwsScanError::InvalidData("record cursor serialization failed".into()))
}
