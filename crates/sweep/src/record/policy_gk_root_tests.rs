mod cases;
pub(in crate::record) mod cbor;
mod fixture;
use crate::execution::page::cursor;

use crate::{
    execution::{
        page::{child, consumer_index::ConsumerIndex},
        routing,
    },
    input, record, request,
};
use cursor::Cursor;
use inventory_core::{
    catalog::runtime::{PlanScope, RuntimeCatalog},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::Record,
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn verify_wire(case: &cases::Case, page: usize, wire: &http::Request<bytes::Bytes>) -> TestResult {
    let token = match page {
        0 => None,
        1 => Some("second"),
        2 => Some("third"),
        _ => return Err("unexpected page".into()),
    };
    let mut expected = case.fixed.clone();
    if let (Some(field), Some(value), Some(fields)) = (case.token, token, expected.as_object_mut())
    {
        fields.insert(field.to_owned(), json!(value));
    }
    if case.service == "glacier" {
        assert_eq!(wire.method(), http::Method::GET);
        assert_eq!(wire.uri().path(), "/-/provisioned-capacity");
        assert_eq!(wire.uri().query(), None);
        assert_eq!(wire.body().len(), 0);
    } else if case.service == "gamelift" {
        assert_eq!(
            cbor::request(wire, "GameLift", case.operation)?,
            expected,
            "{} page {page}",
            case.operation
        );
    } else {
        assert_eq!(wire.method(), http::Method::POST);
        assert_eq!(wire.uri().query(), None);
        let (path, target) = match case.service {
            "imagebuilder" => (format!("/{}", case.operation), None),
            "inspector2" => (
                (match case.operation {
                    "ListConnectors" => "/connector/list",
                    "ListConnectorScanConfigurations" => "/connectorscanconfigurations/list",
                    _ => return Err("unreviewed Inspector HTTP path".into()),
                })
                .to_owned(),
                None,
            ),
            "glue" => ("/".to_owned(), Some(format!("AWSGlue.{}", case.operation))),
            "invoicing" => (
                "/".to_owned(),
                Some(format!("Invoicing.{}", case.operation)),
            ),
            _ => return Err("unreviewed root HTTP service".into()),
        };
        assert_eq!(wire.uri().path(), path);
        assert_eq!(
            wire.headers()
                .get("x-amz-target")
                .map(http::HeaderValue::to_str)
                .transpose()?,
            target.as_deref()
        );
        assert_eq!(
            serde_json::from_slice::<Json>(wire.body())?,
            expected,
            "{} page {page}",
            case.operation
        );
    }
    Ok(())
}

fn response(case: &cases::Case, page: usize) -> TestResult<Json> {
    let (items, token) = if case.token.is_none() {
        (case.rows.to_vec(), None)
    } else {
        match page {
            0 => (Vec::new(), Some("second")),
            1 => (
                vec![case.rows.first().ok_or("first row absent")?.clone()],
                Some("third"),
            ),
            2 => (
                vec![case.rows.last().ok_or("last row absent")?.clone()],
                None,
            ),
            _ => return Err("unexpected response page".into()),
        }
    };
    let mut body = json!({case.items:items});
    if let (Some(name), Some(token), Some(fields)) = (case.token, token, body.as_object_mut()) {
        fields.insert(name.to_owned(), json!(token));
    }
    Ok(body)
}

async fn seed(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    cases: &[cases::Case],
    sweep: &SweepId,
    account: &AccountId,
    home: &RegionId,
) -> TestResult {
    for case in cases {
        let plan = catalog
            .lookup(case.service, case.operation)
            .ok_or("root plan absent")?;
        assert_eq!(plan.kind().as_str(), case.kind);
        let region = if plan.scope() == PlanScope::Account {
            Region::Global
        } else {
            Region::Regional(home.clone())
        };
        let scope = ScopeKey::new(
            account.clone(),
            region.clone(),
            ServiceId::parse(case.service)?,
            OperationId::parse(case.operation)?,
            None,
        )?;
        let base = input::root_bindings(plan, Some(account), Some(&region))?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                base.into_bytes(),
            )?)
            .await?;
    }
    Ok(())
}

async fn consume(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    consumers: &ConsumerIndex<'_, '_>,
    case: &cases::Case,
    (sweep, account, home): (&SweepId, &AccountId, &RegionId),
    page: usize,
) -> TestResult<bool> {
    let at = i64::try_from(page)?;
    let Some(lease) = store.claim(sweep, at, 10_000).await? else {
        return Ok(false);
    };
    assert_eq!(lease.scope().service().as_str(), case.service);
    assert_eq!(lease.scope().operation().as_str(), case.operation);
    let plan = catalog
        .lookup(case.service, case.operation)
        .ok_or("leased plan absent")?;
    let base = Record::from_bytes(lease.bindings().to_vec())?;
    let cursor = Cursor::decode(lease.token())?;
    let fields = cursor.request_fields(plan.pagination())?;
    let region =
        routing::request_region(plan, lease.scope().region(), home, lease.scope().branch())?;
    let endpoint = routing::resolve(plan, &region)?;
    let wire = request::encode(plan, &endpoint.url, &base, &fields)?;
    verify_wire(case, page, &wire)?;
    let actual = input::merge_input(&base, &fields)?;
    let body = cbor::response(plan, &response(case, page)?)?;
    let response = record::policy_dfgk_tests::fixture::decode(plan, &body)?;
    let context = child::Context::new(plan, sweep, account, home, consumers);
    let (observations, children) = child::policy_dfgk_tests::project_children(
        catalog,
        &context,
        lease.scope(),
        &actual,
        &response,
        &DiscoveryContext::empty(),
    )?;
    assert_eq!(
        children.len(),
        0,
        "{} must not schedule detail, paid or credential calls",
        case.operation
    );
    assert_eq!(
        observations.len(),
        if case.token.is_none() {
            2
        } else if page == 0 {
            0
        } else {
            1
        }
    );
    for observation in &observations {
        assert_eq!(observation.key().kind.as_str(), case.kind);
        assert_eq!(&observation.key().account, account);
        assert!(
            case.ids
                .iter()
                .any(|id| id == observation.key().id.as_str())
        );
        assert_eq!(
            observation.key().region,
            if plan.scope() == PlanScope::Account {
                Region::Global
            } else {
                Region::Regional(home.clone())
            }
        );
    }
    let unresolved = child::policy_dfgk_tests::projected_references(
        catalog,
        &context,
        lease.scope(),
        &actual,
        &response,
        &observations,
    )?;
    let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
    let done = next.is_none();
    assert_eq!(
        done,
        case.token.is_none() || page == 2,
        "empty pages must retain continuation"
    );
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
    Ok(true)
}

#[tokio::test]
async fn given_unattached_safe_roots_when_empty_pages_resume_after_sqlite_reopen_then_exact_resources_and_connector_edges_survive_without_forbidden_calls()
-> TestResult {
    // Given the real pinned models, production overlays/kind map, and no parent resources.
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let cases = cases::all();
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("gk-roots.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    // A separate deterministic sweep per producer avoids assuming queue priority.
    for case in &cases {
        let sweep = SweepId::new(format!("roots-{}-{}", case.service, case.operation))?;
        seed(
            &store,
            &catalog,
            std::slice::from_ref(case),
            &sweep,
            &account,
            &home,
        )
        .await?;
        for page in 0..if case.token.is_some() { 3 } else { 1 } {
            assert!(
                consume(
                    &store,
                    &catalog,
                    &consumers,
                    case,
                    (&sweep, &account, &home),
                    page,
                )
                .await?
            );
            drop(store);
            store = BinaryStore::open(&path).await?;
        }
        assert!(
            store.claim(&sweep, 4, 10_000).await?.is_none(),
            "{} scheduled an extra call",
            case.operation
        );
    }
    // Then actual queryable SQLite resources preserve both names/builds and all later-page IDs.
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(resources.len(), 36);
    let actual = resources
        .iter()
        .map(|resource| {
            (
                resource.key.kind.as_str().to_owned(),
                resource.key.id.as_str().to_owned(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = cases
        .iter()
        .flat_map(|case| case.ids.iter().map(|id| (case.kind.to_owned(), id.clone())))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 2);
    for edge in edges.edges {
        assert_eq!(edge.from.kind.as_str(), "AWS::InspectorV2::Connector");
        assert_eq!(
            edge.to.kind.as_str(),
            "AWS::ApiOnly::Inspector2ConnectorScanConfiguration"
        );
        assert_eq!(edge.relation.as_str(), "USES_SCAN_CONFIGURATION");
        let ordinal = if edge.from.id.as_str().ends_with("0001") {
            "first"
        } else {
            "later"
        };
        assert!(edge.to.id.as_str().ends_with(ordinal));
    }
    Ok(())
}

#[test]
fn given_safe_metadata_promotions_when_compiling_embedded_plans_then_content_secrets_paid_mutations_and_provider_reference_calls_stay_absent()
-> TestResult {
    let catalog = fixture::catalog()?;
    let approved = cases::all()
        .into_iter()
        .map(|case| (case.service, case.operation))
        .collect::<BTreeSet<_>>();
    for (service, _, operations) in fixture::SERVICES {
        for operation in *operations {
            if !approved.contains(&(*service, *operation)) {
                assert!(
                    catalog.lookup(service, operation).is_none(),
                    "forbidden {service}.{operation} became callable"
                );
            }
        }
    }
    assert!(
        catalog
            .lookup("imagebuilder", "ListImageBuildVersions")
            .is_none(),
        "failed component-step output is content, even when only ARNs are projected"
    );
    Ok(())
}

#[test]
fn given_trusted_root_scope_when_an_arn_is_malformed_or_foreign_then_no_observation_or_child_is_admitted()
-> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("gk-root-arn-boundary".to_owned())?;
    for case in cases::all() {
        let identity = match (case.service, case.operation) {
            ("gamelift", "ListScripts") => "ScriptArn",
            ("gamelift", "ListLocations") => "LocationArn",
            ("imagebuilder", _) => "arn",
            ("inspector2", "ListConnectors") => "connectorArn",
            ("inspector2", "ListConnectorScanConfigurations") => "awsConfigConnectorArn",
            ("invoicing", _) => "ProcurementPortalPreferenceArn",
            _ => continue,
        };
        let plan = catalog
            .lookup(case.service, case.operation)
            .ok_or("ARN root plan absent")?;
        let region = if plan.scope() == PlanScope::Account {
            Region::Global
        } else {
            Region::Regional(home.clone())
        };
        let scope = ScopeKey::new(
            account.clone(),
            region.clone(),
            ServiceId::parse(case.service)?,
            OperationId::parse(case.operation)?,
            None,
        )?;
        let request = input::root_bindings(plan, Some(&account), Some(&region))?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let row = case.rows.first().ok_or("ARN fixture row absent")?;
        let valid = cbor::response(plan, &json!({case.items:[row]}))?;
        let valid = record::policy_dfgk_tests::fixture::decode(plan, &valid)?;
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            &scope,
            &request,
            &valid,
            &DiscoveryContext::empty(),
        )?;
        assert_eq!(observations.len(), 1);
        assert!(children.is_empty());
        let mut parts = case
            .ids
            .first()
            .ok_or("ARN fixture identity absent")?
            .splitn(6, ':');
        let (
            Some("arn"),
            Some(partition),
            Some(service),
            Some(arn_region),
            Some(arn_account),
            Some(resource),
        ) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        )
        else {
            return Err("fixture identity is not an ARN".into());
        };
        for forged in [
            "not-an-arn".to_owned(),
            format!("arn:{partition}:{service}:{arn_region}:999999999999:{resource}"),
            format!("arn:aws-cn:{service}:{arn_region}:{arn_account}:{resource}"),
            format!("arn:{partition}:{service}:us-west-2:{arn_account}:{resource}"),
        ] {
            let mut forged_row = row.clone();
            forged_row
                .as_object_mut()
                .ok_or("row is not an object")?
                .insert(identity.to_owned(), json!(forged));
            let response = cbor::response(plan, &json!({case.items:[forged_row]}))?;
            let response = record::policy_dfgk_tests::fixture::decode(plan, &response)?;
            let error = child::policy_dfgk_tests::project_children(
                &catalog,
                &context,
                &scope,
                &request,
                &response,
                &DiscoveryContext::empty(),
            )
            .err()
            .ok_or("foreign ARN was admitted")?;
            assert!(
                error.to_string().contains("modeled ARN"),
                "{}: {error}",
                case.operation
            );
        }
    }
    Ok(())
}
