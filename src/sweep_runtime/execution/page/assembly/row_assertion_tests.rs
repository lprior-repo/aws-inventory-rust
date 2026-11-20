use super::*;
use catalog_tool as fixture_catalog;
#[path = "../../../../../crates/catalog-tool/src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod fixture;
use fixture::TestResult;
use inventory_core::{inventory::{OperationId,Region,ResourceId,ScopeKey,ServiceId},record::{Key,Record,Value}};
use inventory_engine::{inventory::{BinaryStore,SchemaRegistry,SweepId,WorkSpec},limiter::{ApiExecutor,ExecutorConfig}};
use std::collections::BTreeMap;

#[tokio::test]
async fn given_an_owned_prefix_then_hostile_filtered_row_after_chunk_boundary_when_prepared_then_assertion_aborts_page_and_reopen_has_no_prefix_or_cursor() -> TestResult {
    // Given actual Cognito Smithy and production ownership plus parent-correlation policies.
    let compiled = fixture::compile("cognito-identity-provider","2016-04-18",&["ListUserPools","ListResourceServers"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("cognito-identity-provider","ListResourceServers").ok_or("resource server plan absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let pool = "us-east-1_Parent";
    let field = compiled.fields.id("UserPoolId").ok_or("modeled pool field absent")?;
    let request_record = Record::encode(&Value::Map(BTreeMap::from([(Key::Field(field),Value::Text(pool.to_owned()))])))?;
    let scope = ScopeKey::new(account.clone(),Region::Regional(home.clone()),ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,Some(ResourceId::parse(pool)?))?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("late-row-assertion.sqlite");
    let store = BinaryStore::create(&path,SchemaRegistry::new(catalog.raw_bytes(),catalog.schemas().to_vec())?).await?;
    let sweep = SweepId::new("late-hostile-row".to_owned())?;
    store.seed_work(WorkSpec::new(sweep.clone(),scope,None,request_record.clone().into_bytes())?).await?;
    let lease = store.claim(&sweep,crate::sweep_runtime::execution::now_ms()?,3_600_000).await?.ok_or("lease absent")?;
    let mut wire_rows = (0..ROWS_PER_CHECKPOINT + 1).map(|ordinal| serde_json::json!({
        "UserPoolId":pool,"Identifier":format!("owned-{ordinal}"),"Name":"customer owned",
    })).collect::<Vec<_>>();
    // A provider-reference filter must not conceal its contradictory returned parent.
    wire_rows.push(serde_json::json!({"UserPoolId":"us-east-1_Unrelated","Identifier":"provider","ManagedBy":"COGNITO"}));
    let bytes = serde_json::to_vec(&serde_json::json!({"ResourceServers":wire_rows,"NextToken":"must-not-commit"}))?;
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        _ => return Err("unexpected actual Cognito protocol".into()),
    };
    let response = call::DecodedResponse { record:aws_wire::response::decode(aws_wire::response::ResponseContext {
        service:plan.service(),operation:plan.operation(),protocol,body:&bytes,headers:&http::HeaderMap::new(),
        status:http::StatusCode::OK,limits:aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    })?,headers:http::HeaderMap::new() };
    let consumers = ConsumerIndex::build(&catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let client = aws_wire::transport::Client::new(aws_wire::transport::Config::default())?;
    let credentials = inventory_aws::model_credentials::ProfileCredentials::load(
        Some("isolated-row-assertion-page"),Some("us-east-1")).await?;
    let config = crate::sweep_runtime::SweepConfig { region:Some("us-east-1".to_owned()),
        max_pages:10,concurrency:1,resume:false,endpoint:None,ca_bundle:None,slice_timeout_secs:None };
    let context = crate::sweep_runtime::execution::RuntimeContext {
        bootstrap:crate::sweep_runtime::execution::BootstrapContext {
            catalog:&catalog,client:&client,executor:&executor,credentials:&credentials,home:&home,config:&config,
        },
        store:&store,account,lease_ms:3_600_000,endpoints:Default::default(),consumers,
    };
    // When the real persistence caller sees a hostile row after a complete valid chunk.
    let summary = super::super::commit_response(&context,lease,plan,cursor::Cursor::default(),response,
        PageRequest { record:request_record,region:home.clone() },&DiscoveryContext::empty()).await?;
    // Then it records a failed scope, not a successful partial-prefix page or a continuation.
    assert_eq!(summary.outcomes.failed,1);
    assert_eq!(summary.rows,0);
    assert_eq!(summary.resource_writes,0);
    let before = store.sweep_summary(&sweep).await?;
    assert_eq!(before.resources_observed,0);
    assert_eq!(before.complete_scopes,0);
    assert_eq!(before.failed_scopes,1);
    assert_eq!(before.work_pending,0);
    drop(context);
    drop(store);
    let reopened = BinaryStore::open(&path).await?;
    assert_eq!(reopened.sweep_summary(&sweep).await?,before);
    let restored = reopened.claim(&sweep,crate::sweep_runtime::execution::now_ms()? + 3_600_001,3_600_000).await?;
    assert!(restored.is_none(),"failed page queued its continuation cursor");
    Ok(())
}
