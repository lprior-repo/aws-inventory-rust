mod bounds;
mod descriptor;
mod expand;
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

const MAX_FILE_BYTES: usize = 1_048_576;

pub fn load(content: &str) -> Result<SeedScenario, SeedError> {
    if content.len() > MAX_FILE_BYTES {
        return Err(SeedError::Invalid("seed file exceeds size limit"));
    }
    let value = toml::from_str::<toml::Value>(content)
        .map_err(|error| SeedError::Message(format!("seed file parse failed: {error}")))?;
    let document = Document::parse(&value)?;
    expand::convert(&document)
}

fn path(value: &str) -> Result<FieldPath, SeedError> {
    FieldPath::new(&value.split('.').collect::<Vec<_>>())
}
