use super::*;
use crate::sweep_runtime::record::policy_lp_tests::fixture as oam_fixture;
use inventory_core::record::FieldId;

const LINK: &str = "arn:aws:oam:us-east-1:123456789012:link/00000000-0000-0000-0000-000000000002";
const SINK: &str = "arn:aws:oam:us-east-1:210987654321:sink/00000000-0000-0000-0000-000000000001";

#[path = "oam_tests/namespace.rs"]
mod namespace;

#[path = "oam_tests/v3_migration.rs"]
mod v3_migration;

async fn oam_page(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, operation: &str, account: &str, at: i64) -> TestResult<PageCommit> {
    oam_page_in_scope(store, catalog, operation, account, at, ("us-east-1", LINK, SINK)).await
}

async fn oam_page_in_scope(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, operation: &str, account: &str, at: i64, (home, link, sink): (&str, &str, &str)) -> TestResult<PageCommit> {
    oam_page_in_sweep(store, catalog, operation, account, at, (home, link, sink), SweepId::new(format!("oam-{at}"))?).await
}

async fn oam_page_in_sweep(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, operation: &str, account: &str, at: i64, (home, link, sink): (&str, &str, &str), sweep: SweepId) -> TestResult<PageCommit> {
    let plan = catalog.lookup("oam", operation).ok_or("OAM plan absent")?;
    let body = if operation == "ListLinks" { json!({"Items":[{"Arn":link,"SinkArn":sink}]}) } else { json!({"Items":[{"Arn":sink,"Id":"00000000-0000-0000-0000-000000000001"}]}) };
    let rows = oam_fixture::rows(plan, body)?;
    let home = RegionId::parse(home)?;
    let account = AccountId::parse(account)?;
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    let observations = rows.iter().map(|row| observation(catalog, record::IdentityContext { account: &account, region: &home }, plan, &Region::Regional(home.clone()), row, &request, &DiscoveryContext::default())).collect::<Result<Vec<_>, _>>()?;
    let unresolved = references(catalog, plan, &home, &rows, &observations, &request)?;
    let scope = ScopeKey::new(account, Region::Regional(home), ServiceId::parse("oam")?, OperationId::parse(operation)?, None)?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, request.into_bytes())?).await?;
    let lease = store.claim(&sweep, at, 10_000).await?.ok_or("OAM lease absent")?;
    Ok(PageCommit { lease, observations, unresolved, edges: Vec::new(), tags: Vec::new(), children: Vec::new(), next_token: None, continuation_digest: None, done: true, observed_at_ms: at })
}

async fn persist_oam(store: &BinaryStore, catalog: &RuntimeCatalog<'_>, operation: &str, account: &str, at: i64) -> TestResult {
    store.commit_page(oam_page(store, catalog, operation, account, at).await?).await?;
    Ok(())
}

#[tokio::test]
async fn given_actual_oam_link_in_account_a_when_foreign_sink_arrives_in_either_order_then_only_observed_account_b_resolves() -> TestResult {
    // Given real pinned OAM models, embedded policy/kind/relation mappings, and wire decoding.
    let bytes = oam_fixture::compiled("oam", "2022-06-10", &["ListLinks", "ListSinks"], "AWS::Oam::Link")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for sink_first in [false, true] {
        let (_directory, store) = store(&catalog).await?;
        // When the B-owned sink and A-owned link commit in either order.
        let operations = if sink_first { [("ListSinks", "210987654321"), ("ListLinks", "123456789012")] } else { [("ListLinks", "123456789012"), ("ListSinks", "210987654321")] };
        for (index, (operation, account)) in operations.into_iter().enumerate() {
            persist_oam(&store, &catalog, operation, account, i64::try_from(index)?.checked_add(1).ok_or("time overflow")?).await?;
        }
        // Then no source-account sink placeholder is invented and the foreign target is exact.
        let resources = store.projection_resources(None, 100).await?;
        assert_eq!(resources.len(), 2);
        assert_eq!(resources.iter().filter(|resource| resource.key.kind.as_str() == "AWS::Oam::Sink" && resource.key.account.as_str() == "123456789012").count(), 0);
        let edges = store.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1);
        let edge = edges.first().ok_or("OAM sink edge absent")?;
        assert_eq!(edge.from.account.as_str(), "123456789012");
        assert_eq!(edge.from.id.as_str(), LINK);
        assert_eq!(edge.to.account.as_str(), "210987654321");
        assert_eq!(edge.to.id.as_str(), SINK);
        assert_eq!(edge.to.kind.as_str(), "AWS::Oam::Sink");
        assert_eq!(edge.relation.as_str(), "TARGETS_SINK");
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_oam_page_when_public_store_receives_forged_relationships_then_every_transaction_is_rejected() -> TestResult {
    use inventory_core::inventory::{Edge, EdgeSource, Kind, Relation};
    // Given a checked native catalog and source observation decoded from the real OAM model.
    let bytes = oam_fixture::compiled("oam", "2022-06-10", &["ListLinks", "ListSinks"], "AWS::Oam::Link")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for attack in ["unreviewed-same-account", "unreviewed-cross-account", "candidate-kind", "target-path", "target-value", "owner", "region", "absent-source", "direct-same-account", "direct-cross-account"] {
        let (directory, store) = store(&catalog).await?;
        let mut page = oam_page(&store, &catalog, "ListLinks", "123456789012", 1).await?;
        let reference = page.unresolved.first_mut().ok_or("reviewed OAM reference absent")?;
        match attack {
            "unreviewed-same-account" | "unreviewed-cross-account" => {
                reference.relation = Relation::parse("UNREVIEWED")?;
                if attack == "unreviewed-same-account" {
                    reference.target = ReferenceTarget::new(AccountId::parse("123456789012")?, reference.target.region().clone(), reference.target.kind().clone(), reference.target.path().to_vec(), reference.target.value().clone())?;
                }
            }
            "candidate-kind" => {
                reference.target = ReferenceTarget::new(reference.target.account().clone(), reference.target.region().clone(), reference.target.kind().clone(), reference.target.path().to_vec(), reference.target.value().clone())?
                    .with_candidate_kinds(vec![reference.target.kind().clone(), Kind::parse("AWS::Oam::Link")?])?;
            }
            "target-path" => {
                reference.target = ReferenceTarget::new(reference.target.account().clone(), reference.target.region().clone(), reference.target.kind().clone(), vec![FieldId::new(1)?, FieldId::new(1)?], reference.target.value().clone())?;
            }
            "target-value" => {
                reference.target = ReferenceTarget::new(reference.target.account().clone(), reference.target.region().clone(), reference.target.kind().clone(), reference.target.path().to_vec(), ResourceId::parse(&SINK.replace("000000000001", "000000000003"))?)?;
            }
            "owner" => {
                reference.target = ReferenceTarget::new(AccountId::parse("123456789012")?, reference.target.region().clone(), reference.target.kind().clone(), reference.target.path().to_vec(), reference.target.value().clone())?;
            }
            "region" => {
                reference.target = ReferenceTarget::new(reference.target.account().clone(), Region::parse("us-west-2")?, reference.target.kind().clone(), reference.target.path().to_vec(), reference.target.value().clone())?;
            }
            "absent-source" => page.observations.clear(),
            "direct-same-account" | "direct-cross-account" => {
                let to = ResourceKey { account: if attack == "direct-same-account" { reference.from.account.clone() } else { reference.target.account().clone() },
                    region: reference.target.region().clone(), kind: reference.target.kind().clone(), id: reference.target.value().clone() };
                page.edges.push(Edge { from: reference.from.clone(), to, relation: Relation::parse("UNREVIEWED")?, source: EdgeSource::ReviewedRule });
                page.unresolved.clear();
            }
            _ => return Err("unknown permission attack".into()),
        }
        // When an untrusted public caller bypasses the producer, no forged page may commit.
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
        type LeaseState = (i64, i64, i64, Option<Vec<u8>>, Vec<u8>, Vec<u8>);
        let before: LeaseState = sqlx::query_as("SELECT state,generation,lease_until,token,bindings,context FROM work_queue").fetch_one(&pool).await?;
        assert!(matches!(store.commit_page(page).await, Err(inventory_engine::inventory::StoreError::Invalid("relationship evidence is not permitted by the checked catalog"))), "{attack}");
        // Then rejection rolls back resources, graph evidence, and retained unresolved candidates.
        assert!(store.projection_resources(None, 100).await?.is_empty(), "{attack}");
        assert!(store.projection_edges(None, 100).await?.edges.is_empty(), "{attack}");
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs").fetch_one(&pool).await?, 0, "{attack}");
        let after: LeaseState = sqlx::query_as("SELECT state,generation,lease_until,token,bindings,context FROM work_queue").fetch_one(&pool).await?;
        assert_eq!(before, after, "{attack}");
        let extra: (i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM resource_facets),(SELECT count(*) FROM projection_outbox),(SELECT count(*) FROM reviewed_reference_evidence),(SELECT count(*) FROM reviewed_reference_grants)").fetch_one(&pool).await?;
        assert_eq!(extra, (0, 0, 0, 0), "{attack}");
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_legacy_matching_foreign_reference_when_catalog_is_attached_then_only_fresh_source_proof_can_promote_it() -> TestResult {
    let bytes = oam_fixture::compiled("oam", "2022-06-10", &["ListLinks", "ListSinks"], "AWS::Oam::Link")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for version in [14_i64, 15] {
        let (directory, original) = store(&catalog).await?;
        // Given an old real source whose payload declares a DIFFERENT foreign sink.
        let mut page = oam_page(&original, &catalog, "ListLinks", "123456789012", 1).await?;
        let plan = catalog.lookup("oam", "ListLinks").ok_or("OAM plan absent")?;
        let other = SINK.replace("000000000001", "000000000003");
        let rows = oam_fixture::rows(plan, json!({"Items":[{"Arn":LINK,"SinkArn":other}]}))?;
        let account = AccountId::parse("123456789012")?;
        let home = RegionId::parse("us-east-1")?;
        let request = Record::encode(&Value::Map(BTreeMap::new()))?;
        page.observations = rows.iter().map(|row| observation(&catalog, record::IdentityContext { account: &account, region: &home }, plan, &Region::Regional(home.clone()), row, &request, &DiscoveryContext::default())).collect::<Result<Vec<_>, _>>()?;
        page.unresolved = references(&catalog, plan, &home, &rows, &page.observations, &request)?;
        original.commit_page(page).await?;
        drop(original);
        let path = directory.path().join("inventory.sqlite");
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        let mut tx = pool.begin().await?;
        // This historical unchecked tuple matches a reviewed rule but not its actual source value.
        sqlx::query("UPDATE unresolved_refs SET target_value=?1").bind(SINK).execute(&mut *tx).await?;
        downgrade_oam(&mut tx, version).await?;
        tx.commit().await?;
        pool.close().await;
        // When catalog attachment is followed ONLY by an unrelated foreign target arrival.
        let reopened = BinaryStore::open_with_catalog(&path, &bytes).await?;
        persist_oam(&reopened, &catalog, "ListSinks", "210987654321", 2).await?;
        assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2);
        assert!(reopened.projection_edges(None, 100).await?.edges.is_empty(), "schema {version}");
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs WHERE from_id=?1").bind(LINK).fetch_one(&pool).await?, 1);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence").fetch_one(&pool).await?, 0);
        pool.close().await;
        // Then history stays dormant, but a later genuine source page can prove the same tuple.
        persist_oam(&reopened, &catalog, "ListLinks", "123456789012", 3).await?;
        let edges = reopened.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1, "schema {version}");
        let edge = edges.first().ok_or("fresh reviewed OAM edge absent")?;
        assert_eq!(edge.from.id.as_str(), LINK);
        assert_eq!(edge.to.id.as_str(), SINK);
        assert_eq!(edge.to.account.as_str(), "210987654321");
        assert_eq!(edge.relation.as_str(), "TARGETS_SINK");
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_oam_owner_when_source_partition_or_concrete_target_grammar_is_forged_then_public_store_rejects_atomically() -> TestResult {
    let bytes = oam_fixture::compiled("oam", "2022-06-10", &["ListLinks", "ListSinks"], "AWS::Oam::Link")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for attack in ["aws-us-gov", "aws-cn", "wildcard-many", "wildcard-one"] {
        let (directory, store) = store(&catalog).await?;
        let home = match attack { "aws-us-gov" => "us-gov-west-1", "aws-cn" => "cn-north-1", _ => "us-east-1" };
        let link = LINK.replace("us-east-1", home);
        let link = if attack.starts_with("aws-") { link.replacen("arn:aws:", &format!("arn:{attack}:"), 1) } else { link };
        let sink = SINK.replace("us-east-1", home);
        let mut page = oam_page_in_scope(&store, &catalog, "ListLinks", "123456789012", 1, (home, &link, &sink)).await?;
        if attack.starts_with("wildcard-") {
            let suffix = if attack == "wildcard-many" { "*" } else { "targe?" };
            let forged = format!("arn:aws:oam:{home}:210987654321:sink/{suffix}");
            let plan = catalog.lookup("oam", "ListLinks").ok_or("OAM plan absent")?;
            let rows = oam_fixture::rows(plan, json!({"Items":[{"Arn":link,"SinkArn":forged}]}))?;
            let account = AccountId::parse("123456789012")?;
            let home = RegionId::parse(home)?;
            let request = Record::encode(&Value::Map(BTreeMap::new()))?;
            page.observations = rows.iter().map(|row| observation(&catalog, record::IdentityContext { account: &account, region: &home }, plan, &Region::Regional(home.clone()), row, &request, &DiscoveryContext::default())).collect::<Result<Vec<_>, _>>()?;
            let reference = page.unresolved.first_mut().ok_or("OAM reference absent")?;
            reference.target = ReferenceTarget::new(reference.target.account().clone(), reference.target.region().clone(), reference.target.kind().clone(), reference.target.path().to_vec(), ResourceId::parse(&forged)?)?;
        }
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", directory.path().join("inventory.sqlite").display())).await?;
        type LeaseState = (i64, i64, i64, Option<Vec<u8>>, Vec<u8>, Vec<u8>);
        let before: LeaseState = sqlx::query_as("SELECT state,generation,lease_until,token,bindings,context FROM work_queue").fetch_one(&pool).await?;
        assert!(matches!(store.commit_page(page).await, Err(inventory_engine::inventory::StoreError::Invalid("relationship evidence is not permitted by the checked catalog"))), "{attack}");
        assert!(store.projection_resources(None, 100).await?.is_empty(), "{attack}");
        assert!(store.projection_edges(None, 100).await?.edges.is_empty(), "{attack}");
        let after: LeaseState = sqlx::query_as("SELECT state,generation,lease_until,token,bindings,context FROM work_queue").fetch_one(&pool).await?;
        assert_eq!(before, after, "{attack}");
        let persisted: (i64, i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM unresolved_refs),(SELECT count(*) FROM resource_facets),(SELECT count(*) FROM projection_outbox),(SELECT count(*) FROM reviewed_reference_evidence),(SELECT count(*) FROM reviewed_reference_grants)").fetch_one(&pool).await?;
        assert_eq!(persisted, (0, 0, 0, 0, 0), "{attack}");
        pool.close().await;
    }
    Ok(())
}

async fn downgrade_oam(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, version: i64) -> TestResult {
    sqlx::query("DROP TABLE reviewed_reference_evidence").execute(&mut **tx).await?;
    sqlx::query("DROP TABLE reviewed_reference_grants").execute(&mut **tx).await?;
    sqlx::query("DELETE FROM metadata WHERE key IN('reviewed_catalog','canonical_facet_dependencies_v1','canonical_facet_dependencies_v2','canonical_facet_dependencies_v3')").execute(&mut **tx).await?;
    sqlx::query("DROP TABLE resource_facets").execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context").execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN authority_generation").execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN authority_complete").execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds").execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution").execute(&mut **tx).await?;
    if version == 14 {
        sqlx::query("DROP TABLE tag_observations").execute(&mut **tx).await?;
        sqlx::query("PRAGMA user_version=14").execute(&mut **tx).await?;
    } else {
        sqlx::query("PRAGMA user_version=15").execute(&mut **tx).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_legacy_store_without_catalog_when_plain_open_commits_unrelated_observation_then_no_implicit_relationship_authority_is_granted() -> TestResult {
    use inventory_core::inventory::Edge;
    let bytes = oam_fixture::compiled("oam", "2022-06-10", &["ListLinks", "ListSinks"], "AWS::Oam::Link")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for version in [14_i64, 15] {
        let (directory, original) = store(&catalog).await?;
        persist_oam(&original, &catalog, "ListLinks", "123456789012", 1).await?;
        drop(original);
        let path = directory.path().join("inventory.sqlite");
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        let mut tx = pool.begin().await?;
        downgrade_oam(&mut tx, version).await?;
        tx.commit().await?;
        pool.close().await;
        // Given no reviewed catalog at all, empty proof storage is bookkeeping, not permission.
        let reopened = BinaryStore::open(&path).await?;
        persist_oam(&reopened, &catalog, "ListSinks", "210987654321", 2).await?;
        assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2, "schema {version}");
        assert!(reopened.projection_edges(None, 100).await?.edges.is_empty());
        for direct in [false, true] {
            let mut page = oam_page(&reopened, &catalog, "ListLinks", "123456789012", if direct { 4 } else { 3 }).await?;
            if direct {
                let reference = page.unresolved.first().ok_or("OAM reference absent")?;
                page.edges.push(Edge { from: reference.from.clone(), to: ResourceKey { account: reference.target.account().clone(), region: reference.target.region().clone(), kind: reference.target.kind().clone(), id: reference.target.value().clone() }, relation: reference.relation.clone(), source: reference.source });
                page.unresolved.clear();
            }
            assert!(matches!(reopened.commit_page(page).await, Err(inventory_engine::inventory::StoreError::Invalid("relationship evidence is not permitted by the checked catalog"))), "schema {version}, direct {direct}");
            assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2);
            assert!(reopened.projection_edges(None, 100).await?.edges.is_empty());
        }
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM metadata WHERE key='reviewed_catalog'").fetch_one(&pool).await?, 0);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_grants").fetch_one(&pool).await?, 0);
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence").fetch_one(&pool).await?, 0);
        pool.close().await;
    }
    Ok(())
}
