use super::*;
use crate::record::policy_dfgk_tests::fixture;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, OperationId, ScopeKey, ServiceId},
    record::Value,
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, StoreError, SweepId, WorkSpec,
};
use serde_json::json;
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "arn:aws:iam::123456789012:role/collector";
const TARGET: &str = "arn:aws:iam::210987654321:role/target";

fn trust_rows(
    catalog: &RuntimeCatalog<'_>,
    owner: (&str, &str),
    document: &str,
) -> TestResult<(Vec<Record>, Vec<Observation>)> {
    let plan = fixture::plan(catalog, "iam", "ListRoles")?;
    let (account, role) = owner;
    let arn = format!("arn:aws:iam::{account}:role/{role}");
    let body = format!(
        "<ListRolesResponse><ListRolesResult><Roles><member><Path>/</Path><RoleName>{role}</RoleName><RoleId>AROA12345678901234567</RoleId><Arn>{arn}</Arn><CreateDate>2020-01-01T00:00:00Z</CreateDate><AssumeRolePolicyDocument>{document}</AssumeRolePolicyDocument></member></Roles><IsTruncated>false</IsTruncated></ListRolesResult></ListRolesResponse>"
    );
    let rows = fixture::rows(plan, body.as_bytes())?
        .iter()
        .map(|row| resource_projection::project(plan, row))
        .collect::<Result<Vec<_>, _>>()?;
    let account = AccountId::parse(account)?;
    let home = RegionId::parse("us-east-1")?;
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
                &Region::Global,
                row,
                &request,
                &DiscoveryContext::default(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((rows, observations))
}

async fn trust_page(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    owner: (&str, &str),
    document: &str,
    at: i64,
) -> TestResult<PageCommit> {
    let plan = fixture::plan(catalog, "iam", "ListRoles")?;
    let (rows, observations) = trust_rows(catalog, owner, document)?;
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    let unresolved = references(
        catalog,
        plan,
        &RegionId::parse("us-east-1")?,
        &rows,
        &observations,
        &request,
        None,
    )?;
    let sweep = SweepId::new(format!("trust-{at}"))?;
    let scope = ScopeKey::new(
        AccountId::parse(owner.0)?,
        Region::Global,
        ServiceId::parse("iam")?,
        OperationId::parse("ListRoles")?,
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
        .ok_or("IAM trust lease absent")?;
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

async fn trust_store(catalog: &RuntimeCatalog<'_>) -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

fn policy(effect: &str, principal: &str) -> String {
    json!({"Statement":{"Effect":effect,"Action":"sts:AssumeRole","Principal":{"AWS":principal}}})
        .to_string()
}

#[tokio::test]
async fn given_actual_iam_allow_principal_when_foreign_role_arrives_in_either_order_then_reviewed_trust_resolves()
-> TestResult {
    // Given actual pinned IAM query decoding, reviewed projection and native catalog authority.
    let catalog = fixture::catalog()?;
    for target_first in [false, true] {
        let (_directory, store) = trust_store(&catalog).await?;
        let source = ("123456789012", "collector");
        let target = ("210987654321", "target");
        let allow = policy("Allow", TARGET);
        let empty = json!({"Statement":[]}).to_string();
        let pages = if target_first {
            [(target, empty.as_str()), (source, allow.as_str())]
        } else {
            [(source, allow.as_str()), (target, empty.as_str())]
        };
        // When the genuine foreign resource and the Allow declaration commit in either order.
        for (index, (owner, document)) in pages.into_iter().enumerate() {
            let at = i64::try_from(index)?
                .checked_add(1)
                .ok_or("time overflow")?;
            store
                .commit_page(trust_page(&store, &catalog, owner, document, at).await?)
                .await?;
        }
        // Then the graph contains exactly the observed cross-account resource, never a placeholder.
        assert_eq!(store.projection_resources(None, 100).await?.len(), 2);
        let edges = store.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1);
        let edge = edges.first().ok_or("reviewed trust edge absent")?;
        assert_eq!(edge.from.id.as_str(), SOURCE);
        assert_eq!(edge.to.id.as_str(), TARGET);
        assert_eq!(edge.to.account.as_str(), "210987654321");
        assert_eq!(edge.to.region, Region::Global);
        assert_eq!(edge.relation.as_str(), "DECLARES_TRUST");
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_iam_trust_tuple_when_public_caller_forges_source_or_wildcard_then_store_rejects_atomically()
-> TestResult {
    let catalog = fixture::catalog()?;
    for attack in [
        "Deny",
        "different-principal",
        "service-principal",
        "wildcard-resource",
        "wildcard-single-character",
        "invalid-role-name",
        "missing-source",
    ] {
        let (directory, store) = trust_store(&catalog).await?;
        // Given a real producer reference, not a handcrafted allowlist or arbitrary relation tuple.
        let mut page = trust_page(
            &store,
            &catalog,
            ("123456789012", "collector"),
            &policy("Allow", TARGET),
            1,
        )
        .await?;
        let document = match attack {
            "Deny" => policy("Deny", TARGET),
            "different-principal" => policy("Allow", "arn:aws:iam::210987654321:role/other"),
            "service-principal" => json!({"Statement":{"Effect":"Allow","Principal":{"Service":"lambda.amazonaws.com"}}}).to_string(),
            "wildcard-resource" | "wildcard-single-character" | "invalid-role-name" => {
                let value = match attack { "wildcard-resource" => "arn:aws:iam::210987654321:role/target*", "wildcard-single-character" => "arn:aws:iam::210987654321:role/targe?", _ => "arn:aws:iam::210987654321:role/target#" };
                let reference = page.unresolved.first_mut().ok_or("trust reference absent")?;
                reference.target = ReferenceTarget::new(reference.target.account().clone(), Region::Global, reference.target.kind().clone(), reference.target.path().to_vec(), ResourceId::parse(value)?)?;
                policy("Allow", value)
            }
            "missing-source" => policy("Allow", TARGET),
            _ => return Err("unknown trust attack".into()),
        };
        // When the public caller bypasses reference production and substitutes a different summary.
        page.observations = trust_rows(&catalog, ("123456789012", "collector"), &document)?.1;
        if attack == "missing-source" {
            page.observations.clear();
        }
        let pool = sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            directory.path().join("inventory.sqlite").display()
        ))
        .await?;
        type LeaseState = (i64, i64, i64, Option<Vec<u8>>, Vec<u8>, Vec<u8>);
        let before: LeaseState = sqlx::query_as(
            "SELECT state,generation,lease_until,token,bindings,context FROM work_queue",
        )
        .fetch_one(&pool)
        .await?;
        assert!(
            matches!(
                store.commit_page(page).await,
                Err(StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ),
            "{attack}"
        );
        // Then no resource, candidate or edge survives the rejected transaction.
        assert!(
            store.projection_resources(None, 100).await?.is_empty(),
            "{attack}"
        );
        assert!(
            store.projection_edges(None, 100).await?.edges.is_empty(),
            "{attack}"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs")
                .fetch_one(&pool)
                .await?,
            0,
            "{attack}"
        );
        let after: LeaseState = sqlx::query_as(
            "SELECT state,generation,lease_until,token,bindings,context FROM work_queue",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(before, after, "{attack}");
        let extra: (i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM resource_facets),(SELECT count(*) FROM projection_outbox),(SELECT count(*) FROM reviewed_reference_evidence),(SELECT count(*) FROM reviewed_reference_grants)").fetch_one(&pool).await?;
        assert_eq!(extra, (0, 0, 0, 0), "{attack}");
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_direct_trust_edge_and_current_target_when_source_is_allow_or_deny_then_only_actual_allow_commits()
-> TestResult {
    use inventory_core::inventory::{Edge, EdgeSource};
    let catalog = fixture::catalog()?;
    for allow in [false, true] {
        let (_directory, store) = trust_store(&catalog).await?;
        // Both endpoints are the same actual observed IAM role; no endpoint is missing.
        let mut page = trust_page(
            &store,
            &catalog,
            ("123456789012", "collector"),
            &policy("Allow", SOURCE),
            1,
        )
        .await?;
        let reference = page
            .unresolved
            .first()
            .ok_or("self trust reference absent")?;
        page.edges.push(Edge {
            from: reference.from.clone(),
            to: reference.from.clone(),
            relation: reference.relation.clone(),
            source: EdgeSource::ReviewedRule,
        });
        page.unresolved.clear();
        if !allow {
            page.observations = trust_rows(
                &catalog,
                ("123456789012", "collector"),
                &policy("Deny", SOURCE),
            )?
            .1;
        }
        // A reviewed label alone cannot grant an edge, even with both real endpoints present.
        let result = store.commit_page(page).await;
        if allow {
            result?;
            assert_eq!(store.projection_resources(None, 100).await?.len(), 1);
            let edges = store.projection_edges(None, 100).await?.edges;
            assert_eq!(edges.len(), 1);
            let edge = edges.first().ok_or("self trust edge absent")?;
            assert_eq!(edge.from.id.as_str(), SOURCE);
            assert_eq!(edge.to.id.as_str(), SOURCE);
            assert_eq!(edge.relation.as_str(), "DECLARES_TRUST");
        } else {
            assert!(matches!(
                result,
                Err(StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ));
            assert!(store.projection_resources(None, 100).await?.is_empty());
            assert!(store.projection_edges(None, 100).await?.edges.is_empty());
        }
    }
    Ok(())
}
