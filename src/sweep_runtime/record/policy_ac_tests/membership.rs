mod cleanrooms;
mod cognito;
mod connect;
mod domains;

use super::{TestResult, compile, cursor::Cursor, decode, resource_key};
use crate::sweep_runtime::{
    execution::page::{child, consumer_index::ConsumerIndex, rows},
    input, record, request,
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{
        AccountId, DiscoveryContext, OperationId, Region, RegionId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::{Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};

const OWNERS: [&str; 2] = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
];

struct Page {
    body: Json,
    id: String,
    wire: NativeRequest,
}

struct Owner {
    value: String,
    pages: [Page; 2],
}

struct Case {
    operation: &'static str,
    kind: &'static str,
    parent_field: &'static str,
    requires_request_identity: bool,
    owners: [Owner; 2],
}

struct NativeRequest {
    method: &'static str,
    path: String,
    query: BTreeSet<String>,
    body: Json,
}

impl NativeRequest {
    fn get(path: String, query: &[&str]) -> Self {
        Self {
            method: "GET",
            path,
            query: query.iter().map(|value| (*value).to_owned()).collect(),
            body: Json::Null,
        }
    }

    fn post(path: String, body: Json) -> Self {
        Self {
            method: "POST",
            path,
            query: BTreeSet::new(),
            body,
        }
    }

    fn assert_matches(&self, wire: &http::Request<bytes::Bytes>) -> TestResult {
        assert_eq!(wire.method().as_str(), self.method);
        assert_eq!(wire.uri().path(), self.path);
        let query = wire.uri().query().map_or_else(BTreeSet::new, |query| {
            query.split('&').map(str::to_owned).collect()
        });
        assert_eq!(query, self.query);
        let body = if wire.body().is_empty() {
            Json::Null
        } else {
            serde_json::from_slice(wire.body())?
        };
        assert_eq!(body, self.body);
        Ok(())
    }
}

fn text(
    record: &Record,
    fields: &catalog_tool::smithy::Registry,
    field: &str,
) -> TestResult<String> {
    match record::path_value(
        record.root(),
        &[fields.id(field).ok_or("modeled field absent")?],
    )? {
        Some(ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err(format!("required text absent: {field}").into()),
    }
}

fn response(items: &str, row: Json, token: &str, later: bool) -> Json {
    if later {
        json!({items:[row]})
    } else {
        json!({items:[row],token:"later"})
    }
}

async fn run(
    model: (&str, &str),
    root: &str,
    root_owner_field: &str,
    root_body: Json,
    operations: &[&str],
    excluded: &[&str],
    cases: Vec<Case>,
) -> TestResult {
    let (service, date) = model;
    // Given the actual pinned model, embedded policy, mappings, and two independently selected parents.
    let compiled = if service == "cleanrooms" {
        let mut operations = operations.to_vec();
        operations.push("ListConfiguredTables");
        super::fixture::compile_many(&[
            (service, date, &operations),
            (
                "cleanroomsml",
                "2023-09-06",
                &["ListConfiguredAudienceModels"],
            ),
            ("entityresolution", "2018-05-10", &["ListIdNamespaces"]),
        ])?
    } else {
        compile(service, date, operations)?
    };
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    for operation in excluded {
        assert!(
            catalog.lookup(service, operation).is_none(),
            "excluded operation became callable: {operation}"
        );
    }
    for case in &cases {
        assert_eq!(
            catalog
                .lookup(service, case.operation)
                .ok_or("child plan absent")?
                .kind()
                .as_str(),
            case.kind
        );
    }
    let consumers = ConsumerIndex::build(&catalog)?;
    let parent = catalog.lookup(service, root).ok_or("parent plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new(format!("{service}-owned-metadata"))?;
    let scope = ScopeKey::new(
        account.clone(),
        region.clone(),
        ServiceId::parse(service)?,
        OperationId::parse(root)?,
        None,
    )?;
    let root_request = input::root_bindings(parent, Some(&account), Some(&region))?;
    let root_response = decode(parent, &root_body)?;
    let context = child::Context::new(parent, &sweep, &account, &home, &consumers);
    let (parents, children) = child::policy_dfgk_tests::project_children(
        &catalog,
        &context,
        &scope,
        &root_request,
        &root_response,
        &DiscoveryContext::empty(),
    )?;
    assert_eq!(parents.len(), 2);
    assert_eq!(
        children.len(),
        cases.len().checked_mul(2).ok_or("child count overflow")?
    );
    let (items, parent_kind, identity) = match service {
        "cleanrooms" => ("membershipSummaries", "AWS::CleanRooms::Membership", "arn"),
        "cognito-identity-provider" => ("UserPools", "AWS::Cognito::UserPool", "Id"),
        "connect" => ("InstanceSummaryList", "AWS::Connect::Instance", "Arn"),
        "connectcases" => ("domains", "AWS::Cases::Domain", "domainArn"),
        "connecthealth" => ("domains", "AWS::ApiOnly::ConnectHealthDomain", "arn"),
        _ => return Err("unreviewed membership root identity".into()),
    };
    let owners = root_body
        .get(items)
        .and_then(Json::as_array)
        .ok_or("native parent rows absent")?
        .iter()
        .map(|row| {
            let owner = row
                .get(root_owner_field)
                .and_then(Json::as_str)
                .ok_or("native owner selector absent")?;
            let id = row
                .get(identity)
                .and_then(Json::as_str)
                .ok_or("native parent identity absent")?;
            Ok((
                owner.to_owned(),
                resource_key(&account, &region, parent_kind, id)?,
            ))
        })
        .collect::<TestResult<BTreeMap<String, ResourceKey>>>()?;
    assert_eq!(owners.len(), 2);
    assert_eq!(
        parents
            .iter()
            .map(|row| row.key().clone())
            .collect::<BTreeSet<_>>(),
        owners.values().cloned().collect::<BTreeSet<_>>()
    );
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("owned-metadata.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            root_request.as_bytes().to_vec(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, 0, 10_000)
        .await?
        .ok_or("parent lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations: parents,
            children,
            unresolved: Vec::new(),
            edges: Vec::new(),
            tags: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: 0,
        })
        .await?;
    let expected = cases
        .iter()
        .flat_map(|case| {
            case.owners.iter().flat_map(move |owner| {
                owner
                    .pages
                    .iter()
                    .map(move |page| (case.kind.to_owned(), page.id.clone()))
            })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected.len(),
        cases.len().checked_mul(4).ok_or("page count overflow")?
    );
    let mut expected_resources = owners.values().cloned().collect::<BTreeSet<_>>();
    for (kind, id) in &expected {
        expected_resources.insert(resource_key(&account, &region, kind, id)?);
    }
    let parent_relation = match service {
        "cleanrooms" => "IN_MEMBERSHIP",
        "cognito-identity-provider" => "IN_USER_POOL",
        "connect" => "IN_INSTANCE",
        "connectcases" | "connecthealth" => "IN_DOMAIN",
        _ => return Err("unreviewed membership parent relation".into()),
    };
    let mut expected_edges = BTreeSet::new();
    for case in &cases {
        for owner in &case.owners {
            let target = owners
                .get(&owner.value)
                .ok_or("fixture owner was not selected")?;
            for page in &owner.pages {
                expected_edges.insert((
                    case.kind.to_owned(),
                    page.id.clone(),
                    target.kind.to_string(),
                    target.id.to_string(),
                    parent_relation.to_owned(),
                ));
            }
        }
    }
    let mut seen = BTreeSet::new();
    // When real child projection, durable work leases, cursor resumes and native serializers consume both pages.
    for at in 1..=expected.len() {
        let at = i64::try_from(at)?;
        let lease = store
            .claim(&sweep, at, 10_000)
            .await?
            .ok_or("expected child page absent")?;
        let plan = catalog
            .lookup(service, lease.scope().operation().as_str())
            .ok_or("leased child absent")?;
        let case = cases
            .iter()
            .find(|case| case.operation == plan.operation_name())
            .ok_or("unreviewed child scheduled")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let owner_value = text(&base, &compiled.fields, case.parent_field)?;
        let owner = case
            .owners
            .iter()
            .find(|owner| owner.value == owner_value)
            .ok_or("child lost selected parent")?;
        assert_eq!(lease.scope().parent(), owners.get(&owner_value));
        let later = lease.token().is_some();
        let page = owner
            .pages
            .get(usize::from(later))
            .ok_or("page fixture absent")?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual_request = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        if service == "cognito-identity-provider" {
            let target = format!("AWSCognitoIdentityProviderService.{}", case.operation);
            assert_eq!(
                wire.headers()
                    .get("x-amz-target")
                    .and_then(|value| value.to_str().ok()),
                Some(target.as_str())
            );
        }
        page.wire.assert_matches(&wire)?;
        let response = decode(plan, &page.body)?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            lease.scope(),
            &actual_request,
            &response,
            &ancestry,
        )?;
        assert_eq!(observations.len(), 1);
        assert_eq!(
            children.len(),
            0,
            "summary must not schedule content or secret details"
        );
        let observation = observations.first().ok_or("summary observation absent")?;
        assert_eq!(observation.key().kind.as_str(), case.kind);
        assert_eq!(observation.key().id.as_str(), page.id);
        assert!(
            seen.insert((case.kind.to_owned(), page.id.clone())),
            "duplicate or parent-colliding identity"
        );
        let actual_branch = crate::sweep_runtime::execution::routing::execution_branch(
            plan,
            lease.scope().branch(),
        )?;
        let mut selected = Vec::new();
        for row in record::item_records(&response, plan.items())? {
            if crate::sweep_runtime::row_policy::selected(
                plan,
                actual_branch,
                &row,
                &actual_request,
                &response,
            )? {
                selected.push(row);
            }
        }
        if case.requires_request_identity {
            let row = selected.first().ok_or("selected summary absent")?;
            assert!(
                record::path_value(
                    row.root(),
                    &[compiled
                        .fields
                        .id(case.parent_field)
                        .ok_or("request field absent")?]
                )?
                .is_none()
            );
            assert!(
                record::resource_id_with_context(
                    row,
                    plan.identity(),
                    None,
                    &ancestry,
                    &root_request
                )
                .is_err(),
                "scoped identity accepted an unrelated root request"
            );
        }
        let unresolved = rows::references(
            &catalog,
            plan,
            &home,
            &selected,
            &observations,
            &actual_request,
            None,
        )?;
        let expected_reference_count = if matches!(
            case.operation,
            "ListConfiguredTableAssociations"
                | "ListConfiguredAudienceModelAssociations"
                | "ListIdNamespaceAssociations"
        ) {
            2
        } else {
            1
        };
        assert_eq!(
            unresolved.len(),
            expected_reference_count,
            "every reviewed endpoint must retain its join"
        );
        let parent_references = unresolved
            .iter()
            .filter(|reference| reference.target.kind() == parent.kind())
            .collect::<Vec<_>>();
        assert_eq!(parent_references.len(), 1);
        let reference = parent_references
            .first()
            .ok_or("reviewed parent reference absent")?;
        assert_eq!(&reference.from, observation.key());
        assert_eq!(
            reference.source,
            inventory_core::inventory::EdgeSource::ReviewedRule
        );
        assert_eq!(reference.target.kind(), parent.kind());
        assert_eq!(reference.target.value().as_str(), owner_value);
        assert_eq!(reference.target.account(), &account);
        assert_eq!(reference.target.region(), &region);
        assert_eq!(reference.relation.as_str(), parent_relation);
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        assert_eq!(
            next.is_none(),
            later,
            "first-page cursor or terminal completion lost"
        );
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| {
            (Some(next.cursor.into_bytes()), next.continuation_digest)
        });
        store
            .commit_page(PageCommit {
                lease,
                observations,
                children,
                unresolved,
                edges: Vec::new(),
                tags: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: at,
            })
            .await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then every exact canonical/API-only row survives SQLite reopen, and no forbidden work remains.
    assert_eq!(seen, expected);
    assert!(store.claim(&sweep, 1000, 10_000).await?.is_none());
    let resources = store.projection_resources(None, 1000).await?;
    assert_eq!(resources.len(), expected_resources.len());
    let persisted = resources
        .into_iter()
        .map(|resource| resource.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(persisted, expected_resources);
    let edges = store.projection_edges(None, 1000).await?;
    assert_eq!(edges.edges.len(), expected_edges.len());
    let persisted_edges = edges
        .edges
        .into_iter()
        .map(|edge| {
            assert_eq!(edge.source, "reviewed-rule");
            assert_eq!(edge.from.account, account);
            assert_eq!(edge.to.account, account);
            assert_eq!(edge.from.region, region);
            assert_eq!(edge.to.region, region);
            (
                edge.from.kind.to_string(),
                edge.from.id.to_string(),
                edge.to.kind.to_string(),
                edge.to.id.to_string(),
                edge.relation.to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(persisted_edges, expected_edges);
    drop(store);
    directory.close()?;
    Ok(())
}
