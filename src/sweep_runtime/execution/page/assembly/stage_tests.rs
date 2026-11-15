use super::*;
use crate::sweep_runtime::{execution::{draining, now_ms, routing}, input, request};
use inventory_core::{inventory::Region, record::Record};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TEMPLATE_KIND: &str = "AWS::Proton::ServiceTemplate";
const VERSION_KIND: &str = "AWS::ApiOnly::ProtonServiceTemplateVersion";
const NEXT: &str = "same-cursor";

fn template_arn(name: &str) -> String {
    format!("arn:aws:proton:us-east-1:123456789012:service-template/{name}")
}

fn root_work(catalog: &RuntimeCatalog<'_>, account: &AccountId, home: &RegionId, sweep: &SweepId) -> TestResult<Vec<WorkSpec>> {
    let mut work = Vec::new();
    for plan in catalog.plans() {
        draining::append_plan_work(account, home, sweep, plan, &[Region::Regional(home.clone())], &mut work)?;
    }
    assert_eq!(work.len(), 1, "dependent stages must not become roots merely because their base bindings are empty");
    assert_eq!(work.first().ok_or("root absent")?.scope().operation().as_str(), "ListServiceTemplates");
    Ok(work)
}

fn wire_request(plan: &CallablePlan<'_>, lease: &WorkLease, home: &RegionId) -> TestResult<(PageRequest, Json)> {
    let region = routing::request_region(plan, lease.scope().region(), home, lease.scope().branch())?;
    let endpoint = routing::resolve(plan, &region)?;
    let base = Record::from_bytes(lease.bindings().to_vec())?;
    let mut fields = cursor::Cursor::decode(lease.token())?.request_fields(plan.pagination())?;
    if let Some(size) = plan.pagination().page_size.as_ref() {
        fields.push(input::InputField { path: size.path.clone(), value: size.value() });
    }
    let wire = request::encode(plan, &endpoint.url, &base, &fields)?;
    let body: Json = serde_json::from_slice(wire.body())?;
    assert_eq!(body.get("nextToken").and_then(Json::as_str), lease.token().is_some().then_some(NEXT));
    Ok((PageRequest { record: input::merge_input(&base, &fields)?, region }, body))
}

fn native_page(plan: &CallablePlan<'_>, lease: &WorkLease, wire: &Json) -> TestResult<Json> {
    let second = lease.token().is_some();
    let mut body = if plan.operation_name() == "ListServiceTemplates" {
        assert_eq!(lease.scope().branch(), None);
        let name = if second { "two" } else { "one" };
        json!({"templates":[{"name":name,"arn":template_arn(name),"createdAt":1,"lastModifiedAt":1}]})
    } else {
        let name = wire.get("templateName").and_then(Json::as_str).ok_or("template request binding absent")?;
        let stage = lease.scope().branch().ok_or("version stage absent")?;
        let (major, minor, selected_name) = match stage {
            "majors" => {
                assert!(wire.get("majorVersion").is_none(), "major discovery accidentally became a minor request");
                (if second { "2" } else { "1" }, "99", "not-the-requested-template")
            }
            "minors" => (wire.get("majorVersion").and_then(Json::as_str).ok_or("minor request major absent")?,
                if second { "2" } else { "1" }, name),
            _ => return Err("undefined stage reached production assembly".into()),
        };
        assert_eq!(lease.scope().parent().ok_or("stage lost real template parent")?.id.as_str(), template_arn(name));
        json!({"templateVersions":[{"templateName":selected_name,"majorVersion":major,"minorVersion":minor,
            "status":"PUBLISHED","createdAt":1,"lastModifiedAt":1,
            "arn":format!("{}:{major}.{minor}",template_arn(name))}]})
    };
    if !second { body.as_object_mut().ok_or("response object absent")?.insert("nextToken".into(), json!(NEXT)); }
    Ok(body)
}

async fn apply_page(store: &BinaryStore, context: &Context<'_, '_>, plan: &CallablePlan<'_>, lease: WorkLease,
    request: PageRequest, body: Json) -> TestResult {
    let response = call::DecodedResponse {
        record: crate::sweep_runtime::record::policy_dfgk_tests::fixture::decode(plan, &serde_json::to_vec(&body)?)?,
        headers: http::HeaderMap::new(),
    };
    let ancestry = super::super::preparation::ancestry(lease.context(), lease.scope().context())?;
    let cursor = cursor::Cursor::decode(lease.token())?;
    let parts = prepare(context, &lease, plan, cursor, response, &request, &ancestry).await?;
    if lease.scope().branch() == Some("majors") {
        assert!(parts.observations.is_empty(), "discovery stage emitted a fake latest-minor resource");
        assert_eq!(parts.rows, 0);
        assert_eq!(parts.children.len(), 1, "major must feed one minor stage, never itself");
        assert_eq!(parts.children.first().ok_or("minor child absent")?.scope().branch(), Some("minors"));
    }
    let done = parts.next.is_none();
    let continuation_digest = parts.next.as_ref().and_then(|advance| advance.continuation_digest);
    store.commit_page(PageCommit { lease, observations: parts.observations, edges: Vec::new(),
        unresolved: parts.references, tags: parts.tags, children: parts.children,
        next_token: parts.next.map(|advance| advance.cursor.into_bytes()), continuation_digest,
        done, observed_at_ms: now_ms()? }).await?;
    Ok(())
}

#[tokio::test]
async fn given_native_proton_stages_when_rooted_projected_and_reopened_per_page_then_only_minors_emit_and_each_cursor_keeps_its_real_parent() -> TestResult {
    let bytes = crate::sweep_runtime::record::policy_lp_tests::fixture::compiled("proton", "2020-07-20",
        &["ListServiceTemplates", "ListServiceTemplateVersions"], TEMPLATE_KIND)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("staged.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&database, registry).await?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("production-proton-stages".to_owned())?;
    store.seed_works(&root_work(&catalog, &account, &home, &sweep)?).await?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let executor = inventory_engine::limiter::ApiExecutor::new(Default::default())?;
    let context = Context { catalog: &catalog, account: &account, home: &home,
        executor: &executor, consumers: &consumers, max_pages: 10 };
    let mut seen = BTreeSet::new();
    for _ in 0..16 {
        let Some(lease) = store.claim(&sweep, now_ms()?, 3_600_000).await? else { break; };
        let plan = catalog.lookup(lease.scope().service().as_str(), lease.scope().operation().as_str())
            .ok_or("persisted reviewed plan absent")?;
        let (request, wire) = wire_request(plan, &lease, &home)?;
        assert!(seen.insert((plan.operation_name().to_owned(), lease.scope().branch().map(str::to_owned),
            wire.get("templateName").and_then(Json::as_str).map(str::to_owned),
            wire.get("majorVersion").and_then(Json::as_str).map(str::to_owned), lease.token().is_some())),
            "duplicate or cross-stage work admitted");
        let body = native_page(plan, &lease, &wire)?;
        apply_page(&store, &context, plan, lease, request, body).await?;
        drop(store);
        store = BinaryStore::open_with_catalog(&database, &bytes).await?;
    }
    assert_eq!(seen.len(), 14);
    let summary = store.sweep_summary(&sweep).await?;
    assert_eq!(summary.work_pending, 0);
    assert_eq!(summary.complete_scopes, 7);
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", database.display())).await?;
    let versions: BTreeSet<String> = sqlx::query_scalar("SELECT id FROM resources WHERE kind=?1")
        .bind(VERSION_KIND).fetch_all(&pool).await?.into_iter().collect();
    let expected: BTreeSet<String> = ["one", "two"].into_iter().flat_map(|name| ["1", "2"].into_iter().flat_map(move |major| {
        ["1", "2"].into_iter().map(move |minor| format!("{}:{major}.{minor}", template_arn(name)))
    })).collect();
    assert_eq!(versions, expected, "major latest-minor summaries are discovery context, not resource observations");
    let parents: Vec<(String, String)> = sqlx::query_as("SELECT parent_id,child_id FROM resource_parents WHERE child_kind=?1")
        .bind(VERSION_KIND).fetch_all(&pool).await?;
    assert_eq!(parents.len(), 8);
    for (parent, child) in parents { assert!(child.strip_prefix(&parent).is_some_and(|version| version.starts_with(':'))); }
    pool.close().await;
    Ok(())
}

#[test]
fn given_reviewed_local_stages_when_resuming_unknown_or_missing_stage_then_request_and_projection_fail_closed() -> TestResult {
    let bytes = crate::sweep_runtime::record::policy_lp_tests::fixture::compiled("proton", "2020-07-20",
        &["ListServiceTemplates", "ListServiceTemplateVersions"], TEMPLATE_KIND)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("proton", "ListServiceTemplateVersions").ok_or("version plan absent")?;
    let home = RegionId::parse("us-east-1")?;
    for branch in [None, Some("missing-stage")] {
        assert!(routing::execution_branch(plan, branch).is_err());
        assert!(routing::request_region(plan, &Region::Regional(home.clone()), &home, branch).is_err());
    }
    let ordinary = catalog.lookup("proton", "ListServiceTemplates").ok_or("template plan absent")?;
    assert!(routing::execution_branch(ordinary, None)?.is_none());
    assert_eq!(routing::request_region(ordinary, &Region::Regional(home.clone()), &home, None)?, home);
    assert!(routing::execution_branch(ordinary, Some("inherited-source-stage")).is_err());
    assert!(routing::request_region(ordinary, &Region::Regional(home.clone()), &home,
        Some("inherited-source-stage")).is_err());
    Ok(())
}
