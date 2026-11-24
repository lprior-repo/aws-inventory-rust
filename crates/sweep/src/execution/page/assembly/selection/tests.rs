use super::*;
use crate::record::policy_dfgk_tests::fixture;
use inventory_core::{
    inventory::{OperationId, Region, ScopeKey, ServiceId},
    record::{Key, Record, Value},
};
use inventory_engine::{
    inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec},
    limiter::{ApiExecutor, ExecutorConfig},
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn lease(catalog: &RuntimeCatalog<'_>) -> TestResult<WorkLease> {
    let directory = tempfile::tempdir()?;
    let store = BinaryStore::create(
        &directory.path().join("selection.db"),
        SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
    )
    .await?;
    let sweep = SweepId::new("selection-ownership".into())?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("iottwinmaker")?,
        OperationId::parse("ListWorkspaces")?,
        None,
    )?;
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, Vec::new())?)
        .await?;
    Ok(store
        .claim(&sweep, 1000, 10_000)
        .await?
        .ok_or("selection lease absent")?)
}

fn select_row(catalog: &RuntimeCatalog<'_>, lease: &WorkLease) -> TestResult<(Record, Record)> {
    let plan = fixture::plan(catalog, "iottwinmaker", "ListWorkspaces")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let consumers = ConsumerIndex::build(catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = Context {
        catalog,
        account: &account,
        home: &home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    let record = fixture::decode_as_sweep(
        plan,
        br#"{"workspaceSummaries":[{
        "workspaceId":"abc", "arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/abc",
        "creationDateTime":0, "updateDateTime":1}]}"#,
    )?;
    let response = call::DecodedResponse {
        record,
        headers: http::HeaderMap::new(),
        http_version: http::Version::HTTP_11,
    };
    let raw = record::item_records(&response.record, plan.items())?
        .into_iter()
        .next()
        .ok_or("selected row absent")?;
    let pointer = raw.as_bytes().as_ptr();
    let baseline = raw.clone();
    let request = PageRequest {
        record: Record::encode(&Value::Map(BTreeMap::new()))?,
        region: home.clone(),
    };
    let ancestry = DiscoveryContext::empty();
    let page = page::Page::new(&context, lease, plan, response, &request, &ancestry)?;
    let chunk = select(&page, &mut std::iter::once(Ok(raw)), 0)?;
    assert_eq!(chunk.raw_count, 1);
    assert_eq!(chunk.items.len(), 1);
    let selected = chunk
        .items
        .into_iter()
        .next()
        .ok_or("projected row absent")?;
    if plan.projection().is_none() {
        assert_eq!(selected.as_bytes().as_ptr(), pointer);
    }
    Ok((baseline, selected))
}

#[tokio::test]
async fn given_optional_projection_when_selecting_then_exact_bytes_or_original_buffer_survive()
-> TestResult {
    use catalog_tool::overlay::{MemberPath, ResourceProjection};
    for projected in [false, true] {
        let bytes = crate::execution::test_catalog::compile(
            "iottwinmaker",
            "2021-11-29",
            &["ListWorkspaces"],
            None,
            |_, policy| {
                policy.projection = if projected {
                    Some(ResourceProjection::SensitiveDigest(MemberPath::new(
                        "workspaceId",
                    )?))
                } else {
                    None
                };
                Ok(())
            },
        )?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let lease = lease(&catalog).await?;
        let (baseline, selected) = select_row(&catalog, &lease)?;
        let mut expected = inventory_core::record::ValueRef::Map(baseline.root()).to_value()?;
        if projected {
            let field = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?
                .projection()
                .ok_or("projection absent")?
                .protected_path()
                .first()
                .copied()
                .ok_or("projection path empty")?;
            let Value::Map(fields) = &mut expected else {
                return Err("row is not a map".into());
            };
            fields.insert(
                Key::Field(field),
                Value::Text(
                    "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                        .into(),
                ),
            );
        }
        assert_eq!(selected.as_bytes(), Record::encode(&expected)?.as_bytes());
    }
    Ok(())
}
