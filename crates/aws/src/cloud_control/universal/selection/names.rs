use super::*;

const MAX_NORMALIZED_NAME: usize = 100;
const DIGEST_SUFFIX_BYTES: usize = 8;

pub(super) fn unique_name(
    prefix: &str,
    normalized: &str,
    suffix: &str,
    seen: &mut BTreeSet<String>,
) -> Result<String, AwsScanError> {
    let candidate = format!("{prefix}{normalized}");
    if candidate.len() <= 128 && seen.insert(candidate.clone()) {
        return Ok(candidate);
    }
    let fallback = format!("{prefix}{normalized}_{suffix}");
    if fallback.len() > 128 || !seen.insert(fallback.clone()) {
        return Err(invalid("universal descriptor name collision"));
    }
    Ok(fallback)
}

pub(super) fn short_digest(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let encoded = format!("{:x}", base16ct::HexDisplay(&digest));
    encoded
        .get(..DIGEST_SUFFIX_BYTES)
        .map(str::to_owned)
        .unwrap_or_else(|| "00000000".to_owned())
}

pub(super) fn normalized_name(type_name: &str) -> Result<String, AwsScanError> {
    validate_normalized(normalize_segments(type_name))
}

fn normalize_segments(type_name: &str) -> String {
    let mut name = String::new();
    let mut previous_lower = false;
    for (index, segment) in type_name.split("::").enumerate() {
        if index == 0 && segment == "AWS" {
            continue;
        }
        if !name.is_empty() && !name.ends_with('_') {
            name.push('_');
        }
        append_segment(&mut name, segment, &mut previous_lower);
        if name.len() > MAX_NORMALIZED_NAME {
            truncate_normalized(&mut name);
            break;
        }
    }
    name
}

fn append_segment(name: &mut String, segment: &str, previous_lower: &mut bool) {
    let mut segment_started = false;
    for character in segment.chars() {
        if !character.is_ascii_alphanumeric() {
            push_separator(name, previous_lower);
            continue;
        }
        push_alphanumeric(name, character, segment_started, *previous_lower);
        *previous_lower = character.is_ascii_lowercase() || character.is_ascii_digit();
        segment_started = true;
    }
}

fn push_separator(name: &mut String, previous_lower: &mut bool) {
    if !name.ends_with('_') {
        name.push('_');
        *previous_lower = false;
    }
}

fn push_alphanumeric(name: &mut String, character: char, started: bool, previous_lower: bool) {
    if started && character.is_ascii_uppercase() && previous_lower && !name.ends_with('_') {
        name.push('_');
    }
    name.push(character.to_ascii_lowercase());
}

fn truncate_normalized(name: &mut String) {
    name.truncate(MAX_NORMALIZED_NAME);
    while name.ends_with('_') {
        name.pop();
    }
}

fn validate_normalized(name: String) -> Result<String, AwsScanError> {
    let safe =
        !name.is_empty() && name.len() <= MAX_NORMALIZED_NAME && name.bytes().all(is_name_byte);
    if !safe {
        return Err(invalid("universal type name cannot be normalized safely"));
    }
    Ok(name)
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
}
