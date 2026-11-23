mod checks;
mod consumption;
mod scenarios;
use checks::verify_catalog;
use consumption::consume;

use super::{TestResult, compile, cursor::Cursor, decode};
use crate::{execution::page::child, input, record, request};
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
        Self {
            operation,
            body: Some(request.clone()),
            request,
            response,
            ids,
            parent: None,
            uri: "/".to_owned(),
            absent: Vec::new(),
            cbor: false,
        }
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
fn later_pages(
    (operation, request): (&'static str, Value),
    parent: &str,
    (items, token): (&str, &str),
    (item, id): (Value, String),
    uri: &str,
    query_token: Option<&str>,
    absent: &[&'static str],
) -> Vec<Page> {
    let mut continued_request = request.clone();
    if let Some(obj) = continued_request.as_object_mut() {
        obj.insert(token.to_owned(), json!("later"));
    }
    let mut first = Page::new(
        operation,
        request,
        json!({items:[],token:"later"}),
        Vec::new(),
    )
    .under(parent);
    let mut second = Page::new(
        operation,
        continued_request.clone(),
        json!({items:[item]}),
        vec![id],
    )
    .under(parent);
    first.absent = absent.to_vec();
    second.absent = absent.to_vec();
    if uri != "/" {
        let continued_uri = query_token.map_or_else(
            || uri.to_owned(),
            |name| {
                format!(
                    "{uri}{}{name}=later",
                    if uri.contains('?') { "&" } else { "?" }
                )
            },
        );
        let continued_body = if query_token.is_some() {
            None
        } else {
            Some(json!({token:"later"}))
        };
        first = first.rest(uri.to_owned(), None);
        second = second.rest(continued_uri, continued_body);
    }
    vec![first, second]
}

fn request_json(record: &Record, fields: &catalog_tool::smithy::Registry) -> TestResult<Value> {
    let entries = record
        .root()
        .iter()?
        .map(|entry| -> TestResult<(String, Value)> {
            let (key, value) = entry?;
            let name = match key {
                KeyRef::Field(id) => fields.name(id).ok_or("request field name absent")?,
                KeyRef::Name(name) => name,
            };
            match value {
                ValueRef::Text(value) => Ok((name.to_owned(), json!(value))),
                _ => Err("unexpected non-text request input".into()),
            }
        })
        .collect::<TestResult<serde_json::Map<_, _>>>()?;
    Ok(Value::Object(entries))
}

fn assert_wire(page: &Page, wire: &http::Request<bytes::Bytes>) -> TestResult {
    let expected: http::Uri = format!("https://fixture.example.test{}", page.uri).parse()?;
    assert_eq!(
        wire.uri().path(),
        expected.path(),
        "{} path",
        page.operation
    );
    let query = |uri: &http::Uri| -> BTreeSet<String> {
        uri.query().map_or_else(BTreeSet::new, |query| {
            query.split('&').map(str::to_owned).collect()
        })
    };
    assert_eq!(
        query(wire.uri()),
        query(&expected),
        "{} query",
        page.operation
    );
    if page.cbor {
        assert_eq!(
            wire.headers()
                .get("smithy-protocol")
                .and_then(|value| value.to_str().ok()),
            Some("rpc-v2-cbor"),
            "{} protocol",
            page.operation
        );
        assert_eq!(
            wire.method(),
            http::Method::POST,
            "{} method",
            page.operation
        );
        assert!(wire.uri().query().is_none(), "{} query", page.operation);
        let expected = page.body.as_ref().ok_or("CBOR page has no request body")?;
        assert_eq!(
            &crate::test_cbor::cbor_to_json(wire.body())?,
            expected,
            "{} body",
            page.operation
        );
        return Ok(());
    }
    match &page.body {
        Some(body) => assert_eq!(
            serde_json::from_slice::<Value>(wire.body())?,
            *body,
            "{} body",
            page.operation
        ),
        None => assert!(
            wire.body().is_empty() || wire.body().as_ref() == b"{}",
            "unexpected REST payload"
        ),
    }
    Ok(())
}

fn assert_rows(
    page: &Page,
    plan: &CallablePlan<'_>,
    response: &Record,
    fields: &catalog_tool::smithy::Registry,
) -> TestResult {
    let selected = record::item_records(response, plan.items())?;
    assert_eq!(
        selected.len(),
        page.ids.len(),
        "{} row count",
        page.operation
    );
    for row in selected {
        for name in &page.absent {
            assert!(
                record::path_value(
                    row.root(),
                    &[fields.id(name).ok_or("absent field unmodeled")?]
                )?
                .is_none(),
                "request keys must not be manufactured in selected rows: {name}"
            );
        }
    }
    Ok(())
}

fn parent_reference(operation: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match operation {
        "ListAnomalyDetectors" | "ListRuleGroupsNamespaces" => {
            ("workspaceId", "AWS::APS::Workspace", "IN_WORKSPACE")
        }
        "ListGatewayRoutes" | "ListRoutes" => ("meshName", "AWS::AppMesh::Mesh", "IN_MESH"),
        "ListBackupSelections" => ("BackupPlanId", "AWS::Backup::BackupPlan", "IN_BACKUP_PLAN"),
        "ListRestoreTestingSelections" => (
            "RestoreTestingPlanName",
            "AWS::Backup::RestoreTestingPlan",
            "IN_RESTORE_TESTING_PLAN",
        ),
        "ListDataSources" => (
            "knowledgeBaseId",
            "AWS::Bedrock::KnowledgeBase",
            "IN_KNOWLEDGE_BASE",
        ),
        "ListAgentAliases" => ("agentId", "AWS::Bedrock::Agent", "IN_AGENT"),
        "ListNamedQueries" | "ListPreparedStatements" => {
            ("WorkGroup", "AWS::Athena::WorkGroup", "IN_WORKGROUP")
        }
        "ListIngestionDestinations" => (
            "ingestionIdentifier",
            "AWS::AppFabric::Ingestion",
            "IN_INGESTION",
        ),
        "ListPackageGroups" => ("domain", "AWS::CodeArtifact::Domain", "IN_DOMAIN"),
        _ => return None,
    })
}
