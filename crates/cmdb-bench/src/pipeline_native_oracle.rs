use super::super::pipeline::{BenchResult, Workload};
use inventory_core::{inventory::{Kind, ResourceId, ResourceKey}, record::{Exclusions, ValueRef, scalar_record}};
use std::fmt::Write;

// Each indexed native lookup is checked against independently constructed
// fixture identity/hash bytes. At most 1,000 expected resources enter a query.
pub(super) fn query(workload: &Workload, start: usize) -> BenchResult<String> {
    let total = workload.resources()?;
    let end = start.saturating_add(1000).min(total);
    let per_scope = workload.rows_per_scope()?;
    if per_scope == 0 { return Err("native resource oracle requires a nonempty workload".into()); }
    let kind = Kind::parse("AWS::DynamoDB::Table")?;
    let mut query = String::new();
    query.try_reserve(512_000)?;
    query.push_str("UNWIND [");
    (start..end).try_for_each(|ordinal| -> BenchResult {
        if ordinal != start { query.push(','); }
        let scope = super::super::pipeline_fixture::scope(ordinal / per_scope)?;
        let name = format!("bench-{:012}", ordinal % per_scope);
        let id = format!("arn:aws:dynamodb:us-east-1:{}:table/{name}", scope.account().as_str());
        let key = ResourceKey { account: scope.account().clone(), region: scope.region().clone(), kind: kind.clone(), id: ResourceId::parse(&id)? };
        let record = scalar_record(ValueRef::Text(&name))?;
        let hash = record.content_hash(&Exclusions::none())?;
        query.push_str("{key:'");
        hex_into(&mut query, key.digest().as_bytes())?;
        query.push_str("',content_hash:'");
        hex_into(&mut query, hash.as_bytes())?;
        write!(query, "',account:'{}',name:'{name}',id:'{id}'}}", scope.account().as_str())?;
        Ok(())
    })?;
    query.push_str("] AS expected OPTIONAL MATCH (r:Resource {key:expected.key}) WITH expected,r WHERE r IS NULL OR r.account IS NULL OR r.account<>expected.account OR r.region IS NULL OR r.region<>'us-east-1' OR r.kind IS NULL OR r.kind<>'AWS::DynamoDB::Table' OR r.id IS NULL OR r.id<>expected.id OR r.name IS NULL OR r.name<>expected.name OR r.content_hash IS NULL OR r.content_hash<>expected.content_hash RETURN count(expected)");
    Ok(query)
}

fn hex_into(output: &mut String, bytes: &[u8; 32]) -> BenchResult {
    bytes.iter().try_for_each(|byte| write!(output, "{byte:02x}").map_err(Into::into))
}
