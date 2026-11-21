use super::*;
use inventory_core::inventory::ResourceKey;

async fn complete<'a, 'c>(
    mut scenario: Scenario<'a, 'c>,
    catalog: &RuntimeCatalog<'_>,
    status: &str,
    keys: &str,
) -> TestResult<(Scenario<'a, 'c>, Vec<ResourceKey>)> {
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10_000)
        .await?
        .ok_or("DNSSEC child absent")?;
    assert_eq!(lease.scope().operation().as_str(), "GetDNSSEC");
    let parts = prepare(&scenario, catalog, &lease, status, keys).await?;
    assert!(parts.next.is_none());
    let observed = parts
        .observations
        .iter()
        .map(|row| row.key().clone())
        .collect::<Vec<_>>();
    scenario
        .store
        .commit_page(PageCommit {
            lease,
            observations: parts.observations,
            edges: Vec::new(),
            unresolved: parts.references,
            tags: parts.tags,
            children: parts.children,
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: 1001,
        })
        .await?;
    let before = scenario.saved().await?;
    drop(scenario.store);
    scenario.store = BinaryStore::open(&scenario.directory.path().join("inventory.db")).await?;
    assert_eq!(
        scenario.saved().await?,
        before,
        "native multi-kind page changed after SQLite reopen"
    );
    assert_eq!(
        scenario
            .store
            .sweep_summary(&scenario.sweep)
            .await?
            .work_pending,
        0
    );
    Ok((scenario, observed))
}

async fn repeat<'a, 'c>(
    mut scenario: Scenario<'a, 'c>,
    catalog: &RuntimeCatalog<'_>,
    sweep: &str,
    status: &str,
    keys: &str,
) -> TestResult<Scenario<'a, 'c>> {
    scenario.sweep = SweepId::new(sweep.to_owned())?;
    seed_zone(&scenario, catalog).await?;
    complete(scenario, catalog, status, keys)
        .await
        .map(|(scenario, _)| scenario)
}

#[tokio::test]
async fn given_two_kinds_from_one_response_when_only_keys_disappear_then_completed_sources_retire_only_the_missing_kind_after_reopen()
-> TestResult {
    // Given native ListHostedZones -> GetDNSSEC, one atomic page, and a real reopened SQLite store.
    let catalog = fixture::catalog()?;
    let scenario = seeded(&catalog, "multi-kind-initial").await?;
    let (mut scenario, observed) = complete(scenario, &catalog, STATUS, KEY).await?;
    assert_eq!(observed.len(), 2);
    let status_key = observed
        .iter()
        .find(|key| key.kind.as_str() == "AWS::Route53::DNSSEC")
        .ok_or("status key absent")?;
    let signing_key = observed
        .iter()
        .find(|key| key.kind.as_str() == "AWS::Route53::KeySigningKey")
        .ok_or("signing key absent")?;
    let original = scenario
        .store
        .projection_resource(status_key)
        .await?
        .ok_or("status projection absent")?
        .content_hash;
    // When only keys disappear in two complete observations of the same operation source.
    scenario = repeat(
        scenario,
        &catalog,
        "multi-kind-key-miss-one",
        STATUS,
        "<KeySigningKeys/>",
    )
    .await?;
    assert!(
        scenario
            .store
            .projection_resource(signing_key)
            .await?
            .is_some()
    );
    scenario = repeat(
        scenario,
        &catalog,
        "multi-kind-key-miss-two",
        STATUS,
        "<KeySigningKeys/>",
    )
    .await?;
    // Then retained status keeps its original hash while only the absent key retires.
    assert!(
        scenario
            .store
            .projection_resource(signing_key)
            .await?
            .is_none()
    );
    assert_eq!(
        scenario
            .store
            .projection_resource(status_key)
            .await?
            .ok_or("status retired with key")?
            .content_hash,
        original
    );
    // And when the key returns, only the key row reappears and the status row is untouched.
    scenario = repeat(scenario, &catalog, "multi-kind-key-returned", STATUS, KEY).await?;
    assert_eq!(
        scenario
            .store
            .projection_resource(signing_key)
            .await?
            .ok_or("key did not return")?
            .key,
        *signing_key
    );
    assert_eq!(
        scenario
            .store
            .projection_resource(status_key)
            .await?
            .ok_or("status retired with key")?
            .content_hash,
        original
    );
    assert_eq!(
        scenario.saved().await?.len(),
        3,
        "hosted zone, status and returned key must remain"
    );
    Ok(())
}
