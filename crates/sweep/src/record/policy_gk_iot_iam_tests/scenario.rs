use super::*;
use crate::execution::page::{
    child::{
        Context,
        policy_dfgk_tests::{project_children, projected_references},
    },
    consumer_index::ConsumerIndex,
};
use inventory_core::inventory::{DiscoveryContext, OperationId, ScopeKey, ServiceId};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};

pub(super) struct Scenario {
    pub directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub account: AccountId,
    pub home: RegionId,
}

impl Scenario {
    pub async fn new(catalog: &RuntimeCatalog<'_>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self {
            directory,
            store,
            sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
        })
    }

    pub async fn reopen(self) -> TestResult<Self> {
        let Self {
            directory,
            store,
            sweep,
            account,
            home,
        } = self;
        drop(store);
        let store = BinaryStore::open(&directory.path().join("inventory.db")).await?;
        Ok(Self {
            directory,
            store,
            sweep,
            account,
            home,
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
        let input = input::root_bindings(plan, Some(&self.account), Some(&region))?;
        self.store
            .seed_work(WorkSpec::new(
                self.sweep.clone(),
                scope,
                None,
                input.into_bytes(),
            )?)
            .await?;
        Ok(())
    }

    pub async fn claim(&self, operation: &str) -> TestResult<WorkLease> {
        let lease = self
            .store
            .claim(&self.sweep, 1000, 10000)
            .await?
            .ok_or("expected durable work absent")?;
        assert_eq!(
            lease.scope().operation().as_str(),
            operation,
            "unexpected operation scheduled"
        );
        Ok(lease)
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
        let cursor = cursor::Cursor::decode(lease.token())?;
        let mut fields = cursor.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField {
                path: size.path.clone(),
                value: Value::Int(i64::from(size.max)),
            });
        }
        let wire = crate::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    pub async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: &[u8],
    ) -> TestResult {
        let response = fixture::decode(plan, body)?;
        let branch = routing::execution_branch(plan, lease.scope().branch())?;
        for row in item_records(&response, plan.items())? {
            if !crate::row_policy::selected(plan, branch, &row, request, &response)? {
                return Err("row policy filtered a row from a filter-free IoT/IAM plan".into());
            }
        }
        let next = cursor::Cursor::decode(lease.token())?.next(
            plan.pagination(),
            &response,
            &http::HeaderMap::new(),
            10,
        )?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let consumers = ConsumerIndex::build(catalog)?;
        let context = Context::new(plan, &self.sweep, &self.account, &self.home, &consumers);
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
                unresolved,
                children,
                tags: Vec::new(),
                edges: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: 1001,
            })
            .await?;
        Ok(())
    }

    pub async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("inventory.db"))
                    .read_only(true),
            )
            .await?)
    }

    pub async fn assert_nodes(
        &self,
        kind: &str,
        expected: usize,
    ) -> TestResult<Vec<(String, String, String)>> {
        let reader = self.reader().await?;
        let nodes: Vec<(String, String, String)> =
            sqlx::query_as("SELECT account,region,id FROM resources WHERE kind=? ORDER BY id")
                .bind(kind)
                .fetch_all(&reader)
                .await?;
        assert_eq!(nodes.len(), expected, "wrong durable node count for {kind}");
        for (account, region, id) in &nodes {
            assert_eq!(account, "123456789012");
            let key = inventory_core::inventory::ResourceKey {
                account: AccountId::parse(account)?,
                region: Region::parse(region)?,
                kind: inventory_core::inventory::Kind::parse(kind)?,
                id: inventory_core::inventory::ResourceId::parse(id)?,
            };
            assert!(
                self.store.projection_resource(&key).await?.is_some(),
                "graph projection resource missing"
            );
        }
        reader.close().await;
        Ok(nodes)
    }
}

pub(super) fn xml(operation: &str, result: &str) -> Vec<u8> {
    format!("<{operation}Response xmlns=\"https://iam.amazonaws.com/doc/2010-05-08/\"><{operation}Result>{result}</{operation}Result></{operation}Response>").into_bytes()
}

pub(super) fn request_contains(wire: &http::Request<bytes::Bytes>, expected: &str) -> TestResult {
    let body = std::str::from_utf8(wire.body())?;
    assert!(
        wire.uri().to_string().contains(expected) || body.contains(expected),
        "request did not contain {expected}: {body}"
    );
    Ok(())
}
