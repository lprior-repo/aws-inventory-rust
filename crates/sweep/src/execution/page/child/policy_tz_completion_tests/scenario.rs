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

    pub(super) async fn seed(
        &self,
        catalog: &RuntimeCatalog<'_>,
        service: &str,
        operations: &[&str],
    ) -> TestResult {
        for operation in operations {
            let plan = catalog
                .lookup(service, operation)
                .ok_or_else(|| format!("seed plan absent: {service}.{operation}"))?;
            let region = if plan.scope() == PlanScope::Account {
                Region::Global
            } else {
                Region::Regional(self.home.clone())
            };
            let scope = ScopeKey::new(
                self.account.clone(),
                region.clone(),
                ServiceId::parse(service)?,
                OperationId::parse(operation)?,
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
        }
        Ok(())
    }

    pub(super) async fn seed_firehose(&self, catalog: &RuntimeCatalog<'_>) -> TestResult {
        self.seed(catalog, "firehose", &["ListDeliveryStreams"])
            .await
    }

    pub(super) fn request(
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

    pub(super) async fn apply(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: Json,
    ) -> TestResult {
        let bytes =
            if plan.protocol() == inventory_core::catalog::runtime::RuntimeProtocol::RpcV2Cbor {
                let service = plan.service();
                let output_target = plan
                    .operation()
                    .output()
                    .map_err(|e| format!("operation output error: {e}"))?;
                json_to_cbor(&body, service, output_target)?
            } else {
                serde_json::to_vec(&body)?
            };
        self.apply_wire(catalog, plan, lease, request, &bytes).await
    }

    pub(super) async fn apply_wire(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        lease: WorkLease,
        request: &Record,
        body: &[u8],
    ) -> TestResult {
        let executor = inventory_engine::limiter::ApiExecutor::new(
            inventory_engine::limiter::ExecutorConfig::default(),
        )?;
        let context = assembly::Context {
            catalog,
            account: &self.account,
            home: &self.home,
            executor: &executor,
            consumers: &self.consumers,
            max_pages: 10,
        };
        let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
        let parts = assembly::prepare(
            &context,
            &lease,
            plan,
            cursor::Cursor::decode(lease.token())?,
            crate::call::DecodedResponse {
                record: decode(plan, body)?,
                headers: http::HeaderMap::new(),
            },
            &PageRequest {
                record: request.clone(),
                region: routing::request_region(
                    plan,
                    lease.scope().region(),
                    &self.home,
                    lease.scope().branch(),
                )?,
            },
            &ancestry,
        )
        .await?;
        let done = parts.next.is_none();
        let (next_token, continuation_digest) = parts.next.map_or((None, None), |advance| {
            (
                Some(advance.cursor.into_bytes()),
                advance.continuation_digest,
            )
        });
        self.store
            .commit_page(PageCommit {
                lease,
                observations: parts.observations,
                unresolved: parts.references,
                tags: parts.tags,
                children: parts.children,
                edges: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: 1001,
            })
            .await?;
        Ok(())
    }

}
