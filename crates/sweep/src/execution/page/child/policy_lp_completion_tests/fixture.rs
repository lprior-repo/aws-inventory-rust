use crate::{
    execution::{
        page::{
            child::{
                Context,
                policy_dfgk_tests::{project_children, projected_references},
            },
            consumer_index::ConsumerIndex,
            cursor,
        },
        routing,
    },
    input, record, request,
};
use aws_wire::{
    request::Protocol,
    response::{DecodeLimits, ResponseContext},
};
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanScope, RuntimeCatalog, RuntimeProtocol},
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::Record,
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use serde_json::Value as Json;
use std::{collections::BTreeSet, error::Error};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(super) type Edge = (String, String, String);

pub(super) fn compiled(
    service: &str,
    version: &str,
    operations: &[&str],
    first_kind: &str,
) -> TestResult<Vec<u8>> {
    record::policy_lp_tests::fixture::compiled(service, version, operations, first_kind)
}

pub(super) fn plan<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
    service: &str,
    operation: &str,
) -> TestResult<&'a CallablePlan<'catalog>> {
    catalog
        .lookup(service, operation)
        .ok_or_else(|| format!("compiled {service}.{operation} absent").into())
}

pub(super) struct Scenario<'a, 'catalog> {
    pub directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub account: AccountId,
    pub home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

impl<'a, 'catalog> Scenario<'a, 'catalog> {
    pub async fn new(catalog: &'a RuntimeCatalog<'catalog>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self {
            directory,
            store,
            sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
            consumers: ConsumerIndex::build(catalog)?,
        })
    }

    pub async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
        let region = if plan.scope() == PlanScope::Account {
            Region::Global
        } else {
            Region::Regional(self.home.clone())
        };
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

    pub async fn claim(&self) -> TestResult<WorkLease> {
        self.store
            .claim(&self.sweep, 1001, 10_000)
            .await?
            .ok_or_else(|| "expected page work absent".into())
    }

    pub fn request(
        &self,
        plan: &CallablePlan<'_>,
        lease: &WorkLease,
    ) -> TestResult<(Record, http::Request<bytes::Bytes>)> {
        let region = routing::request_region(
            plan,
            lease.scope().region(),
            &self.home,
            lease.scope().branch(),
        )?;
        let endpoint = routing::resolve(plan, &region)?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let mut fields =
            cursor::Cursor::decode(lease.token())?.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField {
                path: size.path.clone(),
                value: inventory_core::record::Value::Int(i64::from(size.max)),
            });
        }
        let wire = request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    pub async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: Json,
    ) -> TestResult {
        let bytes = serde_json::to_vec(&body)?;
        let headers = http::HeaderMap::new();
        let protocol = match plan.protocol() {
            RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
            RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
            RuntimeProtocol::RestJson1 => Protocol::RestJson1,
            _ => return Err("unexpected LP fixture protocol".into()),
        };
        let response = aws_wire::response::decode(ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol,
            body: &bytes,
            headers: &headers,
            status: http::StatusCode::OK,
            limits: DecodeLimits::default(),
        })?;
        let next = cursor::Cursor::decode(lease.token())?.next(
            plan.pagination(),
            &response,
            &headers,
            10,
        )?;
        let ancestry = if lease.context().is_empty() {
            DiscoveryContext::empty()
        } else {
            DiscoveryContext::decode(lease.context())?
        };
        let context = Context::new(
            plan,
            &self.sweep,
            &self.account,
            &self.home,
            &self.consumers,
        );
        let (observations, children) = project_children(
            catalog,
            &context,
            lease.scope(),
            request,
            &response,
            &ancestry,
        )?;
        let unresolved = projected_references(
            catalog,
            &context,
            lease.scope(),
            request,
            &response,
            &observations,
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
                tags: vec![],
                edges: vec![],
                next_token,
                continuation_digest,
                done,
                observed_at_ms: 1001,
            })
            .await?;
        Ok(())
    }

    pub async fn stored(&self, kind: &str) -> TestResult<(BTreeSet<String>, BTreeSet<Edge>)> {
        assert!(
            self.store.claim(&self.sweep, 1001, 10_000).await?.is_none(),
            "unexpected unconsumed work"
        );
        let reader = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("inventory.db"))
                    .read_only(true),
            )
            .await?;
        let nodes: Vec<String> =
            sqlx::query_scalar("SELECT id FROM resources WHERE kind=? ORDER BY id")
                .bind(kind)
                .fetch_all(&reader)
                .await?;
        let edges: Vec<Edge> = sqlx::query_as("SELECT from_id,to_id,relation FROM edges WHERE from_kind=? ORDER BY from_id,to_id,relation").bind(kind).fetch_all(&reader).await?;
        reader.close().await;
        Ok((nodes.into_iter().collect(), edges.into_iter().collect()))
    }
}

pub(super) fn page(mut response: Json, token: &str, second: bool) -> TestResult<Json> {
    if !second {
        response
            .as_object_mut()
            .ok_or("native response is not object")?
            .insert(token.to_owned(), Json::String("next-page".to_owned()));
    }
    Ok(response)
}

pub(super) fn native_input(wire: &http::Request<bytes::Bytes>) -> TestResult<String> {
    Ok(format!(
        "{} {}",
        wire.uri(),
        std::str::from_utf8(wire.body())?
    ))
}

pub(super) fn assert_cursor(wire: &http::Request<bytes::Bytes>, second: bool) -> TestResult {
    assert_eq!(native_input(wire)?.contains("next-page"), second);
    Ok(())
}

pub(super) fn text(wire: &Json, field: &str) -> TestResult<String> {
    wire.get(field)
        .and_then(Json::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("native request field {field} absent").into())
}

pub(super) fn bound(plan: &CallablePlan<'_>, request: &Record) -> TestResult<String> {
    let binding = plan
        .bindings()
        .iter()
        .find(|binding| {
            matches!(
                binding.source,
                inventory_core::catalog::runtime::BindingSource::Operation { .. }
            )
        })
        .ok_or("parent binding absent")?;
    match record::path_value(request.root(), &binding.path)? {
        Some(inventory_core::record::ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err("bound parent request is not text".into()),
    }
}

pub(super) fn assert_parent(wire: &http::Request<bytes::Bytes>, parent: &str) -> TestResult {
    let text = native_input(wire)?
        .replace("%3A", ":")
        .replace("%3a", ":")
        .replace("%2F", "/")
        .replace("%2f", "/");
    assert!(
        text.contains(parent),
        "discovered parent absent from native request: {parent}"
    );
    Ok(())
}

impl<'a, 'catalog> Scenario<'a, 'catalog> {
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
        let store = BinaryStore::open(&directory.path().join("inventory.db")).await?;
        Ok(Self {
            directory,
            store,
            sweep,
            account,
            home,
            consumers,
        })
    }
}
