use super::*;
use inventory_core::catalog::runtime::CallablePlan;
use inventory_engine::inventory::WorkLease;

struct Delivery<'a> {
    actual: &'a Record,
    response: &'a Record,
    at: i64,
    page: usize,
}

pub(super) struct Scenario {
    pub(super) store: BinaryStore,
    pub(super) sweep: SweepId,
    pub(super) account: AccountId,
    pub(super) home: RegionId,
    pub(super) path: std::path::PathBuf,
    _directory: tempfile::TempDir,
}

impl Scenario {
    pub(super) async fn new(catalog: &RuntimeCatalog<'_>, name: &str) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("container-fleets.sqlite");
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&path, registry).await?;
        let scenario = Self {
            store,
            sweep: SweepId::new(name.to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-west-2")?,
            path,
            _directory: directory,
        };
        scenario.seed(catalog).await?;
        Ok(scenario)
    }

    async fn seed(&self, catalog: &RuntimeCatalog<'_>) -> TestResult {
        let plan = plan(catalog)?;
        assert_eq!(plan.scope(), PlanScope::Region);
        assert_eq!(plan.kind().as_str(), KIND);
        assert!(!input::requires_parent(plan));
        let region = Region::Regional(RegionId::parse("us-east-1")?);
        let scope = ScopeKey::new(
            self.account.clone(),
            region.clone(),
            ServiceId::parse("gamelift")?,
            OperationId::parse("ListContainerFleets")?,
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

    pub(super) async fn page(
        &self,
        catalog: &RuntimeCatalog<'_>,
        at: i64,
        page: usize,
        body: Json,
    ) -> TestResult {
        let lease = self
            .store
            .claim(&self.sweep, at, 10_000)
            .await?
            .ok_or("fleet page was not reclaimable")?;
        let plan = plan(catalog)?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let region = routing::request_region(
            plan,
            lease.scope().region(),
            &self.home,
            lease.scope().branch(),
        )?;
        assert_eq!(region.as_str(), "us-east-1");
        assert_eq!(lease.scope().service().as_str(), "gamelift");
        assert_eq!(lease.scope().operation().as_str(), "ListContainerFleets");
        assert_eq!(lease.scope().parent(), None);
        let endpoint = routing::resolve(plan, &region)?;
        let wire = request::encode(plan, &endpoint.url, &base, &fields)?;
        verify_wire(page, &wire)?;
        let actual = input::merge_input(&base, &fields)?;
        let response = record::policy_dfgk_tests::fixture::decode(
            plan,
            &crate::record::policy_gk_root_tests::cbor::response(plan, &body)?,
        )?;
        let delivery = Delivery {
            actual: &actual,
            response: &response,
            at,
            page,
        };
        self.commit(catalog, plan, lease, cursor, delivery).await
    }

    async fn commit(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        cursor: Cursor,
        delivery: Delivery<'_>,
    ) -> TestResult {
        let Delivery {
            actual,
            response,
            at,
            page,
        } = delivery;
        let consumers = ConsumerIndex::build(catalog)?;
        let context = child::Context::new(plan, &self.sweep, &self.account, &self.home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(
            catalog,
            &context,
            lease.scope(),
            actual,
            response,
            &DiscoveryContext::empty(),
        )?;
        assert_eq!(
            children.len(),
            0,
            "fleet metadata must not schedule detail or credentials"
        );
        assert_eq!(observations.len(), usize::from(page != 0));
        for observation in &observations {
            assert_eq!(observation.key().kind.as_str(), KIND);
            assert_eq!(&observation.key().account, &self.account);
            assert_eq!(
                observation.key().region,
                Region::Regional(RegionId::parse("us-east-1")?)
            );
            assert_eq!(
                observation.key().id.as_str(),
                fixture::arn(if page == 1 { "first" } else { "later" })
            );
        }
        let unresolved = child::policy_dfgk_tests::projected_references(
            catalog,
            &context,
            lease.scope(),
            actual,
            response,
            &observations,
        )?;
        assert_eq!(
            unresolved.len(),
            0,
            "metadata references must not invent owned graph nodes"
        );
        let next = cursor.next(plan.pagination(), response, &http::HeaderMap::new(), 10)?;
        let done = next.is_none();
        assert_eq!(
            done,
            page == 2,
            "empty first page must preserve continuation"
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
                edges: Vec::new(),
                tags: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: at,
            })
            .await?;
        Ok(())
    }

    pub(super) async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&self.path)
                    .read_only(true),
            )
            .await?)
    }
}

pub(super) fn plan<'r, 'a>(catalog: &'r RuntimeCatalog<'a>) -> TestResult<&'r CallablePlan<'a>> {
    catalog
        .lookup("gamelift", "ListContainerFleets")
        .ok_or_else(|| "container fleet plan absent".into())
}

fn verify_wire(page: usize, wire: &http::Request<bytes::Bytes>) -> TestResult {
    let expected = match page {
        0 => json!({}),
        1 => json!({"NextToken":"second"}),
        2 => json!({"NextToken":"third"}),
        _ => return Err("unexpected fleet request page".into()),
    };
    let actual = crate::record::policy_gk_root_tests::cbor::request(
        wire,
        "GameLift",
        "ListContainerFleets",
    )?;
    assert_eq!(
        actual, expected,
        "must not filter ContainerGroupDefinitionName"
    );
    Ok(())
}
