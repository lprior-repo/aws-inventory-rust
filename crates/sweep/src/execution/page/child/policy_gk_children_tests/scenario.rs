use super::*;
impl<'a, 'catalog> Scenario<'a, 'catalog> {
    pub(super) async fn new(catalog: &'a RuntimeCatalog<'catalog>) -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        Ok(Self {
            directory,
            store,
            sweep: SweepId::new("gk-children-consumer".to_owned())?,
            account: AccountId::parse("123456789012")?,
            home: RegionId::parse("us-east-1")?,
            consumers: ConsumerIndex::build(catalog)?,
        })
    }

    pub(super) async fn reopen(self) -> TestResult<Self> {
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

    pub(super) async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
        let region = Region::Regional(self.home.clone());
        let scope = ScopeKey::new(
            self.account.clone(),
            region.clone(),
            ServiceId::parse(plan.service_name())?,
            OperationId::parse(plan.operation_name())?,
            None,
        )?;
        let bindings = input::root_bindings(plan, Some(&self.account), Some(&region))?;
        self.store
            .seed_work(WorkSpec::new(
                self.sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
        Ok(())
    }

    pub(super) fn request(
        &self,
        plan: &CallablePlan<'_>,
        lease: &WorkLease,
    ) -> TestResult<(Record, String)> {
        let region = routing::request_region(
            plan,
            lease.scope().region(),
            &self.home,
            lease.scope().branch(),
        )?;
        let endpoint = routing::resolve(plan, &region)?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let restored = cursor::Cursor::decode(lease.token())?;
        let mut fields = restored.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField {
                path: size.path.clone(),
                value: Value::Int(i64::from(size.max)),
            });
        }
        let wire = crate::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((
            input::merge_input(&base, &fields)?,
            format!("{} {}", wire.uri(), String::from_utf8_lossy(wire.body())),
        ))
    }

    pub(super) async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: Json,
        expected_children: usize,
    ) -> TestResult {
        let bytes = if plan.protocol() == RuntimeProtocol::RpcV2Cbor {
            json_to_cbor(&body)?
        } else {
            serde_json::to_vec(&body)?
        };
        let response = decoder::decode_as_sweep(plan, &bytes)?;
        let restored = cursor::Cursor::decode(lease.token())?;
        let next = restored.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let branch = routing::execution_branch(plan, lease.scope().branch())?;
        for row in record::item_records(&response, plan.items())? {
            assert!(
                crate::row_policy::selected(plan, branch, &row, request, &response)?,
                "reviewed child rows are never filtered out"
            );
        }
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let expected_context = (!lease.context().is_empty())
            .then(|| ancestry.discriminator())
            .transpose()?;
        assert_eq!(lease.scope().context(), expected_context);
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
        assert_eq!(
            children.len(),
            expected_children,
            "unexpected detail or missing child fanout: {}",
            plan.operation_name()
        );
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

    pub(super) async fn reader(&self) -> TestResult<sqlx::SqlitePool> {
        Ok(sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(self.directory.path().join("inventory.db"))
                    .read_only(true),
            )
            .await?)
    }
}
