use super::super::super::{PageParts, PageRequest, assembly, call};
use super::*;
use crate::SweepError;
use inventory_core::inventory::Observation;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};

const STATUS: &str = "<Status><ServeSignature>SIGNING</ServeSignature></Status>";
const EMPTY_STATUS: &str = "<Status/>";
const KEY: &str =
    "<KeySigningKeys><member><Name>key-a</Name><Status>ACTIVE</Status></member></KeySigningKeys>";

async fn seeded<'a, 'c>(
    catalog: &'a RuntimeCatalog<'c>,
    name: &str,
) -> TestResult<Scenario<'a, 'c>> {
    let scenario = Scenario::new(catalog, name, "us-east-1").await?;
    seed_zone(&scenario, catalog).await?;
    Ok(scenario)
}

async fn seed_zone(scenario: &Scenario<'_, '_>, catalog: &RuntimeCatalog<'_>) -> TestResult {
    scenario
        .seed(fixture::plan(catalog, "route-53", "ListHostedZones")?)
        .await?;
    scenario.step(catalog, &mut |plan, _, _| {
        assert_eq!(plan.operation_name(), "ListHostedZones");
        Ok(b"<ListHostedZonesResponse><HostedZones><HostedZone><Id>/hostedzone/ZA</Id><Name>a.example.test.</Name><CallerReference>a</CallerReference><Config><PrivateZone>false</PrivateZone></Config></HostedZone></HostedZones><IsTruncated>false</IsTruncated><MaxItems>100</MaxItems></ListHostedZonesResponse>".to_vec())
    }).await
}

async fn prepare(
    scenario: &Scenario<'_, '_>,
    catalog: &RuntimeCatalog<'_>,
    lease: &WorkLease,
    status: &str,
    keys: &str,
) -> Result<PageParts, SweepError> {
    let plan = catalog
        .lookup("route-53", "GetDNSSEC")
        .ok_or(SweepError::Operation("DNSSEC plan absent"))?;
    let request = Record::from_bytes(lease.bindings().to_vec())?;
    let body = format!("<GetDNSSECResponse>{status}{keys}</GetDNSSECResponse>");
    let response = call::DecodedResponse {
        http_version: http::Version::HTTP_11,
        record: fixture::decode(plan, body.as_bytes())
            .map_err(|_| SweepError::Operation("native DNSSEC fixture decode failed"))?,
        headers: http::HeaderMap::new(),
    };
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = assembly::Context {
        catalog,
        account: &scenario.account,
        home: &scenario.home,
        executor: &executor,
        consumers: &scenario.consumers,
        max_pages: 10,
    };
    let ancestry =
        super::super::super::preparation::ancestry(lease.context(), lease.scope().context())?;
    assembly::prepare(
        &context,
        lease,
        plan,
        cursor::Cursor::default(),
        response,
        &PageRequest {
            record: request,
            region: scenario.home.clone(),
        },
        &ancestry,
    )
    .await
}

fn observed<'a>(parts: &'a PageParts, kind: &str) -> TestResult<&'a Observation> {
    parts
        .observations
        .iter()
        .find(|row| row.key().kind.as_str() == kind)
        .ok_or_else(|| "resource emission absent".into())
}

#[tokio::test]
async fn given_empty_status_when_key_exists_then_zone_identified_status_and_key_both_emit()
-> TestResult {
    // Given an actual public hosted-zone child request and a modeled-but-empty Status.
    let catalog = fixture::catalog()?;
    let scenario = seeded(&catalog, "multi-emission-empty-primary").await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10_000)
        .await?
        .ok_or("DNSSEC child absent")?;
    // When the production atomic page assembler consumes one decoded response.
    let parts = prepare(&scenario, &catalog, &lease, EMPTY_STATUS, KEY).await?;
    // Then the status row is identified by the bound zone alone and the key still emits beside it.
    assert_eq!(parts.observations.len(), 2);
    assert_eq!(
        observed(&parts, "AWS::Route53::DNSSEC")?
            .key()
            .kind
            .as_str(),
        "AWS::Route53::DNSSEC"
    );
    assert_eq!(
        observed(&parts, "AWS::Route53::KeySigningKey")?
            .key()
            .kind
            .as_str(),
        "AWS::Route53::KeySigningKey"
    );
    assert_eq!(parts.children.len(), 0);
    assert!(parts.next.is_none());
    Ok(())
}

#[tokio::test]
async fn given_no_keys_when_status_exists_then_primary_emitter_survives_empty_additional_selection()
-> TestResult {
    // Given a real child lease and an explicitly empty key collection.
    let catalog = fixture::catalog()?;
    let scenario = seeded(&catalog, "multi-emission-empty-additional").await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10_000)
        .await?
        .ok_or("DNSSEC child absent")?;
    // When selecting both resource descriptors from the same response.
    let parts = prepare(&scenario, &catalog, &lease, STATUS, "<KeySigningKeys/>").await?;
    // Then an empty sibling never erases or substitutes for the status resource.
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        observed(&parts, "AWS::Route53::DNSSEC")?
            .key()
            .kind
            .as_str(),
        "AWS::Route53::DNSSEC"
    );
    Ok(())
}

#[tokio::test]
async fn given_two_emissions_when_only_key_metadata_changes_then_status_identity_record_and_hash_remain_independent()
-> TestResult {
    // Given the same bound request and complete status/key response.
    let catalog = fixture::catalog()?;
    let scenario = seeded(&catalog, "multi-emission-independent-hashes").await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10_000)
        .await?
        .ok_or("DNSSEC child absent")?;
    let before = prepare(&scenario, &catalog, &lease, STATUS, KEY).await?;
    // When only the sibling key's metadata changes.
    let changed = KEY.replace("ACTIVE", "INACTIVE");
    let after = prepare(&scenario, &catalog, &lease, STATUS, &changed).await?;
    // Then descriptor-local records and content hashes remain independent.
    assert_eq!(before.observations.len(), 2);
    assert_eq!(after.observations.len(), 2);
    let primary = observed(&before, "AWS::Route53::DNSSEC")?;
    let primary_after = observed(&after, "AWS::Route53::DNSSEC")?;
    assert_eq!(primary.key(), primary_after.key());
    assert_eq!(primary.record(), primary_after.record());
    assert_eq!(primary.content_hash(), primary_after.content_hash());
    let key = observed(&before, "AWS::Route53::KeySigningKey")?;
    let key_after = observed(&after, "AWS::Route53::KeySigningKey")?;
    assert_eq!(key.key(), key_after.key());
    assert_ne!(key.content_hash(), key_after.content_hash());
    Ok(())
}

fn catalog_with_emission_exclusions() -> TestResult<Vec<u8>> {
    use catalog_tool::overlay::MemberPath;
    crate::execution::test_catalog::compile(
        "route-53",
        "2013-04-01",
        &["ListHostedZones", "GetDNSSEC"],
        None,
        |operation, policy| {
            if operation == "GetDNSSEC" {
                policy.exclude_from_hash = vec![MemberPath::new("ServeSignature")?];
                policy
                    .additional_resources
                    .first_mut()
                    .ok_or("key emission absent")?
                    .exclude_from_hash = vec![MemberPath::new("Status")?];
            }
            Ok(())
        },
    )
}

#[tokio::test]
async fn given_emission_exclusions_when_rows_move_then_legacy_bytes_hash_and_precedence_survive()
-> TestResult {
    let bytes = catalog_with_emission_exclusions()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = fixture::plan(&catalog, "route-53", "GetDNSSEC")?;
    let emission = plan
        .resource_emissions()
        .iter()
        .nth(1)
        .ok_or("key emission absent")?;
    assert_ne!(emission.hash_exclusions(), plan.hash_exclusions());
    assert_eq!(plan.facet_precedence(), 1);
    let scenario = seeded(&catalog, "emission-owned-hash").await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10_000)
        .await?
        .ok_or("DNSSEC child absent")?;
    let before = prepare(&scenario, &catalog, &lease, STATUS, KEY).await?;
    let after = prepare(
        &scenario,
        &catalog,
        &lease,
        STATUS,
        &KEY.replace("ACTIVE", "INACTIVE"),
    )
    .await?;
    let actual = observed(&before, "AWS::Route53::KeySigningKey")?;
    let response = fixture::decode(
        plan,
        format!("<GetDNSSECResponse>{STATUS}{KEY}</GetDNSSECResponse>").as_bytes(),
    )?;
    let legacy_row = record::item_records(&response, emission.items())?
        .into_iter()
        .next()
        .ok_or("legacy key row absent")?;
    let legacy = Observation::new(
        actual.key().clone(),
        legacy_row,
        emission.hash_exclusions(),
        None,
    )?
    .with_facet_precedence(1);
    assert_eq!(actual.record().as_bytes(), legacy.record().as_bytes());
    assert_eq!(actual.content_hash(), legacy.content_hash());
    assert_eq!(actual.hash_exclusions(), legacy.hash_exclusions());
    assert_eq!(actual.facet_precedence(), legacy.facet_precedence());
    let changed = observed(&after, "AWS::Route53::KeySigningKey")?;
    assert_ne!(actual.record().as_bytes(), changed.record().as_bytes());
    assert_eq!(actual.content_hash(), changed.content_hash());
    Ok(())
}

#[tokio::test]
async fn given_valid_status_and_key_when_later_key_identity_is_missing_then_atomic_page_has_no_partial_resource_or_cursor_commit()
-> TestResult {
    let catalog = fixture::catalog()?;
    let scenario = seeded(&catalog, "multi-emission-atomic-baseline").await?;
    let driver = atomic::Driver::new(&scenario.home).await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, super::super::super::now_ms()?, 60_000)
        .await?
        .ok_or("baseline DNSSEC child absent")?;
    let committed = driver
        .commit(&scenario, &catalog, lease, STATUS, KEY)
        .await?;
    assert_eq!(committed.outcomes.complete, 1);
    assert_eq!(committed.rows, 2);
    let mut scenario = scenario.reopen(&catalog).await?;
    let saved = scenario.saved().await?;
    assert_eq!(count(&saved, "AWS::Route53::DNSSEC"), 1);
    assert_eq!(count(&saved, "AWS::Route53::KeySigningKey"), 1);
    assert_eq!(
        scenario
            .edges()
            .await?
            .iter()
            .filter(|(_, _, _, _, relation)| relation == "FOR_HOSTED_ZONE")
            .count(),
        2
    );
    // Given retained native metadata, then a new real leased refresh of that source.
    scenario.sweep = SweepId::new("multi-emission-atomic-failure".to_owned())?;
    seed_zone(&scenario, &catalog).await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, super::super::super::now_ms()?, 60_000)
        .await?
        .ok_or("refresh DNSSEC child absent")?;
    let expected = atomic::snapshot(&scenario, atomic::Checkpoint::AfterFailure).await?;
    // Earlier items would change status/key-a and insert key-b if any prefix escaped.
    let status = "<Status><ServeSignature>NOT_SIGNING</ServeSignature></Status>";
    let keys = "<KeySigningKeys><member><Name>key-a</Name><Status>INACTIVE</Status></member><member><Name>key-b</Name><Status>ACTIVE</Status></member><member><Status>ACTIVE</Status></member></KeySigningKeys>";
    // When the real post-response driver encounters the later missing identity.
    let result = driver
        .commit(&scenario, &catalog, lease, status, keys)
        .await?;
    assert_eq!(result, atomic::failed());
    // Then real failure finalization survives BinaryStore close/reopen, but no prefix does.
    let scenario = scenario.reopen(&catalog).await?;
    atomic::assert_finalized(&scenario).await?;
    assert_eq!(
        atomic::snapshot(&scenario, atomic::Checkpoint::Current).await?,
        expected,
        "failed native multi-emission page changed checkpoint payload, canonical rows, facets, references or edges"
    );
    Ok(())
}

mod atomic;
mod lifecycle;
