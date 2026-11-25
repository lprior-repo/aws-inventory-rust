use super::super::TestResult;
use inventory_core::{
    inventory::{OperationId, ResourceKey, ScopeKey, ServiceId, Tag, TagCoverage, TagObservation},
    record::{Record, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SweepId, WorkSpec};
use std::collections::BTreeMap;

pub(crate) async fn replace(
    store: &BinaryStore,
    resource: &ResourceKey,
    tags: Vec<(&str, &str)>,
    sweep: &str,
    at: i64,
) -> TestResult {
    observe(
        store,
        resource,
        Some(TagCoverage::Complete),
        tags,
        sweep,
        at,
    )
    .await
}

pub(crate) async fn observe(
    store: &BinaryStore,
    resource: &ResourceKey,
    coverage: Option<TagCoverage>,
    tags: Vec<(&str, &str)>,
    sweep: &str,
    at: i64,
) -> TestResult {
    let (service, operation) = match resource.kind.as_str() {
        "AWS::S3::Bucket" => ("s3", "ListBuckets"),
        _ => ("ec2", "DescribeVpcs"),
    };
    let scope = ScopeKey::new(
        resource.account.clone(),
        resource.region.clone(),
        ServiceId::parse(service)?,
        OperationId::parse(operation)?,
        Some(resource.clone()),
    )?;
    let sweep = SweepId::new(sweep.into())?;
    let bindings = Record::encode(&Value::Map(BTreeMap::new()))?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            bindings.as_bytes().to_vec(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("tag-only work was not claimable")?;
    let tags = tags
        .into_iter()
        .map(|(key, value)| Tag::new(key, value))
        .collect::<Result<Vec<_>, _>>()?;
    let observations = coverage
        .map(|coverage| TagObservation::new(resource.clone(), coverage, tags))
        .transpose()?
        .into_iter()
        .collect();
    store
        .commit_page(PageCommit {
            lease,
            observations: Vec::new(),
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags: observations,
            next_token: None,
            continuation_digest: None,
            children: Vec::new(),
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}
