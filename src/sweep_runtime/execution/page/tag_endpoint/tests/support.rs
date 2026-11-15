use super::*;
use inventory_core::{inventory::{Region, ResourceId, Tag, TagCoverage, TagObservation}, record::{Exclusions, Key}};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry};

pub(super) fn iam() -> TestResult<fixture::Compiled> {
    fixture::compile("iam", "2010-05-08", &["ListRoles", "ListRoleTags"])
}

pub(super) fn empty() -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(std::collections::BTreeMap::new()))?)
}

pub(super) fn parent(catalog: &RuntimeCatalog<'_>, registry: &fixture_catalog::smithy::Registry) -> TestResult<(Record, Observation)> {
    let plan = catalog.lookup("iam", "ListRoles").ok_or("parent plan missing")?;
    let id = "arn:aws:iam::123456789012:role/owner";
    let row = Record::encode(&Value::Map([
        (Key::Field(registry.id("Arn").ok_or("Arn missing")?), Value::Text(id.to_owned())),
        (Key::Field(registry.id("RoleName").ok_or("RoleName missing")?), Value::Text("owner".to_owned())),
    ].into_iter().collect()))?;
    let key = ResourceKey { account: AccountId::parse("123456789012")?, region: Region::Global, kind: plan.kind().clone(), id: ResourceId::parse(id)? };
    let observation = Observation::new(key, row.clone(), &Exclusions::none(), None)?.with_facet_precedence(plan.facet_precedence());
    Ok((row, observation))
}

pub(super) async fn seeded(compiled: &fixture::Compiled, sweep: &str) -> TestResult<(tempfile::TempDir, BinaryStore, ResourceKey, SweepId)> {
    let directory = tempfile::tempdir()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let registry = SchemaRegistry::new(&compiled.bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    let sweep = SweepId::new(sweep.to_owned())?;
    let (row, observation) = parent(&catalog, &compiled.fields)?;
    let owner = observation.key().clone();
    let parent_plan = catalog.lookup("iam", "ListRoles").ok_or("parent plan missing")?;
    let tag_plan = catalog.lookup("iam", "ListRoleTags").ok_or("tag plan missing")?;
    let request = empty()?;
    let context = ChildContext { parent_plan, plan: tag_plan, request: &request, sweep: &sweep, discovery: &[], branch: None };
    let mut budget = super::super::super::child::PageContextBudget::new(&DiscoveryContext::empty(),0)?;
    let child = child_work(&context, &row, &observation, &mut budget)?.ok_or("dedicated endpoint child not scheduled")?;
    let scope = ScopeKey::new(owner.account.clone(), owner.region.clone(), ServiceId::parse("iam")?, OperationId::parse("ListRoles")?, None)?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, empty()?.into_bytes())?).await?;
    let lease = claim(&store, &sweep, 0).await?;
    store.commit_page(PageCommit { lease, observations: vec![observation], edges: vec![], unresolved: vec![], tags: vec![TagObservation::new(owner.clone(), TagCoverage::Complete, vec![Tag::new("known", "old")?])?], next_token: None, continuation_digest: None, children: vec![child], done: true, observed_at_ms: 1 }).await?;
    Ok((directory, store, owner, sweep))
}

pub(super) async fn claim(store: &BinaryStore, sweep: &SweepId, now: i64) -> TestResult<WorkLease> {
    Ok(store.claim(sweep, now, 1000).await?.ok_or("pending work missing")?)
}

pub(super) fn xml(plan: &CallablePlan<'_>, tags: Option<&[(&str, &str)]>, marker: Option<&str>) -> TestResult<call::DecodedResponse> {
    let pairs = tags.map(|pairs| format!("<Tags>{}</Tags>", pairs.iter().map(|(key, value)| format!("<member><Key>{key}</Key><Value>{value}</Value></member>")).collect::<String>())).map_or_else(String::new, |value| value);
    let token = marker.map_or_else(|| "<IsTruncated>false</IsTruncated>".to_owned(), |token| format!("<IsTruncated>true</IsTruncated><Marker>{token}</Marker>"));
    let body = format!("<ListRoleTagsResponse xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><ListRoleTagsResult>{pairs}{token}</ListRoleTagsResult></ListRoleTagsResponse>");
    let record = aws_wire::response::decode(aws_wire::response::ResponseContext { service: plan.service(), operation: plan.operation(), protocol: aws_wire::request::Protocol::AwsQuery, body: body.as_bytes(), headers: &http::HeaderMap::new(), status: http::StatusCode::OK, limits: aws_wire::response::DecodeLimits::default().preserve_field_presence() })?;
    Ok(call::DecodedResponse { record, headers: http::HeaderMap::new() })
}

pub(super) fn consume(catalog: &RuntimeCatalog<'_>, lease: &WorkLease, response: call::DecodedResponse, cap: u32) -> TestResult<PageParts> {
    let plan = catalog.lookup("iam", "ListRoleTags").ok_or("tag plan missing")?;
    Ok(prepare_page(catalog, lease.scope().account(), cap, lease, plan, decode_cursor(lease.token())?, response)?)
}

pub(super) async fn commit(store: &BinaryStore, lease: WorkLease, parts: PageParts, at: i64) -> TestResult<inventory_engine::inventory::CommitResult> {
    assert!(parts.observations.is_empty(), "tag endpoint invented resource observations");
    assert!(parts.references.is_empty());
    let done = parts.next.is_none();
    let continuation_digest = parts.next.as_ref().and_then(|advance| advance.continuation_digest);
    Ok(store.commit_page(PageCommit { lease, observations: parts.observations, edges: vec![], unresolved: parts.references, tags: parts.tags, next_token: parts.next.map(|advance| advance.cursor.into_bytes()), continuation_digest, children: parts.children, done, observed_at_ms: at }).await?)
}

pub(super) async fn tags(store: &BinaryStore, owner: &ResourceKey) -> TestResult<Vec<(String, String)>> {
    let mut snapshot = store.begin_projection_snapshot().await?;
    let page = snapshot.tags_for(owner, None, 100).await?;
    assert!(page.next.is_none());
    snapshot.finish().await?;
    Ok(page.tags.into_iter().map(|tag| (tag.key, tag.value)).collect())
}

pub(super) fn pair(tags: &[inventory_core::inventory::TagObservation]) -> TestResult<&inventory_core::inventory::TagObservation> {
    assert_eq!(tags.len(), 1);
    Ok(tags.first().ok_or("owner tag snapshot missing")?)
}
