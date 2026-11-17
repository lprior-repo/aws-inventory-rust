mod scenarios;

use super::{compile, cursor::Cursor, decode, TestResult};
use crate::sweep_runtime::{execution::page::child, input, record, request};
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::{KeyRef, Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

struct Page {
    operation: &'static str,
    request: Value,
    response: Value,
    ids: Vec<String>,
    parent: Option<String>,
    uri: String,
    body: Option<Value>,
    absent: Vec<&'static str>,
    cbor: bool,
}

impl Page {
    fn new(operation: &'static str, request: Value, response: Value, ids: Vec<String>) -> Self {
        Self { operation, body: Some(request.clone()), request, response, ids,
            parent: None, uri: "/".to_owned(), absent: Vec::new(), cbor: false }
    }

    fn rest(mut self, uri: String, body: Option<Value>) -> Self {
        self.uri = uri;
        self.body = body;
        self
    }

    fn cbor(mut self, shape: &str) -> Self {
        self.uri = format!("/service/{shape}/operation/{}", self.operation);
        self.cbor = true;
        self
    }

    fn under(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_owned());
        self
    }
}

// Fixture values are independent of runtime requests: a cross-parent request has no matching page.
fn later_pages(operation: &'static str, request: Value, parent: &str, items: &str,
    token: &str, item: Value, id: String, uri: &str, query_token: Option<&str>,
    absent: &[&'static str]) -> Vec<Page> {
    let mut continued_request = request.clone();
    continued_request[token] = json!("later");
    let mut first = Page::new(operation, request, json!({items:[],token:"later"}), Vec::new()).under(parent);
    let mut second = Page::new(operation, continued_request.clone(), json!({items:[item]}), vec![id]).under(parent);
    first.absent = absent.to_vec();
    second.absent = absent.to_vec();
    if uri != "/" {
        let continued_uri = query_token.map_or_else(|| uri.to_owned(), |name| {
            format!("{uri}{}{name}=later", if uri.contains('?') { "&" } else { "?" })
        });
        let continued_body = if query_token.is_some() { None } else { Some(json!({token:"later"})) };
        first = first.rest(uri.to_owned(), None);
        second = second.rest(continued_uri, continued_body);
    }
    vec![first, second]
}

fn request_json(record: &Record, fields: &catalog_tool::smithy::Registry) -> TestResult<Value> {
    let entries = record.root().iter()?.map(|entry| -> TestResult<(String, Value)> {
        let (key, value) = entry?;
        let name = match key {
            KeyRef::Field(id) => fields.name(id).ok_or("request field name absent")?,
            KeyRef::Name(name) => name,
        };
        match value {
            ValueRef::Text(value) => Ok((name.to_owned(), json!(value))),
            _ => Err("unexpected non-text request input".into()),
        }
    }).collect::<TestResult<serde_json::Map<_, _>>>()?;
    Ok(Value::Object(entries))
}

fn assert_wire(page: &Page, wire: &http::Request<bytes::Bytes>) -> TestResult {
    let expected: http::Uri = format!("https://fixture.example.test{}", page.uri).parse()?;
    assert_eq!(wire.uri().path(), expected.path(), "{} path", page.operation);
    let query = |uri: &http::Uri| -> BTreeSet<String> {
        uri.query().map_or_else(BTreeSet::new, |query| query.split('&').map(str::to_owned).collect())
    };
    assert_eq!(query(wire.uri()), query(&expected), "{} query", page.operation);
    if page.cbor {
        assert_eq!(wire.headers().get("smithy-protocol").and_then(|value| value.to_str().ok()), Some("rpc-v2-cbor"), "{} protocol", page.operation);
        assert_eq!(wire.method(), http::Method::POST, "{} method", page.operation);
        assert!(wire.uri().query().is_none(), "{} query", page.operation);
        let expected = page.body.as_ref().ok_or("CBOR page has no request body")?;
        assert_eq!(&crate::sweep_runtime::test_cbor::cbor_to_json(wire.body())?, expected, "{} body", page.operation);
        return Ok(());
    }
    match &page.body {
        Some(body) => assert_eq!(serde_json::from_slice::<Value>(wire.body())?, *body, "{} body", page.operation),
        None => assert!(wire.body().is_empty() || wire.body().as_ref() == b"{}", "unexpected REST payload"),
    }
    Ok(())
}

fn assert_rows(page: &Page, plan: &CallablePlan<'_>, response: &Record,
    fields: &catalog_tool::smithy::Registry) -> TestResult {
    let selected = record::item_records(response, plan.items())?;
    assert_eq!(selected.len(), page.ids.len(), "{} row count", page.operation);
    for row in selected {
        for name in &page.absent {
            assert!(record::path_value(row.root(), &[fields.id(name).ok_or("absent field unmodeled")?])?.is_none(),
                "request keys must not be manufactured in selected rows: {name}");
        }
    }
    Ok(())
}

async fn consume(service: &str, date: &str, roots: &[&str], kinds: &[(&str, &str)],
    excluded: &[&str], mut pages: Vec<Page>) -> TestResult {
    // Given the pinned models and actual embedded policies, not hand-authored runtime plans.
    let operations = pages.iter().map(|page| page.operation).chain(excluded.iter().copied()).collect::<BTreeSet<_>>();
    let compiled = compile(service, date, &operations.into_iter().collect::<Vec<_>>())?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for (operation, kind) in kinds {
        assert_eq!(catalog.lookup(service, operation).ok_or("child plan absent")?.kind().as_str(), *kind);
    }
    for operation in excluded {
        assert!(catalog.lookup(service, operation).is_none(), "excluded operation became executable: {operation}");
    }
    let expected = pages.iter().flat_map(|page| page.ids.iter().map(|id| (page.operation, id)))
        .map(|(operation, id)| -> TestResult<_> {
            Ok((catalog.lookup(service, operation).ok_or("fixture plan absent")?.kind().as_str().to_owned(), id.clone()))
        }).collect::<TestResult<BTreeSet<_>>>()?;
    let mut target_ids = BTreeMap::new();
    for page in &pages {
        let plan = catalog.lookup(service, page.operation).ok_or("fixture plan absent")?;
        let field = match plan.kind().as_str() {
            "AWS::APS::Workspace" => "workspaceId", "AWS::AppMesh::Mesh" => "meshName",
            "AWS::Backup::BackupPlan" => "BackupPlanId", "AWS::Backup::RestoreTestingPlan" => "RestoreTestingPlanName",
            "AWS::Bedrock::KnowledgeBase" => "knowledgeBaseId", "AWS::Bedrock::Agent" => "agentId",
            "AWS::Athena::WorkGroup" => "Name", "AWS::AppFabric::Ingestion" => "arn",
            "AWS::CodeArtifact::Domain" => "name", _ => continue,
        };
        let response = decode(plan, &page.response)?;
        let rows = record::item_records(&response, plan.items())?;
        assert_eq!(rows.len(), page.ids.len());
        for (row, id) in rows.iter().zip(&page.ids) {
            let field = compiled.fields.id(field).ok_or("target fixture field absent")?;
            let Some(ValueRef::Text(value)) = record::path_value(row.root(), &[field])? else { return Err("target fixture value absent".into()); };
            target_ids.insert((plan.kind().to_string(), value.to_owned()), id.clone());
        }
    }
    let mut expected_edges = BTreeSet::new();
    for page in &pages {
        let Some((input, target_kind, relation)) = parent_reference(page.operation) else { continue; };
        let value = page.request.get(input).and_then(Value::as_str).ok_or("fixture parent input absent")?;
        let target_id = target_ids.get(&(target_kind.to_owned(), value.to_owned())).ok_or("exact target fixture absent")?;
        let kind = catalog.lookup(service, page.operation).ok_or("fixture source plan absent")?.kind();
        for id in &page.ids {
            expected_edges.insert((kind.to_string(), id.clone(), target_kind.to_owned(), target_id.clone(), relation.to_owned()));
        }
    }
    let consumers = crate::sweep_runtime::execution::page::consumer_index::ConsumerIndex::build(&catalog)?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("children.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    let sweep = SweepId::new(format!("ac-children-{service}"))?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    for operation in roots {
        let plan = catalog.lookup(service, operation).ok_or("root plan absent")?;
        let scope = ScopeKey::new(account.clone(), region.clone(), ServiceId::parse(service)?, OperationId::parse(operation)?, None)?;
        let bindings = input::root_bindings(plan, Some(&account), Some(&region))?;
        store.seed_work(WorkSpec::new(sweep.clone(), scope, None, bindings.into_bytes())?).await?;
    }
    // When real queued work is resumed through projection, requests, cursors and SQLite after every page.
    let page_count = i64::try_from(pages.len())?;
    for at in 0..page_count {
        let lease = store.claim(&sweep, at, 10_000).await?.ok_or("inventory stopped before all fixture pages")?;
        assert_eq!(lease.scope().account(), &account);
        assert_eq!(lease.scope().region(), &region);
        let plan = catalog.lookup(service, lease.scope().operation().as_str()).ok_or("queued plan absent")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual_request = input::merge_input(&base, &fields)?;
        let actual_json = request_json(&actual_request, &compiled.fields)?;
        let index = pages.iter().position(|page| page.operation == plan.operation_name() && page.request == actual_json)
            .ok_or_else(|| format!("unexpected or cross-parent request: {} {actual_json}", plan.operation_name()))?;
        let page = pages.remove(index);
        assert_eq!(lease.scope().parent().map(|key| key.id.as_str()), page.parent.as_deref(), "{} parent", page.operation);
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        assert_wire(&page, &wire)?;
        let response = decode(plan, &page.response)?;
        assert_rows(&page, plan, &response, &compiled.fields)?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(&catalog, &context,
            lease.scope(), &actual_request, &response, &ancestry)?;
        assert_eq!(observations.iter().map(|row| row.key().id.as_str()).collect::<Vec<_>>(),
            page.ids.iter().map(String::as_str).collect::<Vec<_>>(), "{} identities", page.operation);
        for work in &children {
            assert!(observations.iter().any(|row| Some(row.key()) == work.scope().parent()), "child lost its selected parent");
        }
        let unresolved = child::policy_dfgk_tests::projected_references(&catalog, &context,
            lease.scope(), &actual_request, &response, &observations)?;
        if let Some((input, target_kind, relation)) = parent_reference(page.operation) {
            let expected_parent = page.request.get(input).and_then(Value::as_str).ok_or("fixture parent request missing")?;
            assert_eq!(unresolved.len(), observations.len());
            for reference in &unresolved {
                assert!(observations.iter().any(|row| row.key() == &reference.from));
                assert_eq!(reference.source, inventory_core::inventory::EdgeSource::ReviewedRule);
                assert_eq!(reference.target.kind().as_str(), target_kind);
                assert_eq!(reference.target.value().as_str(), expected_parent);
                assert_eq!(reference.relation.as_str(), relation);
                assert_eq!(reference.target.account(), &account);
                assert_eq!(reference.target.region(), &region);
            }
        }
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| (Some(next.cursor.into_bytes()), next.continuation_digest));
        store.commit_page(PageCommit { lease, observations, children, unresolved, edges: Vec::new(), tags: Vec::new(),
            next_token, continuation_digest, done, observed_at_ms: at }).await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then every exact scoped identity survives persistence, including empty-first-page continuations.
    assert_eq!(pages.len(), 0);
    assert!(store.claim(&sweep, page_count, 10_000).await?.is_none(), "unexpected executable operation remained");
    let resources = store.projection_resources(None, 100).await?;
    for row in &resources {
        assert_eq!(row.key.account, account);
        assert_eq!(row.key.region, region);
    }
    let actual = resources.iter().map(|row| (row.key.kind.as_str().to_owned(), row.key.id.as_str().to_owned())).collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    let joined_kinds = kinds.iter().filter(|(operation, _)| parent_reference(operation).is_some())
        .map(|(_, kind)| *kind).collect::<BTreeSet<_>>();
    let edges = store.projection_edges(None, 100).await?;
    let mut actual_edges = BTreeSet::new();
    let mut edge_count = 0_usize;
    for edge in edges.edges.into_iter().filter(|edge| joined_kinds.contains(edge.from.kind.as_str())) {
        edge_count = edge_count.checked_add(1).ok_or("edge count overflow")?;
        assert_eq!(edge.source, "reviewed-rule");
        assert_eq!(edge.from.account, account);
        assert_eq!(edge.to.account, account);
        assert_eq!(edge.from.region, region);
        assert_eq!(edge.to.region, region);
        actual_edges.insert((edge.from.kind.to_string(), edge.from.id.to_string(), edge.to.kind.to_string(), edge.to.id.to_string(), edge.relation.to_string()));
    }
    assert_eq!(edge_count, expected_edges.len());
    assert_eq!(actual_edges, expected_edges);
    Ok(())
}

fn parent_reference(operation: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match operation {
        "ListAnomalyDetectors" | "ListRuleGroupsNamespaces" => ("workspaceId", "AWS::APS::Workspace", "IN_WORKSPACE"),
        "ListGatewayRoutes" | "ListRoutes" => ("meshName", "AWS::AppMesh::Mesh", "IN_MESH"),
        "ListBackupSelections" => ("BackupPlanId", "AWS::Backup::BackupPlan", "IN_BACKUP_PLAN"),
        "ListRestoreTestingSelections" => ("RestoreTestingPlanName", "AWS::Backup::RestoreTestingPlan", "IN_RESTORE_TESTING_PLAN"),
        "ListDataSources" => ("knowledgeBaseId", "AWS::Bedrock::KnowledgeBase", "IN_KNOWLEDGE_BASE"),
        "ListAgentAliases" => ("agentId", "AWS::Bedrock::Agent", "IN_AGENT"),
        "ListNamedQueries" | "ListPreparedStatements" => ("WorkGroup", "AWS::Athena::WorkGroup", "IN_WORKGROUP"),
        "ListIngestionDestinations" => ("ingestionIdentifier", "AWS::AppFabric::Ingestion", "IN_INGESTION"),
        "ListPackageGroups" => ("domain", "AWS::CodeArtifact::Domain", "IN_DOMAIN"),
        _ => return None,
    })
}
