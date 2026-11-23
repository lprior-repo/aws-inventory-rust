mod bounds;
mod descriptor;
mod expand;
#[cfg(test)]
mod seed_files;
mod step;
mod value;

use super::{
    config::SeedError,
    scenario::{
        FieldPath, SeedInput, SeedInputValue, SeedOutputCheck, SeedRelationship, SeedScenario,
        SeedScope, SeedStep, StepId,
    },
};
use descriptor::{Document, RawRelationship, RawStep, field, keys};

const MAX_FILE_BYTES: usize = super::MAX_VALUE_BYTES;

pub fn load(content: &str) -> Result<SeedScenario, SeedError> {
    Ok(load_labeled(content)?.0)
}

pub(super) fn load_labeled(content: &str) -> Result<(SeedScenario, Vec<String>), SeedError> {
    if content.len() > MAX_FILE_BYTES {
        return Err(SeedError::Invalid("seed file exceeds size limit"));
    }
    let value = toml::from_str::<toml::Value>(content)
        .map_err(|error| SeedError::Message(format!("seed file parse failed: {error}")))?;
    let document = Document::parse(&value)?;
    let scenario = expand::convert(&document)?;
    Ok((scenario, expand::labels(&document)?))
}

fn path(value: &str) -> Result<FieldPath, SeedError> {
    FieldPath::new(&value.split('.').collect::<Vec<_>>())
}
