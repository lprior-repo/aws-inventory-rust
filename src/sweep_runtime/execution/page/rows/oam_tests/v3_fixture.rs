use super::*;
use catalog_tool::overlay;
use inventory_core::{inventory::ScopeOutcome, record::Key};
use sha2::{Digest, Sha256};

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

pub(super) fn historical_evidence(
    catalog: &RuntimeCatalog<'_>,
    reference: &UnresolvedReference,
) -> TestResult<([u8; 32], ResourceKey)> {
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut hash = Sha256::new();
    // This is the historical source's literal domain, despite its V2 facet marker.
    hash.update(b"aws-inventory-reviewed-reference-v1");
    hash.update(registry.digest());
    for text in [
        reference.from.account.as_str(),
        reference.from.region.as_str(),
        reference.from.kind.as_str(),
        reference.from.id.as_str(),
        reference.target.account().as_str(),
        reference.target.region().as_str(),
        reference.target.kind().as_str(),
        reference.target.value().as_str(),
        reference.relation.as_str(),
        reference.source.as_str(),
    ] {
        hash.update(text.len().to_le_bytes());
        hash.update(text.as_bytes());
    }
    hash.update(reference.target.path().len().to_le_bytes());
    for field in reference.target.path() {
        hash.update(field.value().to_le_bytes());
    }
    hash.update(reference.target.candidate_kinds().len().to_le_bytes());
    for kind in reference.target.candidate_kinds() {
        hash.update(kind.as_str().len().to_le_bytes());
        hash.update(kind.as_str().as_bytes());
    }
    Ok((hash.finalize().into(), reference.from.clone()))
}

pub(super) async fn old_image(
    directory: &tempfile::TempDir,
    (proof, from): &([u8; 32], ResourceKey),
) -> TestResult {
    let pool = pool(directory).await?;
    let mut tx = pool.begin().await?;
    let source: Vec<u8> =
        sqlx::query_scalar("SELECT scope_key FROM work_queue WHERE sweep_id='old-discover'")
            .fetch_one(&mut *tx)
            .await?;
    let bytes: Vec<u8> = sqlx::query_scalar("SELECT relations FROM resource_facets WHERE id=?1")
        .bind(LINK)
        .fetch_one(&mut *tx)
        .await?;
    let relations = Record::from_bytes(bytes)?;
    let Value::List(mut entries) = relations
        .root()
        .get_field(FieldId::new(2)?)?
        .ok_or("facet references absent")?
        .to_value()?
    else {
        return Err("facet references are not a list".into());
    };
    assert_eq!(entries.len(), 1);
    for entry in &mut entries {
        let Value::List(tuple) = entry else {
            return Err("facet reference tuple absent".into());
        };
        assert_eq!(tuple.len(), 8);
        tuple.truncate(7); // The actual pre-V3 codec ended at the strict target pin.
    }
    let old_relations = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::List(entries),
    )])))?;
    sqlx::query("UPDATE resource_facets SET source_key=?1,relations=?2 WHERE id=?3")
        .bind(&source)
        .bind(old_relations.as_bytes())
        .bind(LINK)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE resource_scopes SET scope_key=?1,missing_complete=1,observed_sweep='old-discover',completed_sweep='old-discover' WHERE id=?2").bind(&source).bind(LINK).execute(&mut *tx).await?;
    for statement in [
        "DROP TABLE reviewed_reference_evidence",
        "DROP TABLE reviewed_reference_grants",
        "CREATE TABLE reviewed_reference_grants(proof BLOB NOT NULL CHECK(length(proof)=32),target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "CREATE TABLE reviewed_reference_evidence(proof BLOB NOT NULL CHECK(length(proof)=32),from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "ALTER TABLE resource_facets DROP COLUMN source_authority",
        "ALTER TABLE work_queue DROP COLUMN authority_generation",
        "ALTER TABLE work_queue DROP COLUMN authority_complete",
        "DROP TABLE legacy_observations",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_misses",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_completed_sweep",
        "DELETE FROM metadata WHERE key='canonical_facet_dependencies_v3'",
        "INSERT INTO metadata(key,value) VALUES('canonical_facet_dependencies_v2',X'01') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    ] {
        sqlx::query(statement).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO reviewed_reference_grants(proof,target_id) VALUES(?1,NULL)")
        .bind(proof.as_slice())
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO reviewed_reference_evidence(proof,from_account,from_region,from_kind,from_id,target_id) VALUES(?1,?2,?3,?4,?5,NULL)")
        .bind(proof.as_slice()).bind(from.account.as_str()).bind(from.region.as_str()).bind(from.kind.as_str()).bind(from.id.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    pool.close().await;
    Ok(())
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
        .claim(&sweep, at + 1, 10_000)
        .await?
        .ok_or("partial continuation absent")?;
    store
        .finish_scope(lease, ScopeOutcome::Partial, at + 2)
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
