use super::Failure;
use crate::overlay::{OverlayError, directory};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn services(
    root: &Path,
    overrides: &BTreeSet<String>,
    values: &BTreeMap<String, Value>,
) -> Result<BTreeSet<String>, Failure> {
    let selected = if overrides.is_empty() {
        let text = directory::read_text(&root.join("catalog/overlay/generated.services"))?;
        parse(&text, values)?
    } else {
        overrides.clone()
    };
    if selected.len() > directory::MAX_FILES {
        return Err("generated service limit".into());
    }
    for name in &selected {
        if !values.contains_key(name) {
            return Err(format!("unknown service {name}").into());
        }
    }
    Ok(selected)
}

fn parse(text: &str, values: &BTreeMap<String, Value>) -> Result<BTreeSet<String>, Failure> {
    if text == "*\n" {
        return Ok(values.keys().cloned().collect());
    }
    if !text.is_empty() && !text.ends_with('\n') {
        return Err("generated.services requires a final newline".into());
    }
    let mut selected = BTreeSet::new();
    let mut previous = "";
    for name in text.lines() {
        if name <= previous || !valid_name(name) {
            return Err("generated.services must contain sorted unique service names".into());
        }
        if selected.len() >= directory::MAX_FILES {
            return Err("generated service limit".into());
        }
        selected.insert(name.to_owned());
        previous = name;
    }
    Ok(selected)
}

fn valid_name(name: &str) -> bool {
    name.bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

pub(in crate::overlay) fn deny_text(root: &Path) -> Result<String, OverlayError> {
    let file = root.join("catalog/must-deny.txt");
    match directory::read_text(&file) {
        Ok(text) => Ok(text),
        Err(OverlayError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            Ok(String::new())
        }
        Err(error) => Err(error),
    }
}
