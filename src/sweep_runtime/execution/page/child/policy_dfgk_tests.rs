mod accelerators;
mod ecr_images;
mod iot_names;
mod complete_df;
mod final_integrations;

use super::*;
use super::super::{consumer_index::ConsumerIndex, cursor, rows as page_rows};
use crate::sweep_runtime::{execution::routing, record::policy_dfgk_tests::fixture, resource_projection};
use inventory_core::{catalog::runtime::{Emission, PlanScope}, inventory::UnresolvedReference};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Shared by policy scenarios: real projected observations and actual child planning.
pub(in crate::sweep_runtime) fn project_children(
    catalog: &RuntimeCatalog<'_>,
    context: &Context<'_, '_>, scope: &ScopeKey, request: &Record, response: &Record,
    ancestry: &DiscoveryContext,
) -> TestResult<(Vec<Observation>, Vec<WorkSpec>)> {
    let plan = context.parent_plan;
    let items = projected_items(plan, response)?;
    let mut budget = PageContextBudget::new(ancestry, items.len())?;
    let contexts = contexts_from(plan, Rows { items: &items, offset: 0 },
        request, response, ancestry, scope, &mut budget)?;
    let region = routing::request_region(plan, scope.region(), context.home, scope.branch())?;
    let branch = routing::execution_branch(plan, scope.branch())?;
    let observations = if plan.effective_emission(branch) == Emission::DiscoveryOnly { Vec::new() } else {
        items.iter().zip(&contexts).map(|(row, ancestry)| {
            page_rows::observation(catalog,
                record::IdentityContext { account: context.account, region: &region },
                plan, scope.region(), row, request, ancestry)
        }).collect::<Result<Vec<_>, _>>()?
    };
    let children = work(context, &items, &observations, request, &contexts, scope, &mut budget)?;
    Ok((observations, children))
}

pub(in crate::sweep_runtime) fn projected_references(
    catalog: &RuntimeCatalog<'_>,
    context: &Context<'_, '_>, scope: &ScopeKey, request: &Record, response: &Record,
    observations: &[Observation],
) -> TestResult<Vec<UnresolvedReference>> {
    let items = projected_items(context.parent_plan, response)?;
    let region = routing::request_region(context.parent_plan, scope.region(), context.home, scope.branch())?;
    Ok(page_rows::references(catalog, context.parent_plan, &region, &items, observations, request)?)
}

fn projected_items(plan: &CallablePlan<'_>, response: &Record) -> TestResult<Vec<Record>> {
    record::item_records(response, plan.items())?.iter().map(|row| {
        resource_projection::project(plan, row).map_err(Into::into)
    }).collect()
}

struct Scenario<'a, 'catalog> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

impl<'a, 'catalog> Scenario<'a, 'catalog> {
    async fn new(catalog: &'a RuntimeCatalog<'catalog>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self { directory, store, sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?, home: RegionId::parse("us-east-1")?,
            consumers: ConsumerIndex::build(catalog)? })
    }

    async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
        let region = if plan.scope() == PlanScope::Account { Region::Global } else { Region::Regional(self.home.clone()) };
        let scope = ScopeKey::new(self.account.clone(), region.clone(),
            ServiceId::parse(plan.service_name())?, OperationId::parse(plan.operation_name())?, None)?;
        let bindings = input::root_bindings(plan, Some(&self.account), Some(&region))?;
        self.store.seed_work(WorkSpec::new(self.sweep.clone(), scope, None, bindings.into_bytes())?).await?;
        Ok(())
    }

    fn request(&self, plan: &CallablePlan<'_>, lease: &WorkLease) -> TestResult<(Record, Json)> {
        let (request, wire) = self.prepared_request(plan, lease)?;
        Ok((request, serde_json::from_slice(wire.body())?))
    }

    fn prepared_request(&self, plan: &CallablePlan<'_>, lease: &WorkLease) -> TestResult<(Record, http::Request<bytes::Bytes>)> {
        let region = routing::request_region(plan, lease.scope().region(), &self.home, lease.scope().branch())?;
        let endpoint = routing::resolve(plan, &region)?;
        if plan.service_name() == "global-accelerator" {
            assert_eq!(region.as_str(), "us-west-2");
            assert_eq!(endpoint.signing_region, "us-west-2");
            assert_eq!(lease.scope().region(), &Region::Global);
        }
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let mut fields = cursor.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField { path: size.path.clone(), value: size.value() });
        }
        let wire = crate::sweep_runtime::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    async fn apply(&self, catalog: &RuntimeCatalog<'_>, plan: &CallablePlan<'_>, lease: WorkLease,
        request: &Record, body: Json) -> TestResult {
        let body = serde_json::to_vec(&body)?;
        let response = fixture::decode(plan, &body)?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let (observations, children, unresolved, tags) = match plan.purpose() {
            PlanPurpose::TagEndpoint(endpoint) => {
                let owner = lease.scope().parent().ok_or("tag work lost its canonical owner")?;
                (Vec::new(), Vec::new(), Vec::new(), vec![record::tag_observation(&response, owner, &endpoint.tags)?])
            }
            _ => {
                let ancestry = super::super::preparation::ancestry(lease.context(), lease.scope().context())?;
                let context = Context::new(plan, &self.sweep, &self.account, &self.home, &self.consumers);
                let (observations, children) = project_children(catalog, &context, lease.scope(), request, &response, &ancestry)?;
                let references = projected_references(catalog, &context, lease.scope(), request, &response, &observations)?;
                (observations, children, references, Vec::new())
            }
        };
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |advance| {
            (Some(advance.cursor.into_bytes()), advance.continuation_digest)
        });
        self.store.commit_page(PageCommit { lease, observations, unresolved, tags, children,
            edges: Vec::new(), next_token, continuation_digest, done, observed_at_ms: 1001 }).await?;
        Ok(())
    }

    async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new()
                .filename(self.directory.path().join("inventory.db")).read_only(true)).await?)
    }
}

fn text<'a>(value: &'a Json, name: &str) -> TestResult<&'a str> {
    value.get(name).and_then(Json::as_str).ok_or_else(|| format!("encoded request omitted {name}").into())
}
