use super::*;
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
