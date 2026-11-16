use super::*;
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::inventory::WorkLease;

pub(super) const SECONDARY: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-legacy-secondary";

pub(super) async fn pool(directory: &tempfile::TempDir) -> TestResult<sqlx::SqlitePool> {
    Ok(sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?)
}

pub(super) async fn baseline(store: &BinaryStore, catalog: &RuntimeCatalog<'_>) -> TestResult {
    full(store, catalog, false).await?;
    persist(store, catalog, "GetPortalServiceProviderMetadata", json!({"portalArn":PORTAL,"serviceProviderSamlMetadata":"<old/>","zzMetadataBrowserSettingsArn":SECONDARY}), 5).await
}

pub(super) fn work(sweep: &SweepId, operation: &str, token: Option<Vec<u8>>) -> TestResult<WorkSpec> {
    let scope = ScopeKey::new(AccountId::parse("123456789012")?, Region::parse("us-east-1")?, ServiceId::parse("workspaces-web")?, OperationId::parse(operation)?, None)?;
    let request = Record::encode(&Value::Map(BTreeMap::from([
        (inventory_core::record::Key::Field(FieldId::new(5)?), Value::Text(PORTAL.to_owned())),
    ])))?;
    Ok(WorkSpec::new(sweep.clone(), scope, token, request.into_bytes())?)
}

pub(super) fn native_page(catalog: &RuntimeCatalog<'_>, lease: WorkLease, body: serde_json::Value, at: i64) -> TestResult<PageCommit> {
    let plan = catalog.lookup("workspaces-web", lease.scope().operation().as_str()).ok_or("legacy plan absent")?;
    let body = serde_json::to_vec(&body)?;
    let response = aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(), operation: plan.operation(), protocol: aws_wire::request::Protocol::RestJson1,
        body: &body, headers: &http::HeaderMap::new(), status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default(),
    })?;
    let mut rows = record::item_records(&response, plan.items())?;
    for row in &mut rows {
        *row = resource_projection::project(&plan, row)?;
    }
    let home = RegionId::parse("us-east-1")?;
    let account = AccountId::parse("123456789012")?;
    let request = Record::from_bytes(lease.bindings().to_vec())?;
    let observations = rows.iter().map(|row| observation(catalog, record::IdentityContext { account: &account, region: &home }, &plan, &Region::Regional(home.clone()), row, &request, &DiscoveryContext::default())).collect::<Result<Vec<_>, _>>()?;
    let unresolved = references(catalog, &plan, &home, &rows, &observations, &request, lease.scope().parent())?;
    Ok(PageCommit { lease, observations, unresolved, edges: Vec::new(), tags: Vec::new(), children: Vec::new(), next_token: None, continuation_digest: None, done: true, observed_at_ms: at })
}

pub(super) async fn downgrade(directory: &tempfile::TempDir, version: u8) -> TestResult {
    let pool = pool(directory).await?;
    let mut tx = pool.begin().await?;
    for statement in [
        "DROP TABLE resource_facets",
        "DROP TABLE legacy_unknown_authorities",
        "DROP TABLE legacy_observations",
        "DROP TABLE legacy_refreshes",
        "DROP TABLE legacy_authorities",
        "DROP TABLE legacy_resources",
        "DROP TABLE reviewed_reference_evidence",
        "DROP TABLE reviewed_reference_grants",
        "DELETE FROM metadata WHERE key IN('reviewed_catalog','canonical_facet_dependencies_v1','canonical_facet_dependencies_v2','canonical_facet_dependencies_v3','semantic_authority_v1')",
        "ALTER TABLE resource_scopes DROP COLUMN completed_sweep",
        "ALTER TABLE work_queue DROP COLUMN context",
        "ALTER TABLE work_queue DROP COLUMN authority_generation",
        "ALTER TABLE work_queue DROP COLUMN authority_complete",
        "ALTER TABLE unresolved_refs DROP COLUMN target_kinds",
        "ALTER TABLE unresolved_refs DROP COLUMN resolution",
    ] { sqlx::query(statement).execute(&mut *tx).await?; }
    match version {
        14 => {
            sqlx::query("DROP TABLE tag_observations").execute(&mut *tx).await?;
            sqlx::query("PRAGMA user_version=14").execute(&mut *tx).await?;
        }
        15 => { sqlx::query("PRAGMA user_version=15").execute(&mut *tx).await?; }
        _ => return Err("unsupported legacy fixture version".into()),
    }
    tx.commit().await?;
    pool.close().await;
    Ok(())
}

pub(super) async fn image(directory: &tempfile::TempDir, catalog: &RuntimeCatalog<'_>) -> TestResult<((Record, i64, Vec<u8>), Vec<Vec<String>>)> {
    let canonical = state(directory, catalog).await?;
    let pool = pool(directory).await?;
    let mut tables = Vec::new();
    for query in [
        "SELECT quote(account)||quote(region)||quote(kind)||quote(id)||quote(name)||quote(content_hash)||quote(first_seen)||quote(last_changed) FROM resources ORDER BY account,region,kind,id",
        "SELECT quote(from_account)||quote(from_region)||quote(from_kind)||quote(from_id)||quote(target_scope)||quote(target_kind)||quote(target_path)||quote(target_value)||quote(relation)||quote(source) FROM unresolved_refs ORDER BY from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source",
        "SELECT quote(from_account)||quote(from_region)||quote(from_kind)||quote(from_id)||quote(to_account)||quote(to_region)||quote(to_kind)||quote(to_id)||quote(relation)||quote(source) FROM edges ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source",
        "SELECT quote(resource_key)||quote(revision)||quote(deleted)||quote(full_rebuild) FROM projection_outbox ORDER BY resource_key",
    ] { tables.push(sqlx::query_scalar::<_, String>(query).fetch_all(&pool).await?); }
    pool.close().await;
    Ok((canonical, tables))
}

pub(super) async fn work_image(directory: &tempfile::TempDir) -> TestResult<Vec<Vec<String>>> {
    let pool = pool(directory).await?;
    let mut image = Vec::new();
    for query in [
        "SELECT quote(sweep_id)||quote(scope_key)||quote(token)||quote(bindings)||quote(state)||quote(generation)||quote(lease_until) FROM work_queue ORDER BY sweep_id,scope_key",
        "SELECT quote(sweep_id)||quote(scope_key)||quote(digest) FROM work_tokens ORDER BY sweep_id,scope_key,digest",
        "SELECT quote(sweep_id)||quote(scope_key)||quote(outcome)||quote(observed_at) FROM scope_outcomes ORDER BY sweep_id,scope_key",
    ] { image.push(sqlx::query_scalar::<_, String>(query).fetch_all(&pool).await?); }
    pool.close().await;
    Ok(image)
}

pub(super) async fn staged_work(store: &BinaryStore, catalog: &RuntimeCatalog<'_>) -> TestResult<(SweepId, i64, Vec<u8>, Vec<u8>)> {
    let sweep = SweepId::new("legacy-inflight".to_owned())?;
    store.seed_work(work(&sweep, "ListPortals", Some(b"old-cursor".to_vec()))?).await?;
    let lease = store.claim(&sweep, 10, 100).await?.ok_or("old page absent")?;
    let mut page = native_page(catalog, lease, json!({"portals":[]}), 11)?;
    page.done = false;
    page.next_token = Some(b"next-cursor".to_vec());
    page.continuation_digest = Some([42; 32]);
    store.commit_page(page).await?;
    let lease = store.claim(&sweep, 12, 100).await?.ok_or("inflight page absent")?;
    store.seed_work(work(&sweep, "GetPortalServiceProviderMetadata", None)?).await?;
    Ok((sweep, lease.generation(), lease.bindings().to_vec(), lease.token().ok_or("old token absent")?.to_vec()))
}

pub(super) async fn resume_first_page(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, lease: WorkLease) -> TestResult {
    let mut page = native_page(catalog, lease, json!({"portals":[{"portalArn":PORTAL,"displayName":"buffered-list"}]}), 20_001)?;
    page.done = false;
    page.next_token = Some(b"fresh-cursor".to_vec());
    page.continuation_digest = Some([43; 32]);
    store.commit_page(page).await?;
    Ok(())
}

pub(super) async fn finish_partial(store: &BinaryStore, sweep: &SweepId) -> TestResult {
    let lease = store.claim(sweep, 20_002, 100).await?.ok_or("partial continuation absent")?;
    store.finish_scope(lease, ScopeOutcome::Partial, 20_003).await?;
    let lease = store.claim(sweep, 20_004, 100).await?.ok_or("queued metadata absent")?;
    store.finish_scope(lease, ScopeOutcome::Inaccessible, 20_005).await?;
    Ok(())
}

pub(super) async fn complete_empty(store: &BinaryStore, operation: &str, at: i64) -> TestResult {
    let sweep = SweepId::new(format!("empty-{at}"))?;
    store.seed_work(work(&sweep, operation, None)?).await?;
    let lease = store.claim(&sweep, at, 100).await?.ok_or("empty authority absent")?;
    store.commit_page(PageCommit { lease, observations: Vec::new(), unresolved: Vec::new(), edges: Vec::new(), tags: Vec::new(), children: Vec::new(), next_token: None, continuation_digest: None, done: true, observed_at_ms: at + 1 }).await?;
    Ok(())
}

pub(super) fn child_work(sweep: &SweepId, id: &str, ordinal: Option<u32>) -> TestResult<WorkSpec> {
    use inventory_core::{catalog::runtime::ContextProjectionPlan, inventory::{DiscoveryFrame, DiscoveryContext}};
    let request = Record::encode(&Value::Map(BTreeMap::from([
        (inventory_core::record::Key::Field(FieldId::new(5)?), Value::Text(id.to_owned())),
    ])))?;
    let scope = ScopeKey::new(AccountId::parse("123456789012")?, Region::parse("us-east-1")?, ServiceId::parse("workspaces-web")?, OperationId::parse("GetPortal")?, Some(key()?))?;
    let Some(ordinal) = ordinal else { return Ok(WorkSpec::new(sweep.clone(), scope, None, request.into_bytes())?); };
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan { selected: vec![vec![FieldId::new(5)?].into_boxed_slice()].into_boxed_slice(), ..ContextProjectionPlan::default() };
    let context = DiscoveryContext::empty().push(DiscoveryFrame::project(ServiceId::parse("workspaces-web")?, OperationId::parse("ListPortals")?, ordinal, &projection, &request, &empty, &empty)?)?;
    Ok(WorkSpec::new(sweep.clone(), scope.with_context(context.discriminator()?), None, request.into_bytes())?.with_context(context.encode()?)?)
}

pub(super) async fn child_run(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, sweep: &str, id: &str, ordinal: Option<u32>, name: Option<&str>, at: i64) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(child_work(&sweep, id, ordinal)?).await?;
    let lease = store.claim(&sweep, at, 100).await?.ok_or("legacy child execution absent")?;
    let body = name.map_or_else(|| json!({}), |name| json!({"portal":{"portalArn":id,"displayName":name}}));
    store.commit_page(native_page(catalog, lease, body, at + 1)?).await?;
    Ok(())
}

pub(super) async fn restore_raw_child_scopes(directory: &tempfile::TempDir) -> TestResult {
    let pool = pool(directory).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE resource_parents AS parent SET scope_key=(SELECT archived.scope_key FROM resource_scopes owner JOIN work_queue archived ON archived.sweep_id=owner.observed_sweep WHERE owner.account=parent.child_account AND owner.region=parent.child_region AND owner.kind=parent.child_kind AND owner.id=parent.child_id AND owner.scope_key=parent.scope_key) WHERE EXISTS(SELECT 1 FROM resource_scopes owner WHERE owner.account=parent.child_account AND owner.region=parent.child_region AND owner.kind=parent.child_kind AND owner.id=parent.child_id AND owner.scope_key=parent.scope_key AND owner.observed_sweep IN('z-old','a-new'))")
        .execute(&mut *tx).await?;
    sqlx::query("UPDATE resource_scopes SET scope_key=(SELECT archived.scope_key FROM work_queue archived WHERE archived.sweep_id=resource_scopes.observed_sweep) WHERE observed_sweep IN('z-old','a-new')")
        .execute(&mut *tx).await?;
    tx.commit().await?;
    pool.close().await;
    Ok(())
}
