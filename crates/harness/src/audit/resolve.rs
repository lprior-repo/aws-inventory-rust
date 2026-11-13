//! Resolve enclosing functions, SDK client namespaces and provider builders.

use std::collections::BTreeSet;

use crate::HarnessError;

mod imports;

use imports::module_imports;

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub(crate) fn is_boundary(chars: &[char], index: usize) -> bool {
    index == 0
        || !chars
            .get(index.saturating_sub(1))
            .map(|value| is_word(*value))
            .unwrap_or(false)
}

pub(crate) fn matches_at(chars: &[char], start: usize, text: &[char]) -> bool {
    text.iter().enumerate().all(|(index, expected)| {
        chars
            .get(start.saturating_add(index))
            .map(|value| value == expected)
            .unwrap_or(false)
    })
}

fn skip_space(chars: &[char], start: usize) -> usize {
    let mut position = start;
    while chars.get(position).map(|value| value.is_whitespace()) == Some(true) {
        position = position.saturating_add(1);
    }
    position
}

pub(crate) fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            characters.all(|value| value.is_ascii_alphanumeric() || value == '_')
        }
        _ => false,
    }
}

fn fn_keyword_at(masked: &[char], index: usize) -> bool {
    is_boundary(masked, index)
        && masked.get(index) == Some(&'f')
        && masked.get(index.saturating_add(1)) == Some(&'n')
        && masked
            .get(index.saturating_add(2))
            .map(|value| value.is_whitespace())
            == Some(true)
}

/// Identifier after the `fn` keyword at `index`, when it starts before `offset`.
fn function_name(masked: &[char], index: usize, offset: usize) -> Option<(String, usize)> {
    let mut cursor = skip_space(masked, index.saturating_add(3));
    if masked
        .get(cursor)
        .map(|value| value.is_ascii_alphabetic() || *value == '_')
        != Some(true)
    {
        return None;
    }
    let start = cursor;
    cursor = cursor.saturating_add(1);
    while masked
        .get(cursor)
        .map(|value| value.is_ascii_alphanumeric() || *value == '_')
        == Some(true)
    {
        cursor = cursor.saturating_add(1);
    }
    if cursor > offset {
        return None;
    }
    let name: String = masked.get(start..cursor)?.iter().collect();
    Some((name, index))
}

/// Last `fn` name declared before `offset` in the masked source.
pub(crate) fn function_at(masked: &[char], offset: usize) -> Result<(String, usize), HarnessError> {
    let mut found: Option<(String, usize)> = None;
    for index in 0..offset {
        if fn_keyword_at(masked, index)
            && let Some(name) = function_name(masked, index, offset)
        {
            found = Some(name);
        }
    }
    found.ok_or_else(|| HarnessError::invalid("SDK send is not inside a recognized function"))
}

fn sdk_service_at(chars: &[char], start: usize) -> Option<(String, usize)> {
    if !matches_at(chars, start, &['a', 'w', 's', '_', 's', 'd', 'k', '_']) {
        return None;
    }
    let service_start = start.saturating_add(8);
    let mut end = service_start;
    while chars
        .get(end)
        .map(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || *value == '_')
        == Some(true)
    {
        end = end.saturating_add(1);
    }
    if end == service_start {
        return None;
    }
    let service: String = chars.get(service_start..end)?.iter().collect();
    Some((service, end))
}

fn binding_branch(scope: &[char], start: usize) -> Option<(String, usize)> {
    let mut cursor = start;
    if scope.get(cursor) != Some(&':') {
        return None;
    }
    cursor = skip_space(scope, cursor.saturating_add(1));
    if scope.get(cursor) == Some(&'&') {
        cursor = skip_space(scope, cursor.saturating_add(1));
    }
    let (service, after) = sdk_service_at(scope, cursor)?;
    if !matches_at(scope, after, &[':', ':', 'C', 'l', 'i', 'e', 'n', 't']) {
        return None;
    }
    let end = after.saturating_add(8);
    if scope.get(end).map(|value| is_word(*value)) == Some(true) {
        return None;
    }
    Some((service, end))
}

fn assignment_branch(scope: &[char], start: usize) -> Option<(String, usize)> {
    let mut cursor = start;
    if scope.get(cursor) != Some(&'=') {
        return None;
    }
    cursor = skip_space(scope, cursor.saturating_add(1));
    let (service, after) = sdk_service_at(scope, cursor)?;
    if !matches_at(scope, after, &[':', ':', 'C', 'l', 'i', 'e', 'n', 't']) {
        return None;
    }
    let end = skip_space(scope, after.saturating_add(8));
    if !matches_at(scope, end, &[':', ':']) {
        return None;
    }
    Some((service, end.saturating_add(2)))
}

/// Service bound to `receiver` by an annotation or assignment in `scope`.
fn scope_binding(scope: &[char], receiver: &str) -> Option<String> {
    let receiver_chars: Vec<char> = receiver.chars().collect();
    let mut resolved: Option<String> = None;
    let mut index = 0_usize;
    while index < scope.len() {
        if is_boundary(scope, index) && matches_at(scope, index, &receiver_chars) {
            let start = skip_space(scope, index.saturating_add(receiver_chars.len()));
            if let Some((service, end)) =
                binding_branch(scope, start).or_else(|| assignment_branch(scope, start))
            {
                resolved = Some(service);
                index = end;
                continue;
            }
        }
        index = index.saturating_add(1);
    }
    resolved
}

fn unresolved(receiver: &str) -> HarnessError {
    HarnessError::invalid(format!(
        "cannot uniquely resolve SDK client receiver {receiver}"
    ))
}

/// SDK service namespaces mentioned anywhere in the module, including imports
/// that do not bring `Client` into lexical scope (a parent `use super::*` may
/// supply the client type). Used only when client-import resolution is
/// ambiguous or empty.
pub(crate) fn module_services(masked: &[char]) -> BTreeSet<String> {
    let mut services: BTreeSet<String> = BTreeSet::new();
    let mut cursor = 0_usize;
    while cursor < masked.len() {
        if let Some((service, after)) = sdk_service_at(masked, cursor)
            && matches_at(masked, after, &[':', ':'])
        {
            services.insert(service);
            cursor = after;
            continue;
        }
        cursor = cursor.saturating_add(1);
    }
    services
}

fn unique_service(services: BTreeSet<String>, receiver: &str) -> Result<String, HarnessError> {
    services
        .into_iter()
        .next()
        .ok_or_else(|| unresolved(receiver))
}

fn imported_service(masked: &[char], receiver: &str) -> Result<Option<String>, HarnessError> {
    let imports = module_imports(masked)?;
    if imports.len() != 1 {
        return Ok(None);
    }
    unique_service(imports, receiver).map(Some)
}

fn module_service(masked: &[char], receiver: &str) -> Result<Option<String>, HarnessError> {
    let services = module_services(masked);
    if services.len() != 1 {
        return Ok(None);
    }
    unique_service(services, receiver).map(Some)
}

/// Resolve the SDK service a client receiver belongs to.
pub(crate) fn receiver_service(
    masked: &[char],
    receiver: &str,
    function_start: usize,
    offset: usize,
    hint: Option<&str>,
) -> Result<String, HarnessError> {
    let scope = masked
        .get(function_start..offset)
        .ok_or_else(|| HarnessError::invalid("SDK receiver scope bounds"))?;
    if let Some(service) = scope_binding(scope, receiver) {
        return Ok(service);
    }
    if let Some(service) = imported_service(masked, receiver)? {
        return Ok(service);
    }
    if let Some(service) = module_service(masked, receiver)? {
        return Ok(service);
    }
    if let Some(service) = hint {
        return Ok(service.to_owned());
    }
    Err(unresolved(receiver))
}

fn builder_at(masked: &[char], index: usize, pattern: &[char], builder: &[char]) -> bool {
    if !is_boundary(masked, index) || !matches_at(masked, index, pattern) {
        return false;
    }
    let mut cursor = skip_space(masked, index.saturating_add(pattern.len()));
    if !matches_at(masked, cursor, &[':', ':']) {
        return false;
    }
    cursor = skip_space(masked, cursor.saturating_add(2));
    if !matches_at(masked, cursor, builder) {
        return false;
    }
    cursor = skip_space(masked, cursor.saturating_add(builder.len()));
    masked.get(cursor) == Some(&'(')
}

/// Offsets of `AssumeRoleProvider::builder(...)` constructions.
pub(crate) fn provider_builder_offsets(masked: &[char]) -> Vec<usize> {
    let pattern: Vec<char> = "AssumeRoleProvider".chars().collect();
    let builder: Vec<char> = "builder".chars().collect();
    let mut offsets = Vec::new();
    for index in 0..masked.len() {
        if builder_at(masked, index, &pattern, &builder) {
            offsets.push(index);
        }
    }
    offsets
}
