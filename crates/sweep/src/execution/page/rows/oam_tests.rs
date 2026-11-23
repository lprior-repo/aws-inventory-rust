use super::*;
use crate::record::policy_lp_tests::fixture as oam_fixture;
use inventory_core::record::FieldId;
mod contracts;
mod authority;
mod legacy_open;

const LINK: &str = "arn:aws:oam:us-east-1:123456789012:link/00000000-0000-0000-0000-000000000002";
const SINK: &str = "arn:aws:oam:us-east-1:210987654321:sink/00000000-0000-0000-0000-000000000001";

#[path = "oam_tests/namespace.rs"]
mod namespace;

#[path = "oam_tests/v3_migration.rs"]
mod v3_migration;

async fn oam_page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    account: &str,
    at: i64,
) -> TestResult<PageCommit> {
    oam_page_in_scope(
        store,
        catalog,
        operation,
        account,
        at,
        ("us-east-1", LINK, SINK),
    )
    .await
}

async fn oam_page_in_scope(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    account: &str,
    at: i64,
    (home, link, sink): (&str, &str, &str),
) -> TestResult<PageCommit> {
    oam_page_in_sweep(
        store,
        catalog,
        operation,
        account,
        at,
        (home, link, sink),
        SweepId::new(format!("oam-{at}"))?,
    )
    .await
}

async fn oam_page_in_sweep(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    account: &str,
    at: i64,
    (home, link, sink): (&str, &str, &str),
    sweep: SweepId,
) -> TestResult<PageCommit> {
    let plan = catalog.lookup("oam", operation).ok_or("OAM plan absent")?;
    let body = if operation == "ListLinks" {
        json!({"Items":[{"Arn":link,"SinkArn":sink}]})
    } else {
        json!({"Items":[{"Arn":sink,"Id":"00000000-0000-0000-0000-000000000001"}]})
    };
    let rows = oam_fixture::rows(plan, body)?;
    let home = RegionId::parse(home)?;
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
    let scope = ScopeKey::new(
        account,
        Region::Regional(home),
        ServiceId::parse("oam")?,
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
        .ok_or("OAM lease absent")?;
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

async fn persist_oam(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    operation: &str,
    account: &str,
    at: i64,
) -> TestResult {
    store
        .commit_page(oam_page(store, catalog, operation, account, at).await?)
        .await?;
    Ok(())
}



async fn downgrade_oam(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, version: i64) -> TestResult {
    sqlx::query("DROP TABLE reviewed_reference_evidence")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DROP TABLE reviewed_reference_grants")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM metadata WHERE key IN('reviewed_catalog','canonical_facet_dependencies_v1','canonical_facet_dependencies_v2','canonical_facet_dependencies_v3')").execute(&mut **tx).await?;
    sqlx::query("DROP TABLE resource_facets")
        .execute(&mut **tx)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context")
        .execute(&mut **tx)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN authority_generation")
        .execute(&mut **tx)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN authority_complete")
        .execute(&mut **tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds")
        .execute(&mut **tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution")
        .execute(&mut **tx)
        .await?;
    if version == 14 {
        sqlx::query("DROP TABLE tag_observations")
            .execute(&mut **tx)
            .await?;
        sqlx::query("PRAGMA user_version=14")
            .execute(&mut **tx)
            .await?;
    } else {
        sqlx::query("PRAGMA user_version=15")
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

