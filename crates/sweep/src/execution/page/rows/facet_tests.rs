#[path = "facet_tests/fixture.rs"]
mod fixture;
use super::*;
mod observations;
mod retirement;
use fixture::{TestResult, compiled};
use inventory_core::{
    inventory::{AccountId, OperationId, ScopeKey, ServiceId},
    record::{FieldId, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::json;
use sqlx::Row;
use std::collections::BTreeMap;

const PORTAL: &str = "arn:aws:workspaces-web:us-east-1:123456789012:portal/p-1";
const BROWSER: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-1";

async fn store(catalog: &RuntimeCatalog<'_>) -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

fn key() -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: inventory_core::inventory::Kind::parse("AWS::WorkSpacesWeb::Portal")?,
        id: ResourceId::parse(PORTAL)?,
    })
}

async fn persist(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    body: serde_json::Value,
    at: i64,
) -> TestResult {
    persist_scoped(
        store,
        catalog,
        operation,
        body,
        "123456789012",
        "us-east-1",
        at,
    )
    .await
}

async fn persist_scoped(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    body: serde_json::Value,
    account: &str,
    region: &str,
    at: i64,
) -> TestResult {
    let plan = catalog
        .lookup("workspaces-web", operation)
        .ok_or("fixture plan absent")?;
    let body = serde_json::to_vec(&body)?;
    let response = aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: aws_wire::request::Protocol::RestJson1,
        body: &body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default(),
    })?;
    let mut rows = record::item_records(&response, plan.items())?;
    for row in &mut rows {
        *row = resource_projection::project(plan, row)?;
    }
    let home = RegionId::parse(region)?;
    let account = AccountId::parse(account)?;
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    let observations = rows
        .iter()
        .map(|row| {
            observation(
                catalog,
                record::IdentityContext {
                    account: &account,
                    region: &home,
                },
                plan,
                &Region::Regional(home.clone()),
                row,
                &request,
                &DiscoveryContext::default(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unresolved = references(catalog, plan, &home, &rows, &observations, &request, None)?;
    let sweep = SweepId::new(format!("facet-{at}"))?;
    let scope = ScopeKey::new(
        account,
        Region::Regional(home),
        ServiceId::parse("workspaces-web")?,
        OperationId::parse(operation)?,
        None,
    )?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            request.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, at, 10_000)
        .await?
        .ok_or("fixture work absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            unresolved,
            edges: Vec::new(),
            tags: Vec::new(),
            children: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}

async fn state(
    directory: &tempfile::TempDir,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(Record, i64, Vec<u8>)> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    let schema = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind().as_str() == "AWS::WorkSpacesWeb::Portal")
        .ok_or("portal schema absent")?;
    let mut query =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT record,last_changed,content_hash FROM \"");
    query
        .push(schema.table().as_str())
        .push("\" WHERE id=")
        .push_bind(PORTAL);
    let row = query.build().fetch_one(&pool).await?;
    let result = (
        Record::from_bytes(row.try_get("record")?)?,
        row.try_get("last_changed")?,
        row.try_get("content_hash")?,
    );
    pool.close().await;
    Ok(result)
}

async fn full(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, reverse: bool) -> TestResult {
    persist(
        store,
        catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        1,
    )
    .await?;
    let list = json!({"portals":[{"portalArn":PORTAL,"displayName":"list-name"}]});
    let detail = json!({"portal":{"portalArn":PORTAL,"displayName":"detail-name","browserSettingsArn":BROWSER}});
    let metadata = json!({"portalArn":PORTAL,"serviceProviderSamlMetadata":"<metadata/>"});
    let operations = if reverse {
        [
            ("GetPortalServiceProviderMetadata", metadata),
            ("GetPortal", detail),
            ("ListPortals", list),
        ]
    } else {
        [
            ("ListPortals", list),
            ("GetPortal", detail),
            ("GetPortalServiceProviderMetadata", metadata),
        ]
    };
    for (index, (operation, body)) in operations.into_iter().enumerate() {
        persist(
            store,
            catalog,
            operation,
            body,
            i64::try_from(index)?
                .checked_add(2)
                .ok_or("time overflow")?,
        )
        .await?;
    }
    Ok(())
}


#[path = "candidate_tests.rs"]
mod candidates;

#[path = "oam_tests.rs"]
mod oam;

#[path = "mask_tests.rs"]
mod masks;

fn field(record: &Record, field: u32) -> TestResult<Option<Value>> {
    Ok(record
        .root()
        .get_field(FieldId::new(field)?)?
        .map(|value| value.to_value())
        .transpose()?)
}


async fn edge_image(store: &BinaryStore) -> TestResult<Vec<(String, String, String)>> {
    let edges = store.projection_edges_for(&key()?, None, 100).await?.edges;
    let mut image = edges
        .into_iter()
        .map(|edge| {
            assert_eq!(edge.from, key()?);
            assert_eq!(edge.to.account.as_str(), "123456789012");
            assert_eq!(edge.to.region.as_str(), "us-east-1");
            assert_eq!(edge.to.kind.as_str(), "AWS::WorkSpacesWeb::BrowserSettings");
            Ok((
                edge.to.id.as_str().to_owned(),
                edge.relation.as_str().to_owned(),
                edge.source,
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    image.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(image)
}

#[path = "authority_tests.rs"]
mod authority;

#[path = "legacy_tests.rs"]
mod legacy;
