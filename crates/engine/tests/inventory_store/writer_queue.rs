use super::support::{TestResult, catalog_bytes, page, scope};
use futures_util::future::join_all;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec};
use std::{path::Path, task::Poll, time::Duration};

const REGIONS: [&str; 16] = [
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    "ca-central-1",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "eu-central-1",
    "eu-north-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-south-1",
    "sa-east-1",
];

#[tokio::test]
async fn given_a_held_sqlite_writer_when_concurrent_page_commits_wait_past_five_seconds_then_all_commit()
-> TestResult {
    let catalog = catalog_bytes()?;
    let runtime = RuntimeCatalog::from_bytes(&catalog)?;
    let registry = SchemaRegistry::new(&catalog, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(Path::new(":memory:"), registry).await?;
    let sweep = SweepId::new("writer-queue".to_owned())?;
    let works = REGIONS
        .iter()
        .map(|region| {
            Ok(WorkSpec::new(
                sweep.clone(),
                scope(region)?,
                None,
                Vec::new(),
            )?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    store.seed_works(&works).await?;
    let mut pages = Vec::with_capacity(works.len());
    for _ in &works {
        let lease = store.claim(&sweep, 100, 20).await?.ok_or("missing lease")?;
        pages.push(page(lease, Vec::new(), true, None, 101));
    }
    let snapshot = store.begin_projection_snapshot().await?;

    let commits = join_all(pages.into_iter().map(|page| store.commit_page(page)));
    tokio::pin!(commits);
    tokio::time::pause();
    assert!(futures_util::poll!(&mut commits).is_pending());
    tokio::time::advance(Duration::from_secs(6)).await;
    if let Poll::Ready(results) = futures_util::poll!(&mut commits) {
        results.into_iter().collect::<Result<Vec<_>, _>>()?;
        return Err("commits did not wait for the writer".into());
    }
    tokio::time::resume();
    snapshot.finish().await?;

    let committed = commits.await.into_iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(committed.len(), works.len());
    for result in committed {
        assert_eq!(result.next_generation, Some(1));
        assert!(!result.repeated_token);
    }
    assert!(store.claim(&sweep, 121, 20).await?.is_none());
    Ok(())
}
