use super::*;
use crate::execution::{
    draining::append_plan_work,
    test_catalog,
    test_transport::{Resources, TestResult, serve_task_definition_pages},
};
use catalog_tool::{classes::DenyList, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, Region, RegionId},
};
use inventory_engine::inventory::{BinaryStore, InventoryStore, SchemaRegistry, SweepId};
use serde_json::Value;
use std::{collections::BTreeMap, net::SocketAddr, path::Path};
use tokio::{net::TcpListener, time::Duration};

const ACCOUNT: &str = "123456789012";
const REGION: &str = "us-east-1";
const STATUSES: [&str; 3] = ["ACTIVE", "INACTIVE", "DELETE_IN_PROGRESS"];
const REQUEST_COUNT: usize = 6;

#[tokio::test]
async fn status_branches_encode_tokens_decode_pages_and_persist_task_definition_arns() -> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let directory = tempfile::tempdir()?;
    let bytes = compile_catalog(directory.path(), address)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("ecs", "ListTaskDefinitions")
        .ok_or("ECS task-definition plan absent")?;
    let store = open_store(directory.path(), &catalog).await?;
    let resources = Resources::new(REGION)?;
    let progress = crate::control::Progress::new(None);
    let context = resources.runtime(&catalog, &store, &progress)?;
    let sweep = SweepId::new("ecs-status-product".to_owned())?;
    seed_branches(&context, &sweep, plan).await?;

    let (processed, captured) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::join!(
            process_status_branches(&context, &sweep),
            serve_task_definition_pages(listener, REQUEST_COUNT)
        )
    })
    .await?;
    processed?;
    let captured = captured?;
    assert_encoded_requests(&captured, address)?;
    assert_persisted_arns(&store).await?;
    Ok(())
}

async fn open_store(directory: &Path, catalog: &RuntimeCatalog<'_>) -> TestResult<BinaryStore> {
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    Ok(BinaryStore::create(&directory.join("ecs-status.sqlite"), registry).await?)
}

async fn seed_branches(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
    plan: &CallablePlan<'_>,
) -> TestResult {
    let account = AccountId::parse(ACCOUNT)?;
    let home = RegionId::parse(REGION)?;
    let mut work = Vec::new();
    append_plan_work(
        &account,
        &home,
        sweep,
        plan,
        &[Region::Regional(home.clone())],
        &mut work,
    )?;
    assert_eq!(work.len(), STATUSES.len());
    context.store.seed_works(&work).await?;
    Ok(())
}

async fn process_status_branches(
    context: &RuntimeContext<'_, '_, impl CredentialSource, impl InventoryStore>,
    sweep: &SweepId,
) -> TestResult {
    let mut pages = 0;
    let mut completed = 0_u64;
    while pages < REQUEST_COUNT {
        let Some(lease) = context
            .store
            .claim(sweep, crate::execution::now_ms()?, 60_000)
            .await?
        else {
            break;
        };
        let page = process(context, lease, false).await?;
        pages = pages.checked_add(1).ok_or("page count overflow")?;
        completed = completed
            .checked_add(page.summary.outcomes.complete)
            .ok_or("completed scope count overflow")?;
    }
    assert_eq!(pages, REQUEST_COUNT);
    assert_eq!(completed, u64::try_from(STATUSES.len())?);
    assert!(context
        .store
        .claim(sweep, crate::execution::now_ms()?, 60_000)
        .await?
        .is_none());
    Ok(())
}

fn assert_encoded_requests(
    captured: &[crate::execution::test_transport::Captured],
    address: SocketAddr,
) -> TestResult {
    assert_eq!(captured.len(), REQUEST_COUNT);
    let mut requests = captured
        .iter()
        .map(|request| {
            request.assert_route(address, REGION, "ecs")?;
            request.assert_operation("AmazonEC2ContainerServiceV20141113", "ListTaskDefinitions")?;
            let body: Value = serde_json::from_str(request.body())?;
            let status = body
                .get("status")
                .and_then(Value::as_str)
                .ok_or("encoded status absent")?;
            let token = body.get("nextToken").and_then(Value::as_str);
            assert_eq!(body.get("sort").and_then(Value::as_str), Some("ASC"));
            assert_eq!(body.get("maxResults").and_then(Value::as_u64), Some(100));
            Ok((status.to_owned(), token.map(str::to_owned)))
        })
        .collect::<TestResult<Vec<_>>>()?;
    requests.sort();
    let expected = [
        ("ACTIVE".to_owned(), None),
        ("ACTIVE".to_owned(), Some("active-page-2".to_owned())),
        ("DELETE_IN_PROGRESS".to_owned(), None),
        (
            "DELETE_IN_PROGRESS".to_owned(),
            Some("deleting-page-2".to_owned()),
        ),
        ("INACTIVE".to_owned(), None),
        ("INACTIVE".to_owned(), Some("inactive-page-2".to_owned())),
    ];
    assert_eq!(requests, expected);
    Ok(())
}

async fn assert_persisted_arns(store: &BinaryStore) -> TestResult {
    let mut arns = store
        .projection_resources(None, u16::try_from(REQUEST_COUNT)?)
        .await?
        .into_iter()
        .map(|resource| {
            assert_eq!(resource.key.kind.as_str(), "AWS::ECS::TaskDefinition");
            resource.key.id.as_str().to_owned()
        })
        .collect::<Vec<_>>();
    arns.sort();
    let expected = [
        "arn:aws:ecs:us-east-1:123456789012:task-definition/legacy:1",
        "arn:aws:ecs:us-east-1:123456789012:task-definition/legacy:2",
        "arn:aws:ecs:us-east-1:123456789012:task-definition/pending-removal:2",
        "arn:aws:ecs:us-east-1:123456789012:task-definition/pending-removal:3",
        "arn:aws:ecs:us-east-1:123456789012:task-definition/web:7",
        "arn:aws:ecs:us-east-1:123456789012:task-definition/web:8",
    ];
    assert_eq!(
        arns.iter().map(String::as_str).collect::<Vec<_>>(),
        expected
    );
    Ok(())
}

fn compile_catalog(directory: &Path, address: SocketAddr) -> TestResult<Vec<u8>> {
    let source = directory.join("source");
    let model_dir = source.join("models/ecs/service/2014-11-13");
    std::fs::create_dir_all(&model_dir)?;
    let model = "models/ecs/service/2014-11-13/ecs-2014-11-13.json";
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/aws-models");
    std::fs::copy(pinned.join(model), source.join(model))?;
    let policies = [overlay::embedded()?
        .into_iter()
        .find(|service| service.service() == "ecs")
        .ok_or("embedded ECS overlay absent")?];
    let checked = overlay::check_policies(&source, &policies, &DenyList::builtin())?;
    let registry = smithy::Registry::parse(include_str!("../../../../../catalog/field-ids.txt"))?;
    let base = smithy::build_catalog(&source, &registry, "ecs-status-product")?;
    let kinds = kind_map::embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    let endpoints = BTreeMap::from([(
        "ecs".to_owned(),
        test_catalog::loopback_endpoint("ecs", address)?,
    )]);
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?)
}
