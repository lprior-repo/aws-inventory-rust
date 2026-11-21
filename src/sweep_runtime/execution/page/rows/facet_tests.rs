#[path = "facet_fixture.rs"]
mod fixture;
use super::*;
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

#[tokio::test]
async fn given_native_list_detail_and_metadata_when_sparse_list_repeats_then_canonical_state_and_edges_survive()
-> TestResult {
    // Given native WorkSpaces Web list, singleton detail, and metadata responses.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    let before = state(&directory, &catalog).await?;
    // When a later enumeration contains only the stable ARN.
    persist(
        &store,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL}]}),
        10,
    )
    .await?;
    // Then the detail name, typed values, stable hash/timestamp, and settings edge remain.
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(
        store
            .projection_resource(&key()?)
            .await?
            .ok_or("portal absent")?
            .name
            .as_deref(),
        Some("detail-name")
    );
    assert_eq!(field(&before.0, 2)?, Some(Value::Text(BROWSER.to_owned())));
    assert_eq!(
        field(&before.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    let edges = store.projection_edges_for(&key()?, None, 100).await?;
    assert_eq!(edges.edges.len(), 1);
    assert_eq!(
        edges
            .edges
            .first()
            .ok_or("settings edge absent")?
            .to
            .id
            .as_str(),
        BROWSER
    );
    Ok(())
}

#[tokio::test]
async fn given_identical_native_facets_when_completion_order_reverses_then_final_bytes_hash_name_and_edges_match()
-> TestResult {
    // Given independent durable stores with the same compiled native shapes and policies.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (left_dir, left) = store(&catalog).await?;
    let (right_dir, right) = store(&catalog).await?;
    // When list/detail/sibling responses commit in opposite order.
    full(&left, &catalog, false).await?;
    full(&right, &catalog, true).await?;
    // Then canonical state is independent of network completion order.
    let left_state = state(&left_dir, &catalog).await?;
    let right_state = state(&right_dir, &catalog).await?;
    assert_eq!(left_state.0, right_state.0);
    assert_eq!(left_state.2, right_state.2);
    assert_eq!(
        left.projection_resource(&key()?).await?,
        right.projection_resource(&key()?).await?
    );
    assert_eq!(
        left.projection_edges_for(&key()?, None, 100).await?.edges,
        right.projection_edges_for(&key()?, None, 100).await?.edges
    );
    Ok(())
}

#[tokio::test]
async fn given_previous_detail_fields_when_that_source_omits_them_then_only_owned_values_and_edges_are_removed()
-> TestResult {
    // Given complete detail plus a disjoint sibling metadata facet.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    // When the authoritative detail no longer includes its name or settings association.
    persist(
        &store,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        10,
    )
    .await?;
    // Then stale list values cannot resurrect cleared detail fields; siblings remain.
    let cleared = state(&directory, &catalog).await?;
    assert_eq!(field(&cleared.0, 2)?, None);
    assert_eq!(field(&cleared.0, 3)?, None);
    assert_eq!(
        field(&cleared.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    assert_eq!(
        store
            .projection_resource(&key()?)
            .await?
            .ok_or("portal absent")?
            .name,
        None
    );
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    // When a second sibling also clears its owned metadata, no stale value remains.
    persist(
        &store,
        &catalog,
        "GetPortalServiceProviderMetadata",
        json!({"portalArn":PORTAL}),
        11,
    )
    .await?;
    assert_eq!(field(&state(&directory, &catalog).await?.0, 7)?, None);
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

#[tokio::test]
async fn given_live_list_when_detail_is_absent_twice_after_reopen_then_only_detail_authority_retires()
-> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    let before = state(&directory, &catalog).await?;
    persist(&store, &catalog, "GetPortal", json!({}), 5).await?;
    assert_eq!(state(&directory, &catalog).await?, before);
    persist(
        &store,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL,"displayName":"list-name"}]}),
        6,
    )
    .await?;
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(&reopened, &catalog, "GetPortal", json!({}), 7).await?;
    let current = state(&directory, &catalog).await?;
    assert_eq!(field(&current.0, 2)?, None);
    assert_eq!(
        field(&current.0, 3)?,
        Some(Value::Text("list-name".to_owned()))
    );
    assert_eq!(
        field(&current.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    assert_eq!(current.1, 7);
    assert_ne!(current.2, before.2);
    assert_eq!(
        reopened
            .projection_resource(&key()?)
            .await?
            .ok_or("surviving portal absent")?
            .name
            .as_deref(),
        Some("list-name")
    );
    assert_eq!(
        reopened
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges,
        Vec::new()
    );
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs WHERE from_id=?1")
            .bind(PORTAL)
            .fetch_one(&pool)
            .await?,
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE id=?1")
            .bind(PORTAL)
            .fetch_one(&pool)
            .await?,
        2
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_competing_reference_owners_when_detail_wins_then_exact_edges_follow_canonical_leaf_and_sibling_survives()
-> TestResult {
    const DETAIL: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-2";
    const SIBLING: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-3";
    let bytes = fixture::compiled_competing()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for reverse in [false, true] {
        let (directory, store) = store(&catalog).await?;
        persist(&store, &catalog, "ListBrowserSettings", json!({"browserSettings":[{"browserSettingsArn":BROWSER},{"browserSettingsArn":DETAIL},{"browserSettingsArn":SIBLING}]}), 1).await?;
        let list = (
            "ListPortals",
            json!({"portals":[{"portalArn":PORTAL,"browserSettingsArn":BROWSER}]}),
        );
        let detail = (
            "GetPortal",
            json!({"portal":{"portalArn":PORTAL,"browserSettingsArn":DETAIL}}),
        );
        let sibling = (
            "GetPortalServiceProviderMetadata",
            json!({"portalArn":PORTAL,"zzMetadataBrowserSettingsArn":SIBLING}),
        );
        let operations = if reverse {
            [sibling, detail, list]
        } else {
            [list, detail, sibling]
        };
        for (index, (operation, body)) in operations.into_iter().enumerate() {
            persist(
                &store,
                &catalog,
                operation,
                body,
                i64::try_from(index)?
                    .checked_add(2)
                    .ok_or("time overflow")?,
            )
            .await?;
        }
        let current = state(&directory, &catalog).await?.0;
        assert_eq!(field(&current, 2)?, Some(Value::Text(DETAIL.to_owned())));
        assert_eq!(field(&current, 10)?, Some(Value::Text(SIBLING.to_owned())));
        assert_eq!(
            edge_image(&store).await?,
            vec![
                (
                    DETAIL.to_owned(),
                    "USES_BROWSER_SETTINGS".to_owned(),
                    inventory_core::inventory::EdgeSource::ReviewedRule
                        .as_str()
                        .to_owned()
                ),
                (
                    SIBLING.to_owned(),
                    "USES_METADATA_SETTINGS".to_owned(),
                    inventory_core::inventory::EdgeSource::ReviewedRule
                        .as_str()
                        .to_owned()
                ),
            ]
        );
        drop(store);
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        persist(
            &reopened,
            &catalog,
            "GetPortal",
            json!({"portal":{"portalArn":PORTAL}}),
            8,
        )
        .await?;
        let current = state(&directory, &catalog).await?.0;
        assert_eq!(field(&current, 2)?, None);
        assert_eq!(field(&current, 10)?, Some(Value::Text(SIBLING.to_owned())));
        assert_eq!(
            edge_image(&reopened).await?,
            vec![(
                SIBLING.to_owned(),
                "USES_METADATA_SETTINGS".to_owned(),
                inventory_core::inventory::EdgeSource::ReviewedRule
                    .as_str()
                    .to_owned()
            ),]
        );
    }
    Ok(())
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
