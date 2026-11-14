use std::io::Read;
use std::path::Path;

use crate::HarnessError;

#[derive(Debug)]
pub(crate) enum ReadError {
    Io(std::io::Error),
    TooLarge,
    InvalidBudget,
}

pub(crate) fn read_limited(reader: impl Read, limit: usize) -> Result<Vec<u8>, ReadError> {
    let budget = u64::try_from(limit)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or(ReadError::InvalidBudget)?;
    let mut value = Vec::new();
    reader
        .take(budget)
        .read_to_end(&mut value)
        .map_err(ReadError::Io)?;
    if value.len() > limit {
        return Err(ReadError::TooLarge);
    }
    Ok(value)
}

pub(crate) fn read_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, HarnessError> {
    let file = std::fs::File::open(path)
        .map_err(|error| HarnessError::invalid(format!("{}: {error}", path.display())))?;
    read_limited(file, limit).map_err(|error| match error {
        ReadError::Io(error) => HarnessError::Io(error),
        ReadError::TooLarge => {
            HarnessError::invalid(format!("input byte budget exceeded: {}", path.display()))
        }
        ReadError::InvalidBudget => HarnessError::invalid("input byte budget is not representable"),
    })
}
