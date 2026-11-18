use super::super::super::{child, consumer_index::ConsumerIndex};
use super::*;
use crate::sweep_runtime::{execution::routing, request};
use inventory_core::record::{Exclusions, Key};
use inventory_engine::inventory::{PageCommit, SchemaRegistry, WorkSpec};

fn waf_owner(
    catalog: &RuntimeCatalog<'_>,
    fields: &fixture_catalog::smithy::Registry,
    region: Region,
    arn: &str,
) -> TestResult<(Record, Observation)> {
    let plan = catalog
        .lookup("wafv2", "ListWebACLs")
        .ok_or("Web ACL producer absent")?;
    let row = Record::encode(&Value::Map(
        [
            (
                Key::Field(fields.id("ARN").ok_or("ARN field absent")?),
                Value::Text(arn.to_owned()),
            ),
            (
                Key::Field(fields.id("Name").ok_or("Name field absent")?),
                Value::Text("owner".to_owned()),
            ),
            (
                Key::Field(fields.id("Id").ok_or("Id field absent")?),
                Value::Text("11111111-1111-4111-8111-111111111111".to_owned()),
            ),
        ]
        .into_iter()
        .collect(),
    ))?;
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region,
        kind: plan.kind().clone(),
        id: inventory_core::inventory::ResourceId::parse(arn)?,
    };
    let observation = Observation::new(key, row.clone(), &Exclusions::none(), None)?
        .with_facet_precedence(plan.facet_precedence());
    Ok((row, observation))
}

fn tag_child(
    catalog: &RuntimeCatalog<'_>,
    scope: &ScopeKey,
    sweep: &SweepId,
    home: &inventory_core::inventory::RegionId,
    row: Record,
    observation: &Observation,
    request: &Record,
) -> TestResult<(WorkSpec, Vec<u8>)> {
    let plan = catalog
        .lookup("wafv2", "ListWebACLs")
        .ok_or("Web ACL producer absent")?;
    let consumers = ConsumerIndex::build(catalog)?;
    let context = child::Context::new(plan, sweep, scope.account(), home, &consumers);
    let ancestry = DiscoveryContext::empty();
    let rows = [row];
    let contexts = child::contexts(plan, &rows, request, &empty()?, &ancestry, scope)?;
    let discovery = contexts
        .first()
        .ok_or("producer context absent")?
        .encode()?;
    let mut budget = child::PageContextBudget::new(&ancestry, rows.len())?;
    let mut children = child::work(
        &context,
        &rows,
        std::slice::from_ref(observation),
        request,
        &contexts,
        scope,
        &mut budget,
    )?;
    assert_eq!(
        children.len(),
        1,
        "tag owner must select one reviewed target route, not fan out across both"
    );
    Ok((children.pop().ok_or("tag child absent")?, discovery))
}

async fn durable_tag_route(
    compiled: &fixture::Compiled,
    branch: &str,
    region: Region,
    arn: &str,
    expected_request_region: &str,
) -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let parent_plan = catalog
        .lookup("wafv2", "ListWebACLs")
        .ok_or("Web ACL producer absent")?;
    let parent_branch = parent_plan
        .branches()
        .iter()
        .find(|variant| variant.discriminator == branch)
        .ok_or("reviewed producer route absent")?;
    let (row, observation) = waf_owner(&catalog, &compiled.fields, region.clone(), arn)?;
    let owner = observation.key().clone();
    let home = inventory_core::inventory::RegionId::parse("us-west-2")?;
    let scope = ScopeKey::new(
        owner.account.clone(),
        region.clone(),
        ServiceId::parse("wafv2")?,
        OperationId::parse("ListWebACLs")?,
        None,
    )?
    .with_branch(branch.to_owned())?;
    let input = input::root_bindings_branch(
        parent_plan,
        Some(parent_branch),
        Some(&owner.account),
        Some(&region),
    )?;
    let sweep = SweepId::new(format!("tag-target-{branch}"))?;
    let (child, discovery) = tag_child(&catalog, &scope, &sweep, &home, row, &observation, &input)?;
    assert_eq!(child.scope().parent(), Some(&owner));
    assert_eq!(child.scope().branch(), Some(branch));
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let registry = SchemaRegistry::new(&compiled.bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&database, registry).await?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            input.into_bytes(),
        )?)
        .await?;
    let lease = claim(&store, &sweep, 0).await?;
    store
        .commit_page(PageCommit {
            lease,
            observations: vec![observation],
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags: Vec::new(),
            children: vec![child],
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: 1,
        })
        .await?;
    drop(store);
    let store = BinaryStore::open_with_catalog(&database, &compiled.bytes).await?;
    let lease = claim(&store, &sweep, 2).await?;
    assert_eq!(lease.scope().parent(), Some(&owner));
    assert_eq!(lease.scope().region(), &region);
    assert_eq!(lease.scope().branch(), Some(branch));
    assert_eq!(
        lease.context(),
        discovery.as_slice(),
        "source stage belongs in durable ancestry"
    );
    let plan = catalog
        .lookup("wafv2", "ListTagsForResource")
        .ok_or("tag target absent")?;
    let request_region =
        routing::request_region(plan, lease.scope().region(), &home, lease.scope().branch())?;
    assert_eq!(request_region.as_str(), expected_request_region);
    let endpoint = routing::resolve(plan, &request_region)?;
    let bindings = Record::from_bytes(lease.bindings().to_vec())?;
    let wire = request::encode(plan, &endpoint.url, &bindings, &[])?;
    let body: serde_json::Value = serde_json::from_slice(wire.body())?;
    assert_eq!(
        body.get("ResourceARN").and_then(serde_json::Value::as_str),
        Some(arn)
    );
    assert!(
        body.get("Scope").is_none(),
        "tag endpoint has no Scope input; route is its own reviewed branch"
    );
    Ok(())
}

#[tokio::test]
async fn given_native_waf_route_branches_when_tag_work_reopens_then_each_owner_keeps_only_its_reviewed_target_route()
-> TestResult {
    let compiled = fixture::compile(
        "wafv2",
        "2019-07-29",
        &[
            "ListWebACLs",
            "ListIPSets",
            "ListRegexPatternSets",
            "ListRuleGroups",
            "ListTagsForResource",
        ],
    )?;
    durable_tag_route(&compiled, "REGIONAL", Region::Regional(inventory_core::inventory::RegionId::parse("us-west-2")?),
        "arn:aws:wafv2:us-west-2:123456789012:regional/webacl/owner/11111111-1111-4111-8111-111111111111", "us-west-2").await?;
    durable_tag_route(&compiled, "CLOUDFRONT", Region::Global,
        "arn:aws:wafv2:us-east-1:123456789012:global/webacl/owner/11111111-1111-4111-8111-111111111111", "us-east-1").await?;
    Ok(())
}
