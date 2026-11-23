use crate::{EdgeRow, GraphError, NodeKey, Provenance};
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};
use redis::Value;

pub(crate) fn rows(value: &Value, limit: usize) -> Result<&[Value], GraphError> {
    let Value::Array(parts) = value else {
        return Err(GraphError::Response);
    };
    let Some(Value::Array(rows)) = parts.get(1) else {
        return Err(GraphError::Response);
    };
    if rows.len() > limit {
        return Err(GraphError::Response);
    }
    Ok(rows)
}

pub(crate) fn nodes(value: &Value, limit: usize) -> Result<Vec<NodeKey>, GraphError> {
    rows(value, limit)?
        .iter()
        .map(|row| {
            let fields = fields(row, 4)?;
            node(fields)
        })
        .collect()
}

pub(crate) fn edges(value: &Value, limit: usize) -> Result<Vec<EdgeRow>, GraphError> {
    rows(value, limit)?
        .iter()
        .map(|row| {
            let fields = fields(row, 10)?;
            Ok(EdgeRow {
                from: node(fields.get(..4).ok_or(GraphError::Response)?)?,
                to: node(fields.get(4..8).ok_or(GraphError::Response)?)?,
                relation: Relation::parse(text(fields.get(8))?)
                    .map_err(|_| GraphError::Response)?,
                provenance: Provenance::parse(text(fields.get(9))?)
                    .map_err(|_| GraphError::Response)?,
            })
        })
        .collect()
}

fn fields(value: &Value, count: usize) -> Result<&[Value], GraphError> {
    match value {
        Value::Array(fields) if fields.len() == count => Ok(fields),
        _ => Err(GraphError::Response),
    }
}

fn node(fields: &[Value]) -> Result<NodeKey, GraphError> {
    Ok(NodeKey {
        account: AccountId::parse(text(fields.first())?).map_err(|_| GraphError::Response)?,
        region: Region::parse(text(fields.get(1))?).map_err(|_| GraphError::Response)?,
        kind: Kind::parse(text(fields.get(2))?).map_err(|_| GraphError::Response)?,
        id: ResourceId::parse(text(fields.get(3))?).map_err(|_| GraphError::Response)?,
    })
}

fn text(value: Option<&Value>) -> Result<&str, GraphError> {
    let text = match value {
        Some(Value::BulkString(bytes)) => {
            std::str::from_utf8(bytes).map_err(|_| GraphError::Response)?
        }
        Some(Value::SimpleString(text)) => text.as_str(),
        _ => return Err(GraphError::Response),
    };
    if text.len() > 4096 {
        return Err(GraphError::Response);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_and_oversized_responses_are_rejected() {
        assert!(rows(&Value::Nil, 1).is_err());
        let response = Value::Array(vec![Value::Array(vec![]), Value::Array(vec![Value::Nil])]);
        assert!(rows(&response, 0).is_err());
        assert!(edges(&response, 1).is_err());
        assert!(nodes(&response, 1).is_err());
        assert!(text(Some(&Value::BulkString(vec![255]))).is_err());
    }
}
