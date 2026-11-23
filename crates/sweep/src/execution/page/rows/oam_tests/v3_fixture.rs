use super::*;
use catalog_tool::overlay;
use inventory_core::{inventory::ScopeOutcome, record::Key};
use sha2::{Digest, Sha256};
#[path = "v3_fixture/history.rs"]
mod history;
pub(super) use history::{historical_evidence, old_image};

pub(super) fn catalog_bytes() -> TestResult<Vec<u8>> {
    compiled_catalog(false)
}

pub(super) fn catalog_bytes_with_refresh() -> TestResult<Vec<u8>> {
    compiled_catalog(true)
}

fn compiled_catalog(refresh: bool) -> TestResult<Vec<u8>> {
    oam_fixture::compiled_with_operation(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
        |name, operation| {
            if name == "ListLinks" {
                operation.branches = [
                    ("metadata", overlay::Emission::EmitMetadata),
                    ("discover", overlay::Emission::DiscoveryOnly),
                ]
                .into_iter()
                .chain(refresh.then_some(("refresh", overlay::Emission::EmitMetadata)))
                .map(|(name, emission)| overlay::ExecutionBranch {
                    discriminator: name.to_owned(),
                    fixed: BTreeMap::new(),
                    bind: BTreeMap::new(),
                    emission: Some(emission),
                    row_policy: None,
                    scope: overlay::Scope::Region,
                    request_region: None,
                })
                .collect();
            }
        },
    )
}

pub(super) async fn page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    branch: &str,
    sweep: &str,
    at: i64,
    emit: bool,
) -> TestResult<PageCommit> {
    page_with_sink(store, catalog, branch, sweep, at, emit, Some(SINK)).await
}

pub(super) async fn page_with_sink(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    branch: &str,
    sweep: &str,
    at: i64,
    emit: bool,
    sink: Option<&str>,
) -> TestResult<PageCommit> {
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    source_page(store, catalog, branch, sweep, at, emit, (sink, request)).await
}

pub(super) async fn page_with_input(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    branch: &str,
    sweep: &str,
    at: i64,
    source: (Option<&str>, Record),
) -> TestResult<PageCommit> {
    source_page(store, catalog, branch, sweep, at, true, source).await
}

async fn source_page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    branch: &str,
    sweep: &str,
    at: i64,
    emit: bool,
    (sink, request): (Option<&str>, Record),
) -> TestResult<PageCommit> {
    let plan = catalog
        .lookup("oam", "ListLinks")
        .ok_or("OAM link plan absent")?;
    let body = sink.map_or_else(
        || json!({"Items":[{"Arn":LINK}]}),
        |sink| json!({"Items":[{"Arn":LINK,"SinkArn":sink}]}),
    );
    let rows = oam_fixture::rows(plan, body)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let observations = if emit {
        rows.iter()
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
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    let unresolved = references(
        catalog,
        plan,
        &home,
        if emit { &rows } else { &[] },
        &observations,
        &request,
        None,
    )?;
    let sweep = SweepId::new(sweep.to_owned())?;
    let scope = ScopeKey::new(
        account,
        Region::Regional(home),
        ServiceId::parse("oam")?,
        OperationId::parse("ListLinks")?,
        None,
    )?
    .with_branch(branch.to_owned())?;
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
        .ok_or("OAM branch lease absent")?;
    Ok(PageCommit {
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
}

pub(super) async fn pool(directory: &tempfile::TempDir) -> TestResult<sqlx::SqlitePool> {
    Ok(sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?)
}

pub(super) async fn frozen(
    pool: &sqlx::SqlitePool,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(String, Vec<u8>, Vec<String>)> {
    let canonical = sqlx::query_scalar("SELECT quote(account)||quote(region)||quote(kind)||quote(id)||quote(name)||quote(content_hash)||quote(first_seen)||quote(last_changed) FROM resources WHERE id=?1").bind(LINK).fetch_one(pool).await?;
    let schema = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind().as_str() == "AWS::Oam::Link")
        .ok_or("link schema absent")?;
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT record FROM \"");
    query
        .push(schema.table().as_str())
        .push("\" WHERE id=")
        .push_bind(LINK);
    let typed = query.build_query_scalar().fetch_one(pool).await?;
    let unresolved = sqlx::query_scalar("SELECT quote(target_scope)||quote(target_kind)||quote(target_path)||quote(target_value)||quote(relation)||quote(source)||quote(target_kinds)||quote(resolution) FROM unresolved_refs WHERE from_id=?1 ORDER BY target_value").bind(LINK).fetch_all(pool).await?;
    Ok((canonical, typed, unresolved))
}

pub(super) async fn partial(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    at: i64,
) -> TestResult {
    let mut page = page(store, catalog, "metadata", "fresh-partial", at, true).await?;
    page.done = false;
    page.next_token = Some(b"cursor".to_vec());
    page.continuation_digest = Some([91; 32]);
    store.commit_page(page).await?;
    let sweep = SweepId::new("fresh-partial".to_owned())?;
    let lease = store
        .claim(&sweep, at.saturating_add(1), 10_000)
        .await?
        .ok_or("partial continuation absent")?;
    store
        .finish_scope(lease, ScopeOutcome::Partial, at.saturating_add(2))
        .await?;
    Ok(())
}

pub(super) async fn facet_row(pool: &sqlx::SqlitePool, historical: bool) -> TestResult<String> {
    let table = if historical {
        "legacy_facet_history_v2"
    } else {
        "resource_facets"
    };
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT quote(account)||quote(region)||quote(kind)||quote(id)||quote(service)||quote(operation)||quote(source_key)||quote(precedence)||quote(record)||quote(owned)||quote(exclusions)||quote(relations)||quote(name)||quote(name_owned) FROM ",
    );
    query.push(table).push(" WHERE id=").push_bind(LINK);
    Ok(query.build_query_scalar().fetch_one(pool).await?)
}
