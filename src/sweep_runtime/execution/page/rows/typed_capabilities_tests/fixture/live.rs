use super::*;
use crate::sweep_runtime::{
    call,
    execution::{
        draining, now_ms,
        page::{PageRequest, assembly, consumer_index::ConsumerIndex, cursor, preparation},
        routing,
    },
    input,
};
use inventory_engine::{
    inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease},
    limiter::{ApiExecutor, ExecutorConfig},
};

pub(super) struct Scenario {
    directory: tempfile::TempDir,
    pub catalog: RuntimeCatalog<'static>,
    sweep: SweepId,
    service: String,
    account: AccountId,
    home: RegionId,
}

pub(super) struct NativePage<'a, 'catalog> {
    pub plan: &'a CallablePlan<'catalog>,
    pub lease: &'a WorkLease,
    pub request: &'a Record,
    pub wire: &'a http::Request<bytes::Bytes>,
}

impl NativePage<'_, '_> {
    pub fn text(&self, name: &str) -> TestResult<Option<String>> {
        let id = smithy::Registry::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/catalog/field-ids.txt"
        )))?
        .id(name)
        .ok_or("request field absent from registry")?;
        match record::path_value(self.request.root(), &[id])? {
            None => Ok(None),
            Some(ValueRef::Text(value)) => Ok(Some(value.to_owned())),
            _ => Err("native request selector is not text".into()),
        }
    }
}

impl NativePage<'_, '_> {
    pub fn body_text(&self, name: &str) -> TestResult<Option<String>> {
        let body: serde_json::Value = serde_json::from_slice(self.wire.body())?;
        Ok(body
            .get(name)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned))
    }
}

impl Scenario {
    pub async fn new(service: &str, operation: &str) -> TestResult<Self> {
        Self::new_region(service, operation, "us-east-1").await
    }

    pub async fn new_region(service: &str, operation: &str, region: &str) -> TestResult<Self> {
        let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
        let directory = tempfile::tempdir()?;
        let store = BinaryStore::create(
            &directory.path().join("leased.db"),
            SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
        )
        .await?;
        let sweep = SweepId::new("typed-native-pages".into())?;
        let account = AccountId::parse("123456789012")?;
        let home = RegionId::parse(region)?;
        let mut work = Vec::new();
        for plan in catalog.plans() {
            if plan.service_name() == service {
                draining::append_plan_work(
                    &account,
                    &home,
                    &sweep,
                    plan,
                    &[Region::Regional(home.clone())],
                    &mut work,
                )?;
            }
        }
        assert_eq!(
            work.len(),
            1,
            "only the native producer root may be admitted"
        );
        assert_eq!(
            work.first()
                .ok_or("root admission empty")?
                .scope()
                .operation()
                .as_str(),
            operation
        );
        store.seed_works(&work).await?;
        drop(store);
        Ok(Self {
            directory,
            catalog,
            sweep,
            service: service.into(),
            account,
            home,
        })
    }

    pub async fn run(
        &self,
        mut response: impl FnMut(NativePage<'_, '_>) -> TestResult<serde_json::Value>,
    ) -> TestResult {
        for _ in 0..32 {
            let store =
                BinaryStore::open_with_catalog(&self.path(), self.catalog.raw_bytes()).await?;
            let Some(lease) = store.claim(&self.sweep, now_ms()?, 60_000).await? else {
                return Ok(());
            };
            let plan = self
                .catalog
                .lookup(&self.service, lease.scope().operation().as_str())
                .ok_or("leased native plan absent")?;
            let base = Record::from_bytes(lease.bindings().to_vec())?;
            let cursor = cursor::Cursor::decode(lease.token())?;
            let mut fields = cursor.request_fields(plan.pagination())?;
            if let Some(size) = plan.pagination().page_size.as_ref() {
                fields.push(input::InputField {
                    path: size.path.clone(),
                    value: size.value(),
                });
            }
            let request = input::merge_input(&base, &fields)?;
            let routed = routing::request_region(
                plan,
                lease.scope().region(),
                &self.home,
                lease.scope().branch(),
            )?;
            let endpoint = routing::resolve(plan, &routed)?;
            let wire = crate::sweep_runtime::request::encode(plan, &endpoint.url, &base, &fields)?;
            let body = response(NativePage {
                plan,
                lease: &lease,
                request: &request,
                wire: &wire,
            })?;
            let record = decode(plan, &serde_json::to_vec(&body)?)?;
            self.apply(
                &store,
                lease,
                plan,
                cursor,
                record,
                PageRequest {
                    record: request,
                    region: routed,
                },
            )
            .await?;
            // The store and queue are reopened before every continuation or newly discovered child.
            drop(store);
        }
        Err("native page queue exceeded its finite scenario bound".into())
    }

    async fn apply(
        &self,
        store: &BinaryStore,
        lease: WorkLease,
        plan: &CallablePlan<'_>,
        cursor: cursor::Cursor,
        response: Record,
        request: PageRequest,
    ) -> TestResult {
        let consumers = ConsumerIndex::build(&self.catalog)?;
        let executor = ApiExecutor::new(ExecutorConfig::default())?;
        let context = assembly::Context {
            catalog: &self.catalog,
            account: &self.account,
            home: &self.home,
            executor: &executor,
            consumers: &consumers,
            max_pages: 10,
        };
        let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
        let response = call::DecodedResponse {
            record: response,
            headers: http::HeaderMap::new(),
        };
        let parts = assembly::prepare(
            &context, &lease, plan, cursor, response, &request, &ancestry,
        )
        .await?;
        let done = parts.next.is_none();
        let continuation_digest = parts
            .next
            .as_ref()
            .and_then(|next| next.continuation_digest);
        store
            .commit_page(PageCommit {
                lease,
                observations: parts.observations,
                unresolved: parts.references,
                edges: Vec::new(),
                tags: parts.tags,
                children: parts.children,
                done,
                continuation_digest,
                next_token: parts.next.map(|next| next.cursor.into_bytes()),
                observed_at_ms: now_ms()?,
            })
            .await?;
        Ok(())
    }

    fn path(&self) -> std::path::PathBuf {
        self.directory.path().join("leased.db")
    }

    pub async fn keys(&self, kind: &str) -> TestResult<Vec<(String, String, String, String)>> {
        let reader = self.reader().await?;
        let keys =
            sqlx::query_as("SELECT account,region,kind,id FROM resources WHERE kind=? ORDER BY id")
                .bind(kind)
                .fetch_all(&reader)
                .await?;
        reader.close().await;
        Ok(keys)
    }

    pub async fn edges(
        &self,
        kind: &str,
    ) -> TestResult<
        Vec<(
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        )>,
    > {
        let reader = self.reader().await?;
        let edges = sqlx::query_as("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation FROM edges WHERE from_kind=? ORDER BY from_id,to_id,relation")
            .bind(kind).fetch_all(&reader).await?;
        reader.close().await;
        Ok(edges)
    }

    async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.path())
                    .read_only(true),
            )
            .await?)
    }
}

pub(super) fn keys(kind: &str, ids: &[String]) -> Vec<(String, String, String, String)> {
    let mut keys = ids
        .iter()
        .map(|id| {
            (
                "123456789012".into(),
                "us-east-1".into(),
                kind.into(),
                id.clone(),
            )
        })
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

type EdgeRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

pub(super) fn edges(
    kind: &str,
    target_kind: &str,
    relation: &str,
    pairs: &[(String, String)],
) -> Vec<EdgeRow> {
    let mut edges = pairs
        .iter()
        .map(|(from, to)| {
            (
                "123456789012".into(),
                "us-east-1".into(),
                kind.into(),
                from.clone(),
                "123456789012".into(),
                "us-east-1".into(),
                target_kind.into(),
                to.clone(),
                relation.into(),
            )
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}
