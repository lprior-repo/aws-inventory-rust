use crate::HarnessError;
use crate::provenance::canonical_sha256;

use super::Json;

struct Digests<'a> {
    catalog: &'a str,
    source: &'a str,
    binary: &'a str,
    fixture: &'a str,
}

fn digest(value: Option<&Json>) -> Option<&str> {
    value.and_then(Json::as_str).filter(|text| {
        text.len() == 64
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn digests(binding: &Json) -> Result<Digests<'_>, HarnessError> {
    Ok(Digests {
        catalog: digest(binding.get("catalog_sha256")).ok_or_else(differ)?,
        source: digest(binding.get("source_sha256")).ok_or_else(differ)?,
        binary: digest(binding.get("binary_sha256")).ok_or_else(differ)?,
        fixture: digest(binding.get("architecture_fixture_sha256")).ok_or_else(differ)?,
    })
}

fn aligned(executed: &Json, catalog: &Json, digests: &Digests<'_>) -> bool {
    let canonical = canonical_sha256(&catalog.to_value());
    executed
        .path(&["catalog_canonical_sha256"])
        .and_then(Json::as_str)
        == Some(canonical.as_str())
        && executed.get("catalog_sha256").and_then(Json::as_str) == Some(digests.catalog)
        && executed.path(&["source", "sha256"]).and_then(Json::as_str) == Some(digests.source)
        && executed.path(&["binary", "sha256"]).and_then(Json::as_str) == Some(digests.binary)
        && executed
            .get("architecture_fixture_sha256")
            .and_then(Json::as_str)
            == Some(digests.fixture)
}

pub(crate) fn validate_binding(
    executed: &Json,
    catalog: &Json,
    binding: &Json,
) -> Result<(), HarnessError> {
    let digests = digests(binding)?;
    if !aligned(executed, catalog, &digests) {
        return Err(differ());
    }
    Ok(())
}

fn differ() -> HarnessError {
    HarnessError::invalid("current catalog/source/binary/fixture differs from executed evidence")
}
