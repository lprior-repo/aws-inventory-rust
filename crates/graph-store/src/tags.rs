use crate::falkor::FalkorDb;
use crate::{GraphError, NodeKey, cypher, response};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRow {
    pub identity: [u8; 32],
    pub key: String,
    pub value: String,
}

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

pub(crate) async fn read(
    graph: &FalkorDb,
    key: &NodeKey,
    limit: u16,
) -> Result<TagAffiliations, GraphError> {
    if limit == 0 || limit > 1000 {
        return Err(GraphError::Limits);
    }
    let fetch = usize::from(limit).saturating_add(1);
    let value = graph
        .read_request(&cypher::tag_affiliations(key, fetch)?)
        .await?;
    let mut affiliations = response::rows(&value, fetch)?
        .iter()
        .map(|row| parse(row, key))
        .collect::<Result<Vec<_>, _>>()?;
    let truncated = affiliations.len() > usize::from(limit);
    affiliations.truncate(usize::from(limit));
    Ok(TagAffiliations {
        affiliations,
        truncated,
    })
}

fn parse(row: &redis::Value, key: &NodeKey) -> Result<TagAffiliation, GraphError> {
    let fields = response::fields(row, 5)?;
    let tag = ScopedTag {
        account: response::text(fields.first())?.to_owned(),
        region: response::text(fields.get(1))?.to_owned(),
        key: response::text(fields.get(2))?.to_owned(),
        value: response::text(fields.get(3))?.to_owned(),
    };
    let provenance = response::text(fields.get(4))?.to_owned();
    if tag.account != key.account.as_str()
        || tag.region != key.region.as_str()
        || provenance != "inventory.tags"
    {
        return Err(GraphError::Response);
    }
    Ok(TagAffiliation { tag, provenance })
}
