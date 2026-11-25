mod cases;
pub(in crate::record) mod cbor;
mod fixture;
mod scenarios;
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
            observed_at_ms: 0,
        })
        .await?;
    Ok(true)
}
