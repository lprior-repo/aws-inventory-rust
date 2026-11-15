use super::*;

pub(super) fn parse_edge(value: &redis::Value) -> Result<GraphEdge, ViewError> {
    let redis::Value::Array(fields) = value else {
        return Err(ViewError::GraphResponse);
    };
    Ok(GraphEdge {
        from: parse_node(fields, 0)?,
        to: parse_node(fields, 4)?,
        relation: value_string(fields.get(8))?,
        provenance: value_string(fields.get(9))?,
    })
}

pub(super) fn parse_node(fields: &[redis::Value], start: usize) -> Result<GraphNode, ViewError> {
    let region = start.checked_add(1).ok_or(ViewError::GraphResponse)?;
    let kind = start.checked_add(2).ok_or(ViewError::GraphResponse)?;
    let id = start.checked_add(3).ok_or(ViewError::GraphResponse)?;
    Ok(GraphNode {
        account: value_string(fields.get(start))?,
        region: value_string(fields.get(region))?,
        kind: value_string(fields.get(kind))?,
        id: value_string(fields.get(id))?,
    })
}

pub(super) fn value_string(value: Option<&redis::Value>) -> Result<String, ViewError> {
    match value {
        Some(redis::Value::BulkString(bytes)) => {
            String::from_utf8(bytes.clone()).map_err(|_| ViewError::GraphResponse)
        }
        Some(redis::Value::SimpleString(text)) => Ok(text.clone()),
        _ => Err(ViewError::GraphResponse),
    }
}
