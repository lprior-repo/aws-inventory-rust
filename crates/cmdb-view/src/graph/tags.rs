use super::{GraphNode, ViewError, key_digest, response};
use std::{fmt::Write, time::Duration};

/// A tag value is scoped by account and region, not a synthetic AWS resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedTag {
    pub account: String,
    pub region: String,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagAffiliation {
    pub tag: ScopedTag,
    pub provenance: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagAffiliations {
    pub affiliations: Vec<TagAffiliation>,
    pub truncated: bool,
}

pub(super) async fn read(
    client: &redis::Client,
    graph: &str,
    node: &GraphNode,
    limit: usize,
) -> Result<TagAffiliations, ViewError> {
    let fetch = limit.checked_add(1).ok_or(ViewError::PageLimit)?;
    let query = query(node, fetch)?;
    let value = tokio::time::timeout(Duration::from_secs(10), async {
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(ViewError::Graph)?;
        redis::cmd("GRAPH.RO_QUERY")
            .arg(graph)
            .arg(query)
            .arg("TIMEOUT")
            .arg(10_000_u32)
            .query_async::<redis::Value>(&mut connection)
            .await
            .map_err(ViewError::Graph)
    })
    .await
    .map_err(|_| ViewError::GraphTimeout)??;
    let mut affiliations = parse(value, node, fetch)?;
    let truncated = affiliations.len() > limit;
    affiliations.truncate(limit);
    Ok(TagAffiliations {
        affiliations,
        truncated,
    })
}

fn query(node: &GraphNode, fetch: usize) -> Result<String, ViewError> {
    let digest = key_digest(node)?;
    let mut query = String::with_capacity(768);
    query.push_str("CYPHER");
    [
        ("key", digest.as_str()),
        ("account", node.account.as_str()),
        ("region", node.region.as_str()),
        ("kind", node.kind.as_str()),
        ("id", node.id.as_str()),
    ]
    .into_iter()
    .try_for_each(|(name, value)| {
        write!(query, " {name}=").map_err(|_| ViewError::GraphResponse)?;
        quote_into(&mut query, value)
    })?;
    query.push_str(" MATCH (s:Resource {key:$key,account:$account,region:$region,kind:$kind,id:$id})-[r:HAS_TAG]->(t:Tag) WHERE t.account=$account AND t.region=$region AND r.provenance='inventory.tags' RETURN t.account,t.region,t.tag_key,t.tag_value,r.provenance ORDER BY t.tag_key,t.tag_value,r.provenance LIMIT ");
    write!(query, "{fetch}").map_err(|_| ViewError::GraphResponse)?;
    Ok(query)
}

fn quote_into(query: &mut String, value: &str) -> Result<(), ViewError> {
    query.push('\'');
    value.chars().try_for_each(|character| {
        match character {
            '\\' => query.push_str("\\\\"),
            '\'' => query.push_str("\\'"),
            '\n' => query.push_str("\\n"),
            '\r' => query.push_str("\\r"),
            '\t' => query.push_str("\\t"),
            value if value.is_control() => {
                write!(query, "\\u{:04x}", u32::from(value))
                    .map_err(|_| ViewError::GraphResponse)?;
            }
            value => query.push(value),
        }
        Ok::<(), ViewError>(())
    })?;
    query.push('\'');
    Ok(())
}

fn parse(
    value: redis::Value,
    node: &GraphNode,
    fetch: usize,
) -> Result<Vec<TagAffiliation>, ViewError> {
    let redis::Value::Array(parts) = value else {
        return Err(ViewError::GraphResponse);
    };
    let Some(redis::Value::Array(rows)) = parts.get(1) else {
        return Err(ViewError::GraphResponse);
    };
    if rows.len() > fetch {
        return Err(ViewError::GraphResponse);
    }
    rows.iter().map(|row| parse_row(row, node)).collect()
}

fn parse_row(row: &redis::Value, node: &GraphNode) -> Result<TagAffiliation, ViewError> {
    let redis::Value::Array(fields) = row else {
        return Err(ViewError::GraphResponse);
    };
    if fields.len() != 5 {
        return Err(ViewError::GraphResponse);
    }
    let tag = ScopedTag {
        account: response::value_string(fields.first())?,
        region: response::value_string(fields.get(1))?,
        key: response::value_string(fields.get(2))?,
        value: response::value_string(fields.get(3))?,
    };
    let provenance = response::value_string(fields.get(4))?;
    if tag.account != node.account || tag.region != node.region || provenance != "inventory.tags" {
        return Err(ViewError::GraphResponse);
    }
    Ok(TagAffiliation { tag, provenance })
}
