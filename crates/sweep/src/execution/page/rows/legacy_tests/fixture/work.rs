use super::*;
pub(in super::super) async fn staged_work(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(SweepId, i64, Vec<u8>, Vec<u8>)> {
    let sweep = SweepId::new("legacy-inflight".to_owned())?;
    store
        .seed_work(work(&sweep, "ListPortals", Some(b"old-cursor".to_vec()))?)
        .await?;
    let lease = store
        .claim(&sweep, 10, 100)
        .await?
        .ok_or("old page absent")?;
    let mut page = native_page(catalog, lease, json!({"portals":[]}), 11)?;
    page.done = false;
    page.next_token = Some(b"next-cursor".to_vec());
    page.continuation_digest = Some([42; 32]);
    store.commit_page(page).await?;
    let lease = store
        .claim(&sweep, 12, 100)
        .await?
        .ok_or("inflight page absent")?;
    store
        .seed_work(work(&sweep, "GetPortalServiceProviderMetadata", None)?)
        .await?;
    Ok((
        sweep,
        lease.generation(),
        lease.bindings().to_vec(),
        lease.token().ok_or("old token absent")?.to_vec(),
    ))
}

pub(in super::super) async fn resume_first_page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    lease: WorkLease,
) -> TestResult {
    let mut page = native_page(
        catalog,
        lease,
        json!({"portals":[{"portalArn":PORTAL,"displayName":"buffered-list"}]}),
        20_001,
    )?;
    page.done = false;
    page.next_token = Some(b"fresh-cursor".to_vec());
    page.continuation_digest = Some([43; 32]);
    store.commit_page(page).await?;
    Ok(())
}

pub(in super::super) async fn finish_partial(store: &BinaryStore, sweep: &SweepId) -> TestResult {
    let lease = store
        .claim(sweep, 20_002, 100)
        .await?
        .ok_or("partial continuation absent")?;
    store
        .finish_scope(lease, ScopeOutcome::Partial, 20_003)
        .await?;
    let lease = store
        .claim(sweep, 20_004, 100)
        .await?
        .ok_or("queued metadata absent")?;
    store
        .finish_scope(lease, ScopeOutcome::Inaccessible, 20_005)
        .await?;
    Ok(())
}

pub(in super::super) async fn complete_empty(store: &BinaryStore, operation: &str, at: i64) -> TestResult {
    let sweep = SweepId::new(format!("empty-{at}"))?;
    store.seed_work(work(&sweep, operation, None)?).await?;
    let lease = store
        .claim(&sweep, at, 100)
        .await?
        .ok_or("empty authority absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations: Vec::new(),
            unresolved: Vec::new(),
            edges: Vec::new(),
            tags: Vec::new(),
            children: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: at.saturating_add(1),
        })
        .await?;
    Ok(())
}

pub(in super::super) fn child_work(sweep: &SweepId, id: &str, ordinal: Option<u32>) -> TestResult<WorkSpec> {
    use inventory_core::{
        catalog::runtime::ContextProjectionPlan,
        inventory::{DiscoveryContext, DiscoveryFrame},
    };
    let request = Record::encode(&Value::Map(BTreeMap::from([(
        inventory_core::record::Key::Field(FieldId::new(5)?),
        Value::Text(id.to_owned()),
    )])))?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("workspaces-web")?,
        OperationId::parse("GetPortal")?,
        Some(key()?),
    )?;
    let Some(ordinal) = ordinal else {
        return Ok(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            request.into_bytes(),
        )?);
    };
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan {
        selected: vec![vec![FieldId::new(5)?].into_boxed_slice()].into_boxed_slice(),
        ..ContextProjectionPlan::default()
    };
    let context = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("workspaces-web")?,
        OperationId::parse("ListPortals")?,
        ordinal,
        &projection,
        &request,
        &empty,
        &empty,
    )?)?;
    Ok(WorkSpec::new(
        sweep.clone(),
        scope.with_context(context.discriminator()?),
        None,
        request.into_bytes(),
    )?
    .with_context(context.encode()?)?)
}

pub(in super::super) async fn child_run(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    sweep: &str,
    id: &str,
    ordinal: Option<u32>,
    name: Option<&str>,
    at: i64,
) -> TestResult {
    let sweep = SweepId::new(sweep.to_owned())?;
    store.seed_work(child_work(&sweep, id, ordinal)?).await?;
    let lease = store
        .claim(&sweep, at, 100)
        .await?
        .ok_or("legacy child execution absent")?;
    let body = name.map_or_else(
        || json!({}),
        |name| json!({"portal":{"portalArn":id,"displayName":name}}),
    );
    store
        .commit_page(native_page(catalog, lease, body, at.saturating_add(1))?)
        .await?;
    Ok(())
}

pub(in super::super) async fn restore_raw_child_scopes(directory: &tempfile::TempDir) -> TestResult {
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
