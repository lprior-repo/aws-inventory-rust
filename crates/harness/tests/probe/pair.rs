use super::{Server, TestResult, fixture};
use aws_wire::{
    sign::Credentials,
    transport::{Client, Config},
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, RegionId},
    record::{Value, ValueRef},
};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use inventory_sweep::{CredentialFailure, CredentialSource, ProbeScope, ProbeSession, probe_pair};
use std::time::Duration;

struct TestCredentials;
impl CredentialSource for TestCredentials {
    fn region(&self) -> &str {
        "us-east-1"
    }
    async fn resolve(&self) -> Result<Credentials, CredentialFailure> {
        Credentials::new("TEST", "TEST", None).map_err(|error| CredentialFailure {
            retryable: false,
            message: error.to_string(),
        })
    }
}

#[tokio::test]
async fn given_bound_parent_row_when_pair_is_probed_then_raw_null_and_filled_values_survive()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..Config::default()
    })?;
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let region = RegionId::parse("us-east-1")?;
    let endpoint = server.endpoint()?;
    let account = AccountId::parse("123456789012")?;
    let parent = catalog
        .lookup("bindings", "ListParents0")
        .ok_or("parent missing")?;
    let child = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child missing")?;
    let session = ProbeSession::new(
        &catalog,
        &TestCredentials,
        &client,
        &executor,
        parent,
        ProbeScope {
            region: &region,
            account: Some(&account),
            endpoint_override: Some(&endpoint),
        },
    )?;
    let (completed, completion) = tokio::sync::oneshot::channel();
    let probe = async {
        let result = assert_raw_values(&session, child).await;
        completed.send(()).map_err(|_| "comparison server closed")?;
        result
    };
    let bodies = [
        r#"{"Items":[{"Id":"one","Count":null}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    ];
    let (_, requests) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(probe, server.serve_until(&bodies, completion))
    })
    .await??;
    assert_eq!(requests.len(), 2);
    Ok(())
}

async fn assert_raw_values<'catalog>(
    session: &ProbeSession<'_, 'catalog, TestCredentials>,
    child: &inventory_core::catalog::runtime::CallablePlan<'catalog>,
) -> TestResult {
    let page = session.root_page().await?;
    let row = page.items.first().ok_or("row missing")?;
    let (list, describe) = probe_pair(session, child, &page, row).await?;
    let Value::Map(list) = ValueRef::Map(list.root()).to_value()? else {
        return Err("list is not a structure".into());
    };
    let Value::Map(describe) = ValueRef::Map(describe.root()).to_value()? else {
        return Err("describe is not a structure".into());
    };
    assert!(list.values().any(|value| matches!(value, Value::Null)));
    assert!(
        describe
            .values()
            .any(|value| matches!(value, Value::Int(7)))
    );
    Ok(())
}
