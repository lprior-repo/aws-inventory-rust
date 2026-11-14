use std::path::Path;

use crate::HarnessError;
use crate::catalog::json::Json;

use super::document;

pub const UNIVERSAL_CATALOG: &str = "catalog/cloudcontrol-universal.json";

pub fn rendered(root: &Path) -> Result<String, HarnessError> {
    let document = document(root)?;
    Ok(format!("{}\n", document.pretty()))
}

pub fn check(root: &Path) -> Result<(), HarnessError> {
    verify(root, &rendered(root)?)
}

pub fn run(root: &Path, check_only: bool) -> Result<Json, HarnessError> {
    let document = document(root)?;
    let contents = format!("{}\n", document.pretty());
    if check_only {
        verify(root, &contents)?;
    } else {
        std::fs::write(root.join(UNIVERSAL_CATALOG), contents)?;
    }
    let count = document
        .get("resource_types")
        .and_then(Json::as_array)
        .map_or(0, <[Json]>::len);
    Ok(Json::object(vec![
        ("file", Json::string(UNIVERSAL_CATALOG)),
        ("schema_count", Json::count(count)),
        ("check", Json::Bool(check_only)),
    ]))
}

fn verify(root: &Path, expected: &str) -> Result<(), HarnessError> {
    let committed = std::fs::read(root.join(UNIVERSAL_CATALOG))?;
    if committed == expected.as_bytes() {
        return Ok(());
    }
    Err(HarnessError::invalid(format!(
        "universal catalog differs; regenerate with `inventory-harness catalog universal` \
         (first difference at line {})",
        first_difference_line(expected, &committed)
    )))
}

fn first_difference_line(expected: &str, actual: &[u8]) -> usize {
    let expected_bytes = expected.as_bytes();
    let mut index = 0_usize;
    loop {
        match (expected_bytes.get(index), actual.get(index)) {
            (Some(left), Some(right)) if left == right => {
                index = index.saturating_add(1);
            }
            _ => break,
        }
    }
    let prefix = expected_bytes.get(..index).unwrap_or_default();
    1_usize.saturating_add(prefix.iter().filter(|byte| **byte == b'\n').count())
}
