use super::*;
use crate::{SweepConfig, credentials::TestCredentials};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use sha2::{Digest, Sha256};
use std::time::Duration;

const RUN_TIMEOUT: Duration = Duration::from_secs(20);

struct Environment {
    root: tempfile::TempDir,
    catalog: Vec<u8>,
    peer: peer::Peer,
}

impl Environment {
    async fn new(mode: peer::Mode) -> TestResult<Self> {
        let root = tempfile::tempdir()?;
        let catalog = fixture::catalog(root.path(), false)?;
        let peer = peer::Peer::start(root.path(), mode).await?;
        Ok(Self {
            root,
            catalog,
            peer,
        })
    }

    async fn store(&self, name: &str) -> TestResult<BinaryStore> {
        let catalog = RuntimeCatalog::from_bytes(&self.catalog)?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        Ok(BinaryStore::create(&self.root.path().join(name), registry).await?)
    }

    async fn run(
        &self,
        store: &BinaryStore,
        resume: bool,
        control: SweepControl,
    ) -> TestResult<SweepSummary> {
        let catalog = RuntimeCatalog::from_bytes(&self.catalog)?;
        Ok(crate::run(
            &catalog,
            store,
            self.config(resume),
            &TestCredentials::new("us-east-1"),
            control,
        )
        .await?)
    }

    fn config(&self, resume: bool) -> SweepConfig {
        SweepConfig {
            region: Some("us-east-1".to_owned()),
            max_pages: 10,
            concurrency: 1,
            resume,
            endpoint: Some(self.peer.endpoint.clone()),
            ca_bundle: Some(self.peer.ca.clone()),
            idle_timeout: None,
            pool_idle_per_host: None,
            slice_timeout_secs: None,
            max_rps: None,
            max_rps_milli: None,
            max_page_size: None,
        }
    }
}

#[tokio::test]
async fn control_started_precedes_claims_and_finished_follows_committed_scopes() -> TestResult {
    let environment = Environment::new(peer::Mode::Normal).await?;
    let store = environment.store("ordered.sqlite").await?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
    let summary = environment.run(&store, false, listener(sender)).await?;
    let events = std::iter::from_fn(|| receiver.try_recv().ok())
        .take(32)
        .collect::<Vec<_>>();
    assert_eq!(summary.rows, 2);
    assert_eq!(summary.outcomes.complete, 1);
    assert_eq!(summary.metrics.progress_dropped, 0);
    started(events.first().ok_or("Started absent")?);
    ordered_tail(events.get(1..).ok_or("events absent")?, summary);
    environment.peer.stop().await
}

fn started(event: &ProgressEvent) {
    assert!(matches!(event, ProgressEvent::Started {
        account, regions: 1, root_scopes: 1, ..
    } if account == "123456789012"));
}

fn ordered_tail(events: &[ProgressEvent], summary: SweepSummary) {
    assert_eq!(
        events,
        &[
            queue(),
            page(),
            page(),
            ProgressEvent::Scope {
                outcome: ScopeOutcome::Complete
            },
            ProgressEvent::Finished {
                summary: Box::new(summary),
            },
        ]
    );
}

fn queue() -> ProgressEvent {
    ProgressEvent::Queue {
        pending: 0,
        in_flight: 1,
    }
}

fn page() -> ProgressEvent {
    ProgressEvent::Page {
        rows: 1,
        resource_writes: 1,
    }
}

fn listener(sender: tokio::sync::mpsc::Sender<ProgressEvent>) -> SweepControl {
    SweepControl {
        progress: Some(sender),
        ..SweepControl::default()
    }
}

#[tokio::test]
async fn control_unread_capacity_one_never_blocks_collection() -> TestResult {
    let environment = Environment::new(peer::Mode::Normal).await?;
    let store = environment.store("unread.sqlite").await?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let summary = tokio::time::timeout(
        RUN_TIMEOUT,
        environment.run(&store, false, listener(sender)),
    )
    .await??;
    assert_eq!(summary.rows, 2);
    assert_eq!(summary.metrics.progress_dropped, 5);
    started(&receiver.try_recv()?);
    environment.peer.stop().await
}

#[tokio::test]
async fn control_cancellation_preserves_pending_cursor_and_resume_matches_uninterrupted_digest()
-> TestResult {
    let pause = std::sync::Arc::new(peer::ContinuationPause::default());
    let environment = Environment::new(peer::Mode::PauseFirstContinuation(pause.clone())).await?;
    let store = environment.store("cancel.sqlite").await?;
    cancel_after_first_page(&environment, &store, &pause).await?;
    let sweep = store
        .discover_active_sweep()
        .await?
        .ok_or("cancelled sweep disappeared")?;
    let pending = store.sweep_summary(&sweep).await?;
    assert_eq!(pending.work_pending, 1);
    assert_eq!(pending.resources_observed, 1);
    assert_eq!(pending.partial_scopes, 0);
    resume_matches_uninterrupted(&environment, &store, &pause).await?;
    environment.peer.stop().await
}

async fn resume_matches_uninterrupted(
    environment: &Environment,
    store: &BinaryStore,
    pause: &peer::ContinuationPause,
) -> TestResult {
    let resumed = tokio::time::timeout(
        RUN_TIMEOUT,
        environment.run(store, true, SweepControl::default()),
    )
    .await??;
    assert_eq!(
        pause.take_item_queries().await,
        [Some(String::from("marker=next"))],
        "resume must fetch the persisted continuation without replaying the first page",
    );
    let baseline = environment.store("baseline.sqlite").await?;
    let uninterrupted = tokio::time::timeout(
        RUN_TIMEOUT,
        environment.run(&baseline, false, SweepControl::default()),
    )
    .await??;
    assert_eq!(resumed.rows, uninterrupted.rows);
    assert_eq!(resumed.outcomes, uninterrupted.outcomes);
    assert_eq!(digest(store).await?, digest(&baseline).await?);
    Ok(())
}

async fn cancel_after_first_page(
    environment: &Environment,
    store: &BinaryStore,
    pause: &peer::ContinuationPause,
) -> TestResult {
    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    let control = listener(sender);
    let observer = cancel_on_page(receiver, control.cancel.clone(), pause);
    let run = tokio::time::timeout(RUN_TIMEOUT, environment.run(store, false, control));
    let (result, observed) = tokio::join!(run, observer);
    pause.release.cancel();
    observed?;
    assert_cancelled(result?)?;
    assert_eq!(
        pause.take_item_queries().await,
        [None, Some(String::from("marker=next"))],
        "cancellation must interrupt the continuation request",
    );
    Ok(())
}

async fn cancel_on_page(
    mut receiver: tokio::sync::mpsc::Receiver<ProgressEvent>,
    cancel: CancellationToken,
    pause: &peer::ContinuationPause,
) -> TestResult {
    for _ in 0..16 {
        let event = receiver.recv().await.ok_or("progress ended before page")?;
        if matches!(event, ProgressEvent::Page { .. }) {
            tokio::time::timeout(
                std::time::Duration::from_secs(10),
                pause.requested.notified(),
            )
            .await?;
            cancel.cancel();
            return Ok(());
        }
    }
    Err("page event exceeded fixed event budget".into())
}

async fn digest(store: &BinaryStore) -> TestResult<[u8; 32]> {
    let resources = store.projection_resources(None, 16).await?;
    assert_eq!(resources.len(), 2);
    let hash = resources.iter().fold(Sha256::new(), |mut hash, resource| {
        hash.update(resource.key.digest().as_bytes());
        hash.update(resource.content_hash);
        hash
    });
    Ok(hash.finalize().into())
}

#[tokio::test]
async fn control_pre_cancelled_bootstrap_does_not_create_a_sweep() -> TestResult {
    let environment = Environment::new(peer::Mode::Normal).await?;
    let store = environment.store("pre-cancel.sqlite").await?;
    let control = SweepControl::default();
    control.cancel.cancel();
    assert_cancelled(environment.run(&store, false, control).await)?;
    assert!(store.discover_active_sweep().await?.is_none());
    environment.peer.stop().await
}

fn assert_cancelled(result: TestResult<SweepSummary>) -> TestResult {
    let error = result.err().ok_or("cancellation ignored")?;
    let error = error
        .downcast_ref::<crate::SweepError>()
        .ok_or("cancellation error type lost")?;
    assert!(matches!(
        error,
        crate::SweepError::Api(inventory_engine::limiter::ApiError::Cancelled)
    ));
    Ok(())
}

#[tokio::test]
async fn metrics_report_real_decode_claim_commit_executor_and_transport_activity() -> TestResult {
    let environment = Environment::new(peer::Mode::Normal).await?;
    let store = environment.store("metrics.sqlite").await?;
    let summary = environment
        .run(&store, false, SweepControl::default())
        .await?;
    let metrics = summary.metrics;
    assert_eq!((metrics.api_attempts, metrics.requests_sent), (4, 4));
    assert_eq!(
        (metrics.pages, metrics.store_commits, metrics.claims),
        (2, 2, 1)
    );
    assert_eq!((metrics.api_retries, metrics.api_throttles), (0, 0));
    assert_eq!((metrics.http2_requests, metrics.requests_reused), (0, 0));
    assert_eq!(metrics.connections_opened, 4);
    assert!(metrics.bytes_read > 0);
    assert!(metrics.decode_ns > 0 && metrics.store_commit_ns > 0);
    if metrics.process_metrics_available {
        assert!(metrics.peak_rss_kb > 0);
    }
    environment.peer.stop().await
}

#[tokio::test]
async fn given_sweep_when_requests_are_sent_then_attempted_actions_are_in_generated_policy()
-> TestResult {
    let mut environment = Environment::new(peer::Mode::Normal).await?;
    environment.catalog = fixture::catalog(environment.root.path(), true)?;
    let catalog = RuntimeCatalog::from_bytes(&environment.catalog)?;
    assert_eq!(catalog.plans().len(), 4);
    let attempts = std::sync::Arc::new(crate::AttemptedOperations::new(&catalog));
    assert_eq!(attempts.iter().count(), 0);
    let reference = "# Source: fixture\n# Retrieved: 2026-11-24\ncontrolinventory\tGetCallerIdentity\tread\ncontrolinventory\tListResources\tread\ncontrolinventory\tListUnused\tread\n";
    let mappings =
        "[operation]\n\"controlinventory/ListItems\" = \"controlinventory:ListResources\"\n";
    let gaps = "controlinventory\tDescribeRegions\tcontrolinventory\tunconfirmed: absent from the service authorization reference at 2026-11-24\n";
    let policy = catalog_tool::iam_policy::render(
        &catalog_tool::iam_policy::operations(&catalog)?,
        mappings,
        reference,
        gaps,
    )?;
    let store = environment.store("attempts.sqlite").await?;
    let summary = environment
        .run(
            &store,
            false,
            SweepControl {
                attempts: Some(attempts.clone()),
                ..SweepControl::default()
            },
        )
        .await?;
    assert_eq!(summary.rows, 2);
    assert_eq!(summary.metrics.api_attempts, 4);
    assert_eq!(
        attempts.iter().collect::<Vec<_>>(),
        vec![
            ("controlinventory", "DescribeRegions"),
            ("controlinventory", "GetCallerIdentity"),
            ("controlinventory", "ListItems"),
        ]
    );
    assert!(
        !attempts
            .iter()
            .any(|(_, operation)| operation == "ListUnused")
    );
    assert_attempt_coverage(&attempts, &policy)?;
    environment.peer.stop().await
}

fn assert_attempt_coverage(
    attempts: &crate::AttemptedOperations,
    policy: &catalog_tool::iam_policy::Rendered,
) -> TestResult {
    let required = attempts
        .iter()
        .map(|(service, operation)| {
            policy
                .operation_actions
                .get(&format!("{service}/{operation}"))
                .cloned()
                .ok_or("attempted action absent")
        })
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    assert_eq!(
        required,
        std::collections::BTreeSet::from([
            "controlinventory:DescribeRegions".to_owned(),
            "controlinventory:GetCallerIdentity".to_owned(),
            "controlinventory:ListResources".to_owned(),
        ])
    );
    assert!(required.is_subset(&policy.actions));
    assert_eq!(
        policy.unconfirmed,
        std::collections::BTreeSet::from(["controlinventory/DescribeRegions".to_owned()])
    );
    Ok(())
}

#[tokio::test]
async fn given_wrong_attempt_table_when_sweep_signs_then_unknown_operation_is_an_error()
-> TestResult {
    let environment = Environment::new(peer::Mode::Normal).await?;
    let unrelated =
        crate::record::policy_dfgk_tests::fixture::catalog().map_err(|error| error.to_string())?;
    let attempts = std::sync::Arc::new(crate::AttemptedOperations::new(&unrelated));
    let store = environment.store("wrong-attempt-table.sqlite").await?;
    let error = environment
        .run(
            &store,
            false,
            SweepControl {
                attempts: Some(attempts.clone()),
                ..SweepControl::default()
            },
        )
        .await
        .err()
        .ok_or("unknown attempt silently discarded")?;
    assert!(
        error
            .to_string()
            .contains("attempted operation absent from receipt table")
    );
    assert_eq!(attempts.iter().count(), 0);
    environment.peer.stop().await
}

#[tokio::test]
async fn given_rejected_and_retried_calls_when_responses_are_blocked_then_attempts_are_already_recorded()
-> TestResult {
    let failures = std::sync::Arc::new(peer::Rejections::default());
    let environment = Environment::new(peer::Mode::Reject(failures.clone())).await?;
    let catalog = RuntimeCatalog::from_bytes(&environment.catalog)?;
    let attempts = std::sync::Arc::new(crate::AttemptedOperations::new(&catalog));
    let policy = catalog_tool::iam_policy::render(
        &catalog_tool::iam_policy::operations(&catalog)?,
        "[operation]\n\"controlinventory/ListItems\" = \"controlinventory:ListResources\"\n",
        "# Retrieved: 2026-11-24\ncontrolinventory\tGetCallerIdentity\tread\ncontrolinventory\tListResources\tread\n",
        "controlinventory\tDescribeRegions\tcontrolinventory\tunconfirmed: absent from the service authorization reference at 2026-11-24\n",
    )?;
    let receipt = attempts.clone();
    let worker = tokio::spawn(async move {
        let store = environment.store("rejected-attempts.sqlite").await?;
        let summary = environment
            .run(
                &store,
                false,
                SweepControl {
                    attempts: Some(receipt),
                    ..SweepControl::default()
                },
            )
            .await?;
        environment.peer.stop().await?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(summary)
    });
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        failures.requested.notified(),
    )
    .await?;
    let recorded_before_response = attempts.iter().collect::<Vec<_>>();
    failures.release.notify_one();
    let summary = worker.await??;
    assert_eq!(summary.rows, 0);
    assert_eq!(summary.outcomes.inaccessible, 1);
    assert_eq!(summary.metrics.api_attempts, 4);
    let retried = failures
        .retry_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(retried, 2);
    let expected = vec![
        ("controlinventory", "DescribeRegions"),
        ("controlinventory", "GetCallerIdentity"),
        ("controlinventory", "ListItems"),
    ];
    assert_eq!(recorded_before_response, expected);
    assert_eq!(attempts.iter().collect::<Vec<_>>(), expected);
    assert_attempt_coverage(&attempts, &policy)?;
    Ok(())
}
