use super::*;
use crate::execution::page::child;
use crate::fixture_catalog::overlay::{OperationName, ServiceOverlay, SkipEvidence};
use inventory_core::inventory::{
    AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId,
};
use inventory_core::record::{Record, Value};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId};

fn compile(skip: bool) -> TestResult<fixture::Compiled> {
    let sources = [(
        "accessanalyzer",
        "2019-11-01",
        ["ListAnalyzers", "GetAnalyzer"].as_slice(),
    )];
    let ordinary = fixture::compile_many(&sources)?;
    if !skip {
        return Ok(ordinary);
    }
    let runtime = RuntimeCatalog::from_bytes(&ordinary.bytes)?;
    let fingerprint =
        SkipEvidence::fingerprint(&runtime, "accessanalyzer", "ListAnalyzers", "GetAnalyzer")?;
    fixture::compile_many_with(&sources, |service, policy| {
        let entries = policy
            .operations()
            .iter()
            .map(|(name, plan)| {
                let mut plan = plan.clone();
                if name.as_str() == "ListAnalyzers" {
                    let child = OperationName::new("GetAnalyzer")?;
                    plan.skip.push(child.clone());
                    plan.skip_evidence.insert(
                        child,
                        SkipEvidence::new(
                            service,
                            "ListAnalyzers",
                            "GetAnalyzer",
                            &fingerprint,
                            "test seeded analyzer selected members and raw values",
                            1,
                            "2026-11-24T00:00:00Z",
                        )?,
                    );
                }
                Ok((name.as_str(), plan))
            })
            .collect::<TestResult<Vec<_>>>()?;
        Ok(ServiceOverlay::new(service, entries)?)
    })
}

fn children(catalog: &RuntimeCatalog<'_>) -> TestResult<usize> {
    let parent = catalog
        .lookup("accessanalyzer", "ListAnalyzers")
        .ok_or("parent missing")?;
    let index = ConsumerIndex::build(catalog)?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("skip-test".into())?;
    let scope = ScopeKey::new(
        account.clone(),
        Region::Regional(region.clone()),
        ServiceId::parse("accessanalyzer")?,
        OperationId::parse("ListAnalyzers")?,
        None,
    )?;
    let context = child::Context::new(parent, &sweep, &account, &region, &index);
    let response = crate::record::policy_dfgk_tests::fixture::decode(parent, br#"{"analyzers":[{"arn":"arn:aws:access-analyzer:us-east-1:123456789012:analyzer/example","name":"example","type":"ACCOUNT","status":"ACTIVE"}]}"#)?;
    let request = Record::encode(&Value::Map(Default::default()))?;
    let (_, work) = child::policy_dfgk_tests::project_children(
        catalog,
        &context,
        &scope,
        &request,
        &response,
        &DiscoveryContext::empty(),
    )?;
    Ok(work.len())
}

#[test]
fn hand_skip_compiles_and_produces_no_child_work() -> TestResult {
    let ordinary = compile(false)?;
    let ordinary = RuntimeCatalog::from_bytes(&ordinary.bytes)?;
    assert_eq!(children(&ordinary)?, 1);
    let skipped = compile(true)?;
    let catalog = RuntimeCatalog::from_bytes(&skipped.bytes)?;
    let parent = catalog
        .lookup("accessanalyzer", "ListAnalyzers")
        .ok_or("parent missing")?;
    assert_eq!(parent.skips(), &[Box::<str>::from("GetAnalyzer")]);
    assert_eq!(children(&catalog)?, 0);
    Ok(())
}

#[tokio::test]
async fn skipped_child_kind_keeps_its_sqlite_table() -> TestResult {
    let compiled = compile(true)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let child = catalog
        .lookup("accessanalyzer", "GetAnalyzer")
        .ok_or("child missing")?;
    let parent = catalog
        .lookup("accessanalyzer", "ListAnalyzers")
        .ok_or("parent missing")?;
    assert_eq!(parent.kind(), child.kind());
    let schema = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind() == child.kind())
        .ok_or("schema missing")?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let registry = SchemaRegistry::new(&compiled.bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let pool = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .read_only(true),
    )
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(schema.table().as_str())
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

fn compile_pair(child: &str, operations: &[&str]) -> TestResult<fixture::Compiled> {
    fixture::compile_many_with(
        &[("accessanalyzer", "2019-11-01", operations)],
        |service, policy| {
            let entries = policy
                .operations()
                .iter()
                .map(|(name, plan)| {
                    let mut plan = plan.clone();
                    if name.as_str() == "ListAnalyzers" {
                        let child = OperationName::new(child)?;
                        plan.skip.push(child.clone());
                        plan.skip_evidence.insert(
                            child.clone(),
                            SkipEvidence::new(
                                service,
                                "ListAnalyzers",
                                child.as_str(),
                                &"a".repeat(64),
                                "seeded comparison",
                                1,
                                "2026-11-24",
                            )?,
                        );
                    }
                    Ok((name.as_str(), plan))
                })
                .collect::<TestResult<Vec<_>>>()?;
            Ok(ServiceOverlay::new(service, entries)?)
        },
    )
}

#[test]
fn given_different_child_kind_when_skip_compiles_then_missing_parent_emission_is_refused()
-> TestResult {
    let error = compile_pair("ListArchiveRules", &["ListAnalyzers", "ListArchiveRules"])
        .err()
        .ok_or("different child kind silently skipped")?;
    assert!(
        error
            .to_string()
            .contains("skip parent does not emit child kind"),
        "{error}"
    );
    Ok(())
}

#[test]
fn given_parent_child_dependent_chain_when_child_is_skipped_then_consumer_is_refused() -> TestResult
{
    let error = compile_pair(
        "ListArchiveRules",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"],
    )
    .err()
    .ok_or("skipped child's dependent silently dropped")?;
    assert!(
        error.to_string().contains("skip child has consumers"),
        "{error}"
    );
    Ok(())
}
