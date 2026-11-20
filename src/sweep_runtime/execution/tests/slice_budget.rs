use super::support::{Fixture, TestResult};
use super::*;
use super::routing;
use aws_wire::transport::{Client, Config};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_engine::inventory::{PageCommit, WorkLease};

async fn test_context(
    fixture: &Fixture,
) -> TestResult<(
    RuntimeContext<'_, '_>,
    Client,
    ApiExecutor,
    ProfileCredentials,
)> {
    let catalog = crate::sweep_runtime::record::policy_dfgk_tests::fixture::catalog()?;
    let consumers = page::consumer_index::ConsumerIndex::build(&catalog)?;
    let executor = ApiExecutor::new(executor_config(4)?)?;
    let client = Client::new(Config::default())?;
    let credentials = ProfileCredentials::load(Some("slice-budget-test"), Some("us-east-1"))
        .await
        .map_err(|e| e.to_string())?;
    let config = SweepConfig {
        region: Some("us-east-1".to_owned()),
        max_pages: 100,
        concurrency: 4,
        resume: true,
        endpoint: None,
        ca_bundle: None,
        slice_timeout_secs: None,
    };
    let account = AccountId::parse("123456789012")?;
    let context = RuntimeContext {
        bootstrap: BootstrapContext {
            catalog: &catalog,
            client: &client,
            executor: &executor,
            credentials: &credentials,
            home: &RegionId::parse("us-east-1")?,
            config: &config,
        },
        store: &fixture.store,
        account,
        lease_ms: fixture.lease_ms,
        endpoints: discovery::EndpointCache::default(),
        consumers,
    };
    Ok((context, client, executor, credentials))
}

#[tokio::test]
async fn given_four_work_items_when_drained_in_two_slices_then_all_complete_after_second_slice()
-> TestResult {
    let fixture = Fixture::new("slice-two-pass").await?;
    // Seed four distinct enumeration scopes (one per account).
    for ordinal in 0..4 {
        let scope = fixture.scope_for(ordinal)?;
        fixture
            .store
            .seed_work(WorkSpec::new(
                fixture.sweep.clone(),
                scope,
                None,
                Vec::new(),
            )?)
            .await?;
    }
    let (context, _, _, _) = test_context(&fixture).await?;
    // First slice: step budget of 2 processes only two items.
    let first = drain_work(&context, &fixture.sweep, None, Some(2)).await?;
    assert_eq!(first.scopes, 2);
    assert_eq!(first.outcomes.complete, 2);
    // Second slice: larger budget processes the remaining two.
    let (context2, _, _, _) = test_context(&fixture).await?;
    let second = drain_work(&context2, &fixture.sweep, None, Some(10)).await?;
    assert_eq!(second.scopes, 2);
    assert_eq!(second.outcomes.complete, 2);
    // Combined: all four scopes completed.
    let total_scopes = first.scopes + second.scopes;
    let total_complete = first.outcomes.complete + second.outcomes.complete;
    assert_eq!(total_scopes, 4);
    assert_eq!(total_complete, 4);
    fixture.store.close().await?;
    Ok(())
}

#[tokio::test]
async fn given_slice_time_budget_when_timeout_reached_then_work_remains_for_next_slice()
-> TestResult {
    let fixture = Fixture::new("slice-timeout").await?;
    // Seed two work items.
    for ordinal in 0..2 {
        let scope = fixture.scope_for(ordinal)?;
        fixture
            .store
            .seed_work(WorkSpec::new(
                fixture.sweep.clone(),
                scope,
                None,
                Vec::new(),
            )?)
            .await?;
    }
    let (context, _, _, _) = test_context(&fixture).await?;
    // Slice with 0ms timeout should process nothing (budget already exhausted).
    let result = drain_work(&context, &fixture.sweep, Some(0), None).await?;
    assert_eq!(result.scopes, 0);
    assert_eq!(result.outcomes.complete, 0);
    // Next slice with generous budget completes both items.
    let (context2, _, _, _) = test_context(&fixture).await?;
    let completed = drain_work(&context2, &fixture.sweep, None, Some(10)).await?;
    assert_eq!(completed.scopes, 2);
    assert_eq!(completed.outcomes.complete, 2);
    fixture.store.close().await?;
    Ok(())
}