use super::*;
impl<'a, 'catalog> Scenario<'a, 'catalog> {
    pub(super) async fn new(catalog: &'a RuntimeCatalog<'catalog>, name: &str) -> TestResult<Self> {
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

    pub(super) async fn seed(&self, plan: &CallablePlan<'_>) -> TestResult {
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

    pub(super) fn request(&self, plan: &CallablePlan<'_>, lease: &WorkLease) -> TestResult<(Record, Json)> {
        let (request, wire) = self.prepared_request(plan, lease)?;
        Ok((request, serde_json::from_slice(wire.body())?))
    }

    pub(super) fn prepared_request(
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
        if plan.service_name() == "global-accelerator" {
            assert_eq!(region.as_str(), "us-west-2");
            assert_eq!(endpoint.signing_region, "us-west-2");
            assert_eq!(lease.scope().region(), &Region::Global);
        }
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let mut fields = cursor.request_fields(plan.pagination())?;
        if let Some(size) = plan.pagination().page_size.as_ref() {
            fields.push(input::InputField {
                path: size.path.clone(),
                value: size.value(),
            });
        }
        let wire = crate::request::encode(plan, &endpoint.url, &base, &fields)?;
        Ok((input::merge_input(&base, &fields)?, wire))
    }

    pub(super) async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: Json,
    ) -> TestResult {
        use inventory_core::catalog::runtime::RuntimeProtocol;
        let bytes = if plan.protocol() == RuntimeProtocol::RpcV2Cbor {
            complete_df::json_to_cbor(&body)?
        } else {
            serde_json::to_vec(&body)?
        };
        let response = fixture::decode(plan, &bytes)?;
        let cursor = cursor::Cursor::decode(lease.token())?;
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let (observations, children, unresolved, tags) = match plan.purpose() {
            PlanPurpose::TagEndpoint(endpoint) => {
                let owner = lease
                    .scope()
                    .parent()
                    .ok_or("tag work lost its canonical owner")?;
                (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec![record::tag_observation(&response, owner, &endpoint.tags)?],
                )
            }
            _ => {
                let ancestry =
                    crate::execution::page::preparation::ancestry(lease.context(), lease.scope().context())?;
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
                let references = projected_references(
                    catalog,
                    &context,
                    lease.scope(),
                    request,
                    &response,
                    &observations,
                )?;
                (observations, children, references, Vec::new())
            }
        };
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
                tags,
                children,
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
