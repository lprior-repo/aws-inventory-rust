use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};
use inventory_graph::{EdgeRow, NodeKey, NodeRow, Provenance};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn key(account: &str, id: &str) -> TestResult<NodeKey> {
    Ok(NodeKey {
        account: AccountId::parse(account)?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::EC2::Vpc")?,
        id: ResourceId::parse(id)?,
    })
}

pub fn row(key: &NodeKey) -> NodeRow {
    NodeRow {
        key: key.clone(),
        name: Some("O'Brien\\Cloud\n".into()),
        content_hash: [7; 32],
    }
}

pub fn edge(from: &NodeKey, to: &NodeKey, provenance: &str) -> TestResult<EdgeRow> {
    Ok(EdgeRow {
        from: from.clone(),
        to: to.clone(),
        relation: Relation::parse("CONTAINS")?,
        provenance: Provenance::parse(provenance)?,
    })
}
