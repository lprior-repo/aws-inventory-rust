//! `use` scanning for SDK client imports.

use std::collections::BTreeSet;

use crate::HarnessError;

use super::{is_boundary, matches_at, sdk_service_at, skip_space};

fn use_keyword_at(masked: &[char], cursor: usize) -> bool {
    is_boundary(masked, cursor)
        && matches_at(masked, cursor, &['u', 's', 'e'])
        && masked
            .get(cursor.saturating_add(3))
            .map(|value| value.is_whitespace())
            == Some(true)
}

/// Import body after `::` and the offset after its terminating `;`.
fn import_body(masked: &[char], after: usize) -> Result<Option<(String, usize)>, HarnessError> {
    let body_start = skip_space(masked, after.saturating_add(2));
    let mut end = body_start;
    while masked.get(end).map(|value| *value != ';') == Some(true) {
        end = end.saturating_add(1);
    }
    if end == body_start || masked.get(end) != Some(&';') {
        return Ok(None);
    }
    let body: String = masked
        .get(body_start..end)
        .ok_or_else(|| HarnessError::invalid("SDK import bounds"))?
        .iter()
        .filter(|value| !value.is_whitespace())
        .collect();
    Ok(Some((body, end.saturating_add(1))))
}

/// One SDK `Client` import, with the offset after its terminating `;`.
fn sdk_import_at(
    masked: &[char],
    cursor: usize,
) -> Result<Option<(Option<String>, usize)>, HarnessError> {
    if !use_keyword_at(masked, cursor) {
        return Ok(None);
    }
    let start = skip_space(masked, cursor.saturating_add(3));
    let Some((service, after)) = sdk_service_at(masked, start) else {
        return Ok(None);
    };
    if !matches_at(masked, after, &[':', ':']) {
        return Ok(None);
    }
    let Some((body, end)) = import_body(masked, after)? else {
        return Ok(None);
    };
    let client = body == "Client" || body.starts_with("{Client,") || body.starts_with("{Client}");
    Ok(Some((client.then_some(service), end)))
}

/// Services whose `Client` type is imported into this module.
pub(super) fn module_imports(masked: &[char]) -> Result<BTreeSet<String>, HarnessError> {
    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut cursor = 0_usize;
    while cursor < masked.len() {
        match sdk_import_at(masked, cursor)? {
            Some((Some(service), end)) => {
                imports.insert(service);
                cursor = end;
            }
            Some((None, end)) => cursor = end,
            None => cursor = cursor.saturating_add(1),
        }
    }
    Ok(imports)
}
