use super::super::pipeline::{BenchResult, Workload};
use inventory_core::{
    inventory::{Kind, ResourceId, ResourceKey},
    record::{Exclusions, ValueRef, scalar_record},
};
use inventory_graph::NodeRow;

pub(super) fn rows(workload: &Workload, start: usize) -> BenchResult<Vec<NodeRow>> {
    let total = workload.resources()?;
    let end = start.saturating_add(1000).min(total);
    let per_scope = workload.rows_per_scope()?;
    if per_scope == 0 {
        return Err("native resource oracle requires a nonempty workload".into());
    }
    let kind = Kind::parse("AWS::DynamoDB::Table")?;
    (start..end)
        .map(|ordinal| -> BenchResult<NodeRow> {
            let scope = super::super::pipeline_fixture::scope(
                ordinal
                    .checked_div(per_scope)
                    .ok_or("scope index overflow")?,
            )?;
            let name = format!(
                "bench-{:012}",
                ordinal
                    .checked_rem(per_scope)
                    .ok_or("scope slot overflow")?
            );
            let id = format!(
                "arn:aws:dynamodb:us-east-1:{}:table/{name}",
                scope.account().as_str()
            );
            let key = ResourceKey {
                account: scope.account().clone(),
                region: scope.region().clone(),
                kind: kind.clone(),
                id: ResourceId::parse(&id)?,
            };
            let record = scalar_record(ValueRef::Text(&name))?;
            let hash = record.content_hash(&Exclusions::none())?;
            Ok(NodeRow {
                key,
                name: Some(name),
                content_hash: *hash.as_bytes(),
            })
        })
        .collect()
}
