use crate::{GraphError, MAX_QUERY_BYTES, NodeKey};
use std::fmt::Write;

pub(crate) struct Query(String);

impl Query {
    pub(crate) fn new() -> Self {
        Self(String::new())
    }

    pub(crate) fn push(&mut self, value: &str) -> Result<(), GraphError> {
        let length = self
            .0
            .len()
            .checked_add(value.len())
            .ok_or(GraphError::QueryTooLarge)?;
        if length > MAX_QUERY_BYTES {
            return Err(GraphError::QueryTooLarge);
        }
        self.0
            .try_reserve(value.len())
            .map_err(|_| GraphError::Allocation)?;
        self.0.push_str(value);
        Ok(())
    }

    pub(crate) fn parameter(&mut self, name: &str, value: Option<&str>) -> Result<(), GraphError> {
        self.push(&format!(" {name}="))?;
        match value {
            Some(value) => self.push(&quote(value)?),
            None => self.push("null"),
        }
    }

    pub(crate) fn finish(self) -> String {
        self.0
    }
}

pub(crate) fn quote(value: &str) -> Result<String, GraphError> {
    if value.len() > 4096 {
        return Err(GraphError::InvalidRow);
    }
    let mut query = Query::new();
    query.push("'")?;
    value.chars().try_for_each(|character| match character {
        '\\' => query.push("\\\\"),
        '\'' => query.push("\\'"),
        '\n' => query.push("\\n"),
        '\r' => query.push("\\r"),
        '\t' => query.push("\\t"),
        value if value.is_control() => query.push(&format!("\\u{:04x}", u32::from(value))),
        value => query.push(value.encode_utf8(&mut [0; 4])),
    })?;
    query.push("'")?;
    Ok(query.finish())
}

pub(crate) fn digest(key: &NodeKey) -> Result<String, GraphError> {
    hex(key.digest().as_bytes())
}

pub(crate) fn hex(bytes: &[u8; 32]) -> Result<String, GraphError> {
    let mut value = String::new();
    value
        .try_reserve_exact(64)
        .map_err(|_| GraphError::Allocation)?;
    bytes
        .iter()
        .try_for_each(|byte| write!(value, "{byte:02x}").map_err(|_| GraphError::QueryTooLarge))?;
    Ok(value)
}

pub(crate) fn identifier(value: &str) -> Result<String, GraphError> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(GraphError::InvalidRow);
    }
    Ok(format!("`{}`", value.replace('`', "``")))
}
