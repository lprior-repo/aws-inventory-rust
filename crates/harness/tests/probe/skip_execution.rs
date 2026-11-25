use super::{
    Server, TestResult,
    server::target,
    skip_runtime::{self, Case},
};
use aws_wire::{
    sign::Credentials,
    transport::{Client, Config},
};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::RegionId};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use inventory_sweep::{CredentialFailure, CredentialSource, SweepConfig, SweepControl};
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

async fn run(case: Case) -> TestResult<(inventory_sweep::SweepSummary, Vec<String>, Vec<String>)> {
    let bytes = skip_runtime::catalog(case)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let work = tempfile::tempdir()?;
    let path = work.path().join("inventory.sqlite");
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let server = Server::new().await?;
    let config = SweepConfig {
        region: Some("us-east-1".into()),
        max_pages: 4,
        concurrency: 1,
        resume: false,
        endpoint: Some(server.endpoint()?),
        ca_bundle: Some(server.ca.clone()),
        slice_timeout_secs: None,
        max_rps: None,
        max_rps_milli: None,
        idle_timeout: None,
        pool_idle_per_host: None,
        max_page_size: None,
    };
    let bodies = [
        r#"{"Account":"123456789012"}"#,
        r#"{"Regions":[{"RegionName":"us-east-1"}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    ];
    let (completed, completion) = tokio::sync::oneshot::channel();
    let (summary, requests) = tokio::time::timeout(Duration::from_secs(20), async {
        tokio::try_join!(
            async {
                let result = inventory_sweep::run(
                    &catalog,
                    &store,
                    config,
                    &TestCredentials,
                    SweepControl::default(),
                )
                .await;
                completed.send(()).map_err(|_| "sweep server closed")?;
                Ok::<_, Box<dyn std::error::Error>>(result?)
            },
            server.serve_until(&bodies, completion)
        )
    })
    .await??;
    let resources = store.projection_resources(None, 16).await?;
    let kinds = resources
        .iter()
        .map(|resource| resource.key.kind.as_str().to_owned())
        .collect();
    Ok((summary, requests, kinds))
}

#[tokio::test]
async fn given_reviewed_skip_when_full_sweep_runs_then_no_child_call_and_kind_rows_survive()
-> TestResult {
    let (ordinary, ordinary_requests, _) = run(Case::Ordinary).await?;
    let (skipped, skipped_requests, kinds) = run(Case::Skipped).await?;
    assert_eq!(
        ordinary_requests
            .iter()
            .filter(|request| request.contains(".DescribeThing"))
            .count(),
        1
    );
    assert_eq!(
        skipped_requests
            .iter()
            .filter(|request| request.contains(".DescribeThing"))
            .count(),
        0
    );
    assert_eq!(
        (ordinary.metrics.api_attempts, skipped.metrics.api_attempts),
        (4, 3)
    );
    assert_eq!((ordinary.outcomes.failed, skipped.outcomes.failed), (0, 0));
    assert_eq!(kinds, ["AWS::ApiOnly::SkipThing"]);
    assert_eq!(skipped.rows, 1);
    Ok(())
}

#[tokio::test]
async fn given_reviewed_skip_when_probing_then_child_is_called_with_the_listed_identifier()
-> TestResult {
    let bytes = skip_runtime::catalog(Case::Skipped)?;
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
    let (report, requests) = server
        .capture_with(
            async {
                Ok(inventory_harness::probe::run(
                    &catalog,
                    &TestCredentials,
                    &client,
                    &executor,
                    &region,
                    &endpoint,
                    &[],
                )
                .await?)
            },
            |request| match target(request)? {
                "GetCallerIdentity" => Ok((200, r#"{"Account":"123456789012"}"#)),
                "DescribeRegions" => Ok((200, r#"{"Regions":[{"RegionName":"us-east-1"}]}"#)),
                "ListThings" | "DescribeThing" => {
                    Ok((200, r#"{"Items":[{"Id":"one","Count":7}]}"#))
                }
                _ => Err("unexpected probe operation".into()),
            },
        )
        .await?;
    let mut describes = requests
        .iter()
        .filter(|request| request.contains(".DescribeThing"));
    let request = describes.next().ok_or("probe never sent DescribeThing")?;
    assert_eq!(describes.next(), None);
    let (_, body) = request
        .split_once("\r\n\r\n")
        .ok_or("missing request body")?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(body)?,
        serde_json::json!({"Id":"one"})
    );
    let expected = "skipruntime DescribeThing awsJson1_1 items 1";
    assert!(report.lines().any(|line| line == expected), "{report}");
    Ok(())
}

#[tokio::test]
async fn given_parent_additional_emission_when_different_kind_child_is_skipped_then_both_kinds_are_stored()
-> TestResult {
    let (summary, requests, mut kinds) = run(Case::AdditionalEmission).await?;
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains(".DescribeThing"))
            .count(),
        0
    );
    kinds.sort();
    assert_eq!(
        kinds,
        ["AWS::ApiOnly::SkipChild", "AWS::ApiOnly::SkipThing"]
    );
    assert_eq!(summary.rows, 2);
    assert_eq!(summary.outcomes.failed, 0);
    Ok(())
}

#[tokio::test]
async fn given_same_named_child_in_another_service_when_parent_skips_local_child_then_remote_child_still_runs()
-> TestResult {
    let (summary, requests, mut kinds) = run(Case::CrossService).await?;
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("Skipruntime.DescribeThing"))
            .count(),
        0
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.contains("Otherskip.DescribeThing"))
            .count(),
        1
    );
    assert_eq!(summary.metrics.api_attempts, 4);
    kinds.sort();
    assert_eq!(
        kinds,
        ["AWS::ApiOnly::SkipChild", "AWS::ApiOnly::SkipThing"]
    );
    Ok(())
}
