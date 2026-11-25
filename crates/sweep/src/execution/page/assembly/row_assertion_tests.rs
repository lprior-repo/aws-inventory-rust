use super::*;
use crate::consumer_fixture as fixture;
use fixture::TestResult;
use inventory_core::{
    inventory::{Kind, OperationId, Region, ResourceId, ResourceKey, ScopeKey, ServiceId},
    record::{Key, Record, Value},
};
use inventory_engine::{
    inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec},
    limiter::{ApiExecutor, ExecutorConfig},
};
use std::collections::BTreeMap;

#[tokio::test]
async fn given_an_owned_prefix_then_hostile_filtered_row_after_chunk_boundary_when_prepared_then_assertion_aborts_page_and_reopen_has_no_prefix_or_cursor()
-> TestResult {
    // Given actual Cognito Smithy and production ownership plus parent-correlation policies.
    let compiled = fixture::compile(
        "cognito-identity-provider",
        "2016-04-18",
        &["ListUserPools", "ListResourceServers"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("cognito-identity-provider", "ListResourceServers")
        .ok_or("resource server plan absent")?;
    let field = compiled
        .fields
        .id("UserPoolId")
        .ok_or("modeled pool field absent")?;
    let page = PageFixture::new(&catalog, plan, field).await?;
    let home = RegionId::parse("us-east-1")?;
    let resources = TestResources::new()?;
    let progress = crate::control::Progress::new(None);
    let context = crate::execution::RuntimeContext {
        bootstrap: resources.bootstrap(&catalog, &home),
        store: &page.store,
        account: AccountId::parse("123456789012")?,
        lease_ms: 3_600_000,
        endpoints: Default::default(),
        consumers: ConsumerIndex::build(&catalog)?,
        progress: &progress,
    };
    // When the real persistence caller sees a hostile row after a complete valid chunk.
    let summary = super::super::commit_response(
        &context,
        page.lease,
        false,
        super::super::PageSetup {
            plan,
            cursor: cursor::Cursor::default(),
            ancestry: DiscoveryContext::empty(),
        },
        hostile_response(plan)?,
        PageRequest {
            record: page.request,
            region: home.clone(),
        },
    )
    .await?;
    // Then it records a failed scope, not a partial prefix or continuation.
    assert_eq!(summary.summary.outcomes.failed, 1);
    assert_eq!(summary.summary.rows, 0);
    assert_eq!(summary.summary.resource_writes, 0);
    assert!(summary.next.is_none());
    drop(context);
    assert_durable_failure(
        page.store,
        &page.directory.path().join("late-row-assertion.sqlite"),
        &page.sweep,
    )
    .await
}

struct PageFixture {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    lease: inventory_engine::inventory::WorkLease,
    request: Record,
}

impl PageFixture {
    async fn new(
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        field: inventory_core::record::FieldId,
    ) -> TestResult<Self> {
        let request = Record::encode(&Value::Map(BTreeMap::from([(
            Key::Field(field),
            Value::Text("us-east-1_Parent".to_owned()),
        )])))?;
        let scope = ScopeKey::new(
            AccountId::parse("123456789012")?,
            Region::Regional(RegionId::parse("us-east-1")?),
            ServiceId::parse(plan.service_name())?,
            OperationId::parse(plan.operation_name())?,
            Some(ResourceKey {
                account: AccountId::parse("123456789012")?,
                region: Region::Regional(RegionId::parse("us-east-1")?),
                kind: Kind::parse("AWS::Cognito::UserPool")?,
                id: ResourceId::parse("us-east-1_Parent")?,
            }),
        )?;
        let directory = tempfile::tempdir()?;
        let store = BinaryStore::create(
            &directory.path().join("late-row-assertion.sqlite"),
            SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
        )
        .await?;
        let sweep = SweepId::new("late-hostile-row".to_owned())?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                request.clone().into_bytes(),
            )?)
            .await?;
        let lease = store
            .claim(&sweep, crate::execution::now_ms()?, 3_600_000)
            .await?
            .ok_or("lease absent")?;
        Ok(Self {
            directory,
            store,
            sweep,
            lease,
            request,
        })
    }
}

fn hostile_response(plan: &CallablePlan<'_>) -> TestResult<call::DecodedResponse> {
    let count = ROWS_PER_CHECKPOINT
        .checked_add(1)
        .ok_or("row count overflow")?;
    let mut wire_rows = (0..count)
        .map(|ordinal| serde_json::json!({
            "UserPoolId":"us-east-1_Parent","Identifier":format!("owned-{ordinal}"),"Name":"customer owned",
        }))
        .collect::<Vec<_>>();
    // A provider-reference filter must not conceal its contradictory returned parent.
    wire_rows.push(serde_json::json!({"UserPoolId":"us-east-1_Unrelated","Identifier":"provider","ManagedBy":"COGNITO"}));
    let bytes = serde_json::to_vec(
        &serde_json::json!({"ResourceServers":wire_rows,"NextToken":"must-not-commit"}),
    )?;
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson10 => {
            aws_wire::request::Protocol::AwsJson10
        }
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => {
            aws_wire::request::Protocol::AwsJson11
        }
        _ => return Err("unexpected actual Cognito protocol".into()),
    };
    Ok(call::DecodedResponse {
        http_version: http::Version::HTTP_11,
        record: aws_wire::response::decode(aws_wire::response::ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol,
            body: &bytes,
            headers: &http::HeaderMap::new(),
            status: http::StatusCode::OK,
            limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
        })?,
        headers: http::HeaderMap::new(),
    })
}

async fn assert_durable_failure(
    store: BinaryStore,
    path: &std::path::Path,
    sweep: &SweepId,
) -> TestResult {
    let before = store.sweep_summary(sweep).await?;
    assert_eq!(before.resources_observed, 0);
    assert_eq!(before.complete_scopes, 0);
    assert_eq!(before.failed_scopes, 1);
    assert_eq!(before.work_pending, 0);
    drop(store);
    let reopened = BinaryStore::open(path).await?;
    assert_eq!(reopened.sweep_summary(sweep).await?, before);
    let later = crate::execution::now_ms()?
        .checked_add(3_600_001)
        .ok_or("clock overflow")?;
    let restored = reopened.claim(sweep, later, 3_600_000).await?;
    assert!(
        restored.is_none(),
        "failed page queued its continuation cursor"
    );
    Ok(())
}

struct TestResources {
    executor: ApiExecutor,
    client: aws_wire::transport::Client,
    credentials: crate::credentials::TestCredentials,
    config: crate::SweepConfig,
    metrics: crate::metrics::Accumulators,
    endpoints: crate::execution::resolved_endpoints::ResolvedEndpoints,
}

impl TestResources {
    fn new() -> TestResult<Self> {
        Ok(Self {
            executor: ApiExecutor::new(ExecutorConfig::default())?,
            client: aws_wire::transport::Client::new(aws_wire::transport::Config::default())?,
            credentials: crate::credentials::TestCredentials::new("us-east-1"),
            config: crate::SweepConfig {
                region: Some("us-east-1".to_owned()),
                max_pages: 10,
                concurrency: 1,
                resume: false,
                endpoint: None,
                ca_bundle: None,
                slice_timeout_secs: None,
                max_rps: None,
                max_page_size: None,
            },
            metrics: Default::default(),
            endpoints: Default::default(),
        })
    }

    fn bootstrap<'a, 'catalog>(
        &'a self,
        catalog: &'a RuntimeCatalog<'catalog>,
        home: &'a RegionId,
    ) -> crate::execution::BootstrapContext<'a, 'catalog, crate::credentials::TestCredentials> {
        crate::execution::BootstrapContext {
            catalog,
            client: &self.client,
            executor: &self.executor,
            credentials: &self.credentials,
            home,
            config: &self.config,
            metrics: &self.metrics,
            resolved_endpoints: &self.endpoints,
        }
    }
}
