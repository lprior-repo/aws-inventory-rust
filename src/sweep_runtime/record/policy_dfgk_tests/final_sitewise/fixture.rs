use super::cursor::Cursor;
use crate::sweep_runtime::{
    execution::{
        page::{child, consumer_index::ConsumerIndex},
        routing,
    },
    input,
    record::policy_dfgk_tests::fixture::decode,
    request,
};
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::{
        AccountId, DiscoveryContext, OperationId, Region, RegionId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::Record,
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use serde_json::Value as Json;
use std::sync::LazyLock;

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SERVICES: &[(&str, &str, &[&str])] = &[
    (
        "iotsitewise",
        "2019-12-02",
        &[
            "ListPortals",
            "ListProjects",
            "ListDashboards",
            "ListAccessPolicies",
            "ListWorkspaces",
            "ListApplications",
            "ListDatasets",
            "ListPipelines",
            "ListTasks",
            "ListTimeSeries",
            "GetAssetPropertyValue",
            "ExecuteQuery",
            "DescribeDashboard",
            "ListPipelineExecutions",
        ],
    ),
    (
        "iottwinmaker",
        "2021-11-29",
        &[
            "ListWorkspaces",
            "ListScenes",
            "ListSyncJobs",
            "GetScene",
            "GetPropertyValue",
            "ExecuteQuery",
            "ListComponentTypes",
        ],
    ),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned SiteWise consumer fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let embedded = overlay::embedded()?;
    let policies = SERVICES
        .iter()
        .map(|(service, date, operations)| {
            crate::sweep_runtime::pinned_model::install(
                &source.join("models"),
                service,
                date,
                operations,
            )?;
            let policy = embedded
                .iter()
                .find(|policy| policy.service() == *service)
                .ok_or("production service absent")?;
            let selected = operations
                .iter()
                .map(|operation| {
                    let key = overlay::OperationName::new(operation)?;
                    Ok((
                        *operation,
                        policy
                            .operations()
                            .get(&key)
                            .ok_or("production operation absent")?
                            .clone(),
                    ))
                })
                .collect::<TestResult<Vec<_>>>()?;
            Ok(overlay::ServiceOverlay::new(service, selected)?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(
        &source,
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &endpoints::verified_endpoints(&source)?,
    )?)
}

pub(super) fn plan<'a, 'c>(
    catalog: &'a RuntimeCatalog<'c>,
    service: &str,
    operation: &str,
) -> TestResult<&'a CallablePlan<'c>> {
    catalog
        .lookup(service, operation)
        .ok_or_else(|| format!("missing production plan {service}.{operation}").into())
}

pub(super) struct Scenario<'a, 'c> {
    directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub account: AccountId,
    pub home: RegionId,
    consumers: ConsumerIndex<'a, 'c>,
}

impl<'a, 'c> Scenario<'a, 'c> {
    pub async fn new(catalog: &'a RuntimeCatalog<'c>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store =
            BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
        Ok(Self {
            directory,
            store,
            sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
            consumers: ConsumerIndex::build(catalog)?,
        })
    }

    pub async fn reopen(self) -> TestResult<Self> {
        let Self {
            directory,
            store,
            sweep,
            account,
            home,
            consumers,
        } = self;
        drop(store);
        let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        Ok(Self {
            directory,
            store,
            sweep,
            account,
            home,
            consumers,
        })
    }

    pub async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
        let region = Region::Regional(self.home.clone());
        let scope = ScopeKey::new(
            self.account.clone(),
            region.clone(),
            ServiceId::parse(plan.service_name())?,
            OperationId::parse(plan.operation_name())?,
            None,
        )?;
        if plan.branches().is_empty() {
            let base = input::root_bindings(plan, Some(&self.account), Some(&region))?;
            self.store
                .seed_work(WorkSpec::new(
                    self.sweep.clone(),
                    scope,
                    None,
                    base.into_bytes(),
                )?)
                .await?;
        } else {
            for branch in plan.branches() {
                let base = routing::branch_bindings(plan, branch, &self.account, &region)?;
                self.store
                    .seed_work(WorkSpec::new(
                        self.sweep.clone(),
                        scope.clone().with_branch(branch.discriminator.clone())?,
                        None,
                        base.into_bytes(),
                    )?)
                    .await?;
            }
        }
        Ok(())
    }

    pub fn request(
        &self,
        plan: &CallablePlan<'_>,
        lease: &WorkLease,
    ) -> TestResult<(Record, http::Request<bytes::Bytes>)> {
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    pub async fn apply(
        &mut self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: Json,
    ) -> TestResult<Vec<ResourceKey>> {
        let response = decode(plan, &serde_json::to_vec(&body)?)?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let context = child::Context::new(
            plan,
            &self.sweep,
            &self.account,
            &self.home,
            &self.consumers,
        );
        let (observations, children) = child::policy_dfgk_tests::project_children(
            catalog,
            &context,
            lease.scope(),
            request,
            &response,
            &ancestry,
        )?;
        let unresolved = child::policy_dfgk_tests::projected_references(
            catalog,
            &context,
            lease.scope(),
            request,
            &response,
            &observations,
        )?;
        let keys = observations
            .iter()
            .map(|observation| observation.key().clone())
            .collect();
        let next = Cursor::decode(lease.token())?.next(
            plan.pagination(),
            &response,
            &http::HeaderMap::new(),
            10,
        )?;
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |advance| {
            (
                Some(advance.cursor.into_bytes()),
                advance.continuation_digest,
            )
        });
        self.store
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
                observed_at_ms: 1001,
            })
            .await?;
        self.store = BinaryStore::open(&self.directory.path().join("inventory.sqlite")).await?;
        Ok(keys)
    }

    pub async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("inventory.sqlite"))
                    .read_only(true),
            )
            .await?)
    }
}
