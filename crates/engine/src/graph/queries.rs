pub(super) const INDEX_QUERY: &str = "CREATE INDEX FOR (r:Resource) ON (r.key)";
pub(super) const RESOURCE_QUERY: &str = "UNWIND $rows AS row MERGE (r:Resource {key:row.key}) SET r.id=row.id, r.scan_id=row.scan_id, r.kind=row.kind, r.account_id=row.account_id, r.region=row.region, r.name=row.name, r.properties_json=row.properties_json, r.tags_json=row.tags_json, r.source=row.source, r.observed_at=row.observed_at, r.unresolved=false";
pub(super) const EDGE_QUERY: &str = "UNWIND $rows AS row MERGE (a:Resource {key:row.from_key}) ON CREATE SET a.id=row.from_id, a.scan_id=row.scan_id, a.unresolved=true MERGE (b:Resource {key:row.to_key}) ON CREATE SET b.id=row.to_id, b.scan_id=row.scan_id, b.unresolved=true MERGE (a)-[e:RELATES_TO {kind:row.relation, source:row.source}]->(b) SET e.observed_at=row.observed_at, e.properties_json=row.properties_json";
pub(super) const MARKER_QUERY: &str =
    "MERGE (s:InventoryScan {id:$scan}) SET s.state=$state, s.resources=$resources, s.edges=$edges";
