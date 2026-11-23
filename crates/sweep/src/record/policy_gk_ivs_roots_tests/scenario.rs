use super::*;
use crate::execution::page::child::policy_dfgk_tests::{project_children, projected_references};
use inventory_engine::inventory::WorkLease;
mod assertions;

pub(super) struct Scenario {
    pub directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub account: AccountId,
    pub home: RegionId,
}

impl Scenario {
    pub async fn new(catalog: &RuntimeCatalog<'_>, case: &cases::Case) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("ivs.sqlite"), registry).await?;
        Ok(Self {
            directory,
            store,
            sweep: SweepId::new(case.operation.to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
        })
    }

    pub async fn seed(&self, catalog: &RuntimeCatalog<'_>, case: &cases::Case) -> TestResult {
        let plan = catalog
            .lookup("ivs", case.operation)
            .ok_or("IVS root plan absent")?;
        assert_eq!(plan.kind().as_str(), case.kind);
        assert_eq!(plan.scope(), PlanScope::Region);
        assert_eq!(
            plan.bindings().len(),
            0,
            "unfiltered list must not depend on a parent"
        );
        let region = Region::Regional(self.home.clone());
        let scope = ScopeKey::new(
            self.account.clone(),
            region.clone(),
            ServiceId::parse("ivs")?,
            OperationId::parse(case.operation)?,
            None,
        )?;
        let base = input::root_bindings(plan, Some(&self.account), Some(&region))?;
        self.store
            .seed_work(WorkSpec::new(
                self.sweep.clone(),
                scope,
                None,
                base.into_bytes(),
            )?)
            .await?;
        Ok(())
    }

    pub async fn claim(&self, case: &cases::Case, at: i64) -> TestResult<WorkLease> {
        let lease = self
            .store
            .claim(&self.sweep, at, 10)
            .await?
            .ok_or("IVS root work absent")?;
        assert_eq!(lease.scope().operation().as_str(), case.operation);
        assert_eq!(lease.scope().service().as_str(), "ivs");
        assert_eq!(lease.scope().parent(), None);
        Ok(lease)
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
        let store = BinaryStore::open(&directory.path().join("ivs.sqlite")).await?;
        Ok(Self {
            directory,
            store,
            sweep,
            account,
            home,
        })
    }

    fn request(
        &self,
        plan: &CallablePlan<'_>,
        lease: &WorkLease,
        page: usize,
    ) -> TestResult<Record> {
        let cursor = cursor::Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let region = routing::request_region(
            plan,
            lease.scope().region(),
            &self.home,
            lease.scope().branch(),
        )?;
        let endpoint = routing::resolve(plan, &region)?;
        let wire = request::encode(plan, &endpoint.url, &base, &fields)?;
        assert_eq!(wire.method(), http::Method::POST);
        assert_eq!(wire.uri().host(), Some("ivs.us-east-1.amazonaws.com"));
        assert_eq!(wire.uri().path(), format!("/{}", plan.operation_name()));
        assert_eq!(wire.uri().query(), None);
        let expected = match page {
            0 => json!({}),
            1 => json!({"nextToken":"second"}),
            2 => json!({"nextToken":"third"}),
            _ => return Err("unexpected IVS request page".into()),
        };
        assert_eq!(serde_json::from_slice::<Json>(wire.body())?, expected);
        Ok(input::merge_input(&base, &fields)?)
    }

    pub async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        case: &cases::Case,
        lease: WorkLease,
        page: usize,
        body: Json,
        at: i64,
    ) -> TestResult {
        let plan = catalog
            .lookup("ivs", case.operation)
            .ok_or("IVS leased plan absent")?;
        let request = self.request(plan, &lease, page)?;
        let response =
            record::policy_dfgk_tests::fixture::decode(plan, &serde_json::to_vec(&body)?)?;
        for row in record::item_records(&response, plan.items())? {
            assert!(
                crate::row_policy::selected(plan, None, &row, &request, &response)?,
                "unfiltered IVS root unexpectedly suppresses owned rows"
            );
        }
        let consumers = ConsumerIndex::build(catalog)?;
        let context = child::Context::new(plan, &self.sweep, &self.account, &self.home, &consumers);
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let (observations, children) = project_children(
            catalog,
            &context,
            lease.scope(),
            &request,
            &response,
            &ancestry,
        )?;
        assert_eq!(
            children.len(),
            0,
            "metadata root must not feed secret, content, or detail calls"
        );
        assert_eq!(observations.len(), if page == 0 { 0 } else { 1 });
        let unresolved = projected_references(
            catalog,
            &context,
            lease.scope(),
            &request,
            &response,
            &observations,
        )?;
        assert_eq!(unresolved.len(), 0);
        let tag_plan = plan.tags().ok_or("IVS root tags are not compiled")?;
        let tags = observations
            .iter()
            .map(|observation| {
                record::tag_observation(observation.record(), observation.key(), tag_plan)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let next = cursor::Cursor::decode(lease.token())?.next(
            plan.pagination(),
            &response,
            &http::HeaderMap::new(),
            10,
        )?;
        let done = next.is_none();
        assert_eq!(
            done,
            page == 2,
            "empty pages must preserve the native token"
        );
        let (next_token, continuation_digest) = next.map_or((None, None), |next| {
            (Some(next.cursor.into_bytes()), next.continuation_digest)
        });
        self.store
            .commit_page(PageCommit {
                lease,
                observations,
                children,
                unresolved,
                tags,
                edges: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: at,
            })
            .await?;
        Ok(())
    }

    pub async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("ivs.sqlite"))
                    .read_only(true),
            )
            .await?)
    }

    pub async fn assert_empty(&self) -> TestResult {
        assert_eq!(self.store.projection_resources(None, 10).await?, Vec::new());
        assert_eq!(
            self.store.projection_edges(None, 10).await?.edges,
            Vec::new()
        );
        let reader = self.reader().await?;
        for table in [
            "resources",
            "resource_facets",
            "tags",
            "edges",
            "unresolved_refs",
            "scope_outcomes",
        ] {
            let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT COUNT(*) FROM \"");
            query.push(table).push("\"");
            let count: i64 = query.build_query_scalar().fetch_one(&reader).await?;
            assert_eq!(count, 0, "rejected/empty page mutated {table}");
        }
        reader.close().await;
        Ok(())
    }
}
