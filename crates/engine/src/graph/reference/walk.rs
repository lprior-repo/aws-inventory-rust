use super::{PROPERTIES_PREFIX, ReferenceConfig};
use serde_json::{Map, Value};
use std::io::{self, Write};

const PRIMARY_IDENTIFIER: &str = "resource_identifier";

pub(super) struct WalkOutcome {
    pub(super) truncated: bool,
}

struct WalkState {
    nodes: usize,
    max_nodes: usize,
    max_depth: usize,
    max_reference_bytes: usize,
    truncated: bool,
}

pub(super) fn walk_properties(
    properties: &Value,
    config: &ReferenceConfig,
    visit: &mut impl FnMut(&str, &str),
) -> WalkOutcome {
    let mut state = WalkState {
        nodes: 0,
        max_nodes: config.max_nodes_per_resource.max(1),
        max_depth: config.max_depth,
        max_reference_bytes: config.max_reference_bytes,
        truncated: false,
    };
    let mut pointer = String::from(PROPERTIES_PREFIX);
    walk_value(properties, &mut pointer, 0, &mut state, visit);
    WalkOutcome {
        truncated: state.truncated,
    }
}

fn walk_value(
    value: &Value,
    pointer: &mut String,
    depth: usize,
    state: &mut WalkState,
    visit: &mut impl FnMut(&str, &str),
) {
    if state.nodes >= state.max_nodes {
        state.truncated = true;
        return;
    }
    state.nodes = state.nodes.saturating_add(1);
    match value {
        Value::Object(entries) => walk_object(entries, pointer, depth, state, visit),
        Value::Array(items) => walk_array(items, pointer, depth, state, visit),
        Value::String(text) => walk_leaf(text, pointer, state, visit),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn walk_object(
    entries: &Map<String, Value>,
    pointer: &mut String,
    depth: usize,
    state: &mut WalkState,
    visit: &mut impl FnMut(&str, &str),
) {
    if depth >= state.max_depth {
        state.truncated = true;
        return;
    }
    for (key, child) in entries {
        if depth == 0 && key == PRIMARY_IDENTIFIER {
            continue;
        }
        let restore = pointer.len();
        push_segment(pointer, key);
        walk_value(child, pointer, depth.saturating_add(1), state, visit);
        pointer.truncate(restore);
    }
}

fn walk_array(
    items: &[Value],
    pointer: &mut String,
    depth: usize,
    state: &mut WalkState,
    visit: &mut impl FnMut(&str, &str),
) {
    if depth >= state.max_depth {
        state.truncated = true;
        return;
    }
    for (index, child) in items.iter().enumerate() {
        let restore = pointer.len();
        pointer.push('/');
        pointer.push_str(&index.to_string());
        walk_value(child, pointer, depth.saturating_add(1), state, visit);
        pointer.truncate(restore);
    }
}

fn walk_leaf(text: &str, pointer: &str, state: &WalkState, visit: &mut impl FnMut(&str, &str)) {
    if text.len() <= state.max_reference_bytes {
        visit(pointer, text);
    }
}

fn push_segment(pointer: &mut String, segment: &str) {
    pointer.push('/');
    for character in segment.chars() {
        match character {
            '~' => pointer.push_str("~0"),
            '/' => pointer.push_str("~1"),
            other => pointer.push(other),
        }
    }
}

pub(super) fn serialized_size_exceeds(value: &Value, limit: usize) -> bool {
    serde_json::to_writer(LimitedSink { used: 0, limit }, value).is_err()
}

struct LimitedSink {
    used: usize,
    limit: usize,
}

impl Write for LimitedSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self.used.saturating_add(bytes.len());
        if next > self.limit {
            return Err(io::Error::other("payload exceeds bound"));
        }
        self.used = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
