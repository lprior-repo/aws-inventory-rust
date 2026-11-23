use super::*;
use crate::{SweepConfig, credentials::TestCredentials};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use sha2::{Digest, Sha256};

struct Environment {
    root: tempfile::TempDir,
    catalog: Vec<u8>,
    peer: peer::Peer,
}

impl Environment {
    async fn new() -> TestResult<Self> {
        let root = tempfile::tempdir()?;
        let catalog = fixture::catalog(root.path())?;
        let peer = peer::Peer::start(root.path()).await?;
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
            slice_timeout_secs: None,
        }
    }
}

#[tokio::test]
async fn control_started_precedes_claims_and_finished_follows_committed_scopes() -> TestResult {
    let environment = Environment::new().await?;
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
            queue(),
            page(),
            ProgressEvent::Scope {
                outcome: ScopeOutcome::Complete
            },
            ProgressEvent::Finished { summary },
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
    let environment = Environment::new().await?;
    let store = environment.store("unread.sqlite").await?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let summary = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        environment.run(&store, false, listener(sender)),
    )
    .await??;
    assert_eq!(summary.rows, 2);
    assert_eq!(summary.metrics.progress_dropped, 6);
    started(&receiver.try_recv()?);
    environment.peer.stop().await
}

#[tokio::test]
async fn control_cancellation_preserves_pending_cursor_and_resume_matches_uninterrupted_digest()
-> TestResult {
    let environment = Environment::new().await?;
    let store = environment.store("cancel.sqlite").await?;
    cancel_after_first_page(&environment, &store).await?;
    let sweep = store
        .discover_active_sweep()
        .await?
        .ok_or("cancelled sweep disappeared")?;
    let pending = store.sweep_summary(&sweep).await?;
    assert_eq!(pending.work_pending, 1);
    assert_eq!(pending.resources_observed, 1);
    assert_eq!(pending.partial_scopes, 0);
    resume_matches_uninterrupted(&environment, &store).await?;
    environment.peer.stop().await
}

async fn resume_matches_uninterrupted(
    environment: &Environment,
    store: &BinaryStore,
) -> TestResult {
    let resumed = environment
        .run(store, true, SweepControl::default())
        .await?;
    let baseline = environment.store("baseline.sqlite").await?;
    let uninterrupted = environment
        .run(&baseline, false, SweepControl::default())
        .await?;
    assert_eq!(resumed.rows, uninterrupted.rows);
    assert_eq!(resumed.outcomes, uninterrupted.outcomes);
    assert_eq!(digest(store).await?, digest(&baseline).await?);
    Ok(())
}

async fn cancel_after_first_page(environment: &Environment, store: &BinaryStore) -> TestResult {
    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    let control = listener(sender);
    let observer = cancel_on_page(receiver, control.cancel.clone());
    let (result, observed) = tokio::join!(environment.run(store, false, control), observer);
    observed?;
    assert_cancelled(result)
}

async fn cancel_on_page(
    mut receiver: tokio::sync::mpsc::Receiver<ProgressEvent>,
    cancel: CancellationToken,
) -> TestResult {
    for _ in 0..16 {
        let event = receiver.recv().await.ok_or("progress ended before page")?;
        if matches!(event, ProgressEvent::Page { .. }) {
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
    let environment = Environment::new().await?;
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
