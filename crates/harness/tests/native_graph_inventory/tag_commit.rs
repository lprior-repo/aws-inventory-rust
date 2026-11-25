use super::super::{TestResult, runtime};
use inventory_core::{
    catalog::runtime::{BindingSource, BindingValue, RuntimeCatalog},
    inventory::{OperationId, ResourceKey, ScopeKey, ServiceId, Tag, TagCoverage, TagObservation},
    record::{Key, Record, Value},
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
    let (scope, bindings) = fixture_work(resource, Some(resource.clone()))?;
    let sweep = SweepId::new(sweep.into())?;
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

pub(crate) fn fixture_work(
    resource: &ResourceKey,
    parent: Option<ResourceKey>,
) -> TestResult<(ScopeKey, Record)> {
    let (service, operation, fields) = match resource.kind.as_str() {
        "AWS::S3::Bucket" => ("s3", "ListBuckets", list_buckets_binding()?),
        "AWS::EC2::VPC" => ("ec2", "DescribeVpcs", BTreeMap::new()),
        "AWS::EC2::Subnet" => ("ec2", "DescribeSubnets", BTreeMap::new()),
        kind => return Err(format!("no catalog work fixture for {kind}").into()),
    };
    Ok((
        ScopeKey::new(
            resource.account.clone(),
            resource.region.clone(),
            ServiceId::parse(service)?,
            OperationId::parse(operation)?,
            parent,
        )?,
        Record::encode(&Value::Map(fields))?,
    ))
}

fn list_buckets_binding() -> TestResult<BTreeMap<Key, Value>> {
    let bytes = std::fs::read(runtime::catalog_path())?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("s3", "ListBuckets")
        .ok_or("ListBuckets missing from the checked catalog")?;
    let [binding] = plan.bindings() else {
        return Err("expected one fixed ListBuckets request binding (MaxBuckets)".into());
    };
    let ([field], BindingSource::Fixed(BindingValue::Integer(value))) =
        (binding.path.as_ref(), &binding.source)
    else {
        return Err("expected a top-level fixed integer ListBuckets request binding".into());
    };
    if catalog.catalog().field_name(*field)? != "MaxBuckets" {
        return Err("expected ListBuckets fixed integer to bind MaxBuckets".into());
    }
    Ok(BTreeMap::from([(Key::Field(*field), Value::Int(*value))]))
}
