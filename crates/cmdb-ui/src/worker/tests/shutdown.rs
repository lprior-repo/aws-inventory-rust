use super::isolation::{supervised, supervised_async};
use super::real::fixtures::{ACCOUNT, Fixture, REGION, TestResult, root, snapshot};
use super::support::{BlockedGraph, close_worker, queued};
use super::*;

async fn writer(fixture: &Fixture) -> TestResult<sqlx::SqlitePool> {
    Ok(sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&fixture.path)
                .busy_timeout(Duration::from_millis(100)),
        )
        .await?)
}

async fn checkpoint(pool: &sqlx::SqlitePool) -> TestResult<(i64, i64, i64)> {
    Ok(sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
        .fetch_one(pool)
        .await?)
}

#[test]
fn given_real_blocked_graph_query_when_worker_closes_then_query_socket_and_thread_are_released()
-> TestResult {
    supervised_async(|| async {
        // Given a real engine database and a peer acknowledging an actual query.
        let fixture = Fixture::new().await?;
        fixture
            .commit("seed", ACCOUNT, REGION, &[("shared", "retained")], &[], 1)
            .await?;
        let peer = BlockedGraph::start()?;
        let worker = DataWorker::start(peer.config(fixture.path.clone()), SignalToUI::default());
        let (_, rows, _) = snapshot(&worker).await?;
        assert_eq!(rows.first().map(|row| row.name.as_str()), Some("retained"));
        worker.expand(root())?;
        let request = peer.await_query()?;
        assert_eq!(request.first().map(String::as_str), Some("GRAPH.RO_QUERY"));
        assert!(request.get(2).is_some_and(|query| query.contains("MATCH")));
        // When close happens before the peer replies (the normal timeout is 10s).
        close_worker(worker)?;
        // Then both cancellation/join and transport EOF must finish within 2s.
        peer.finish()?;
        Ok(())
    })
}

/// UI-02: No retained snapshot. Each page read opens and closes its own snapshot.
/// A writer checkpoint must succeed immediately after a read completes, because
/// no long-lived reader holds the database open.
#[test]
fn given_idle_real_worker_with_short_lived_snapshot_when_closed_then_no_reader_survives()
-> TestResult {
    supervised_async(|| async {
        // Given a worker that performed a page read (which opens and closes its own snapshot).
        let fixture = Fixture::new().await?;
        fixture
            .commit("seed", ACCOUNT, REGION, &[("shared", "before")], &[], 1)
            .await?;
        let worker = fixture.worker(None);
        snapshot(&worker).await?;
        // The read completed; its snapshot is closed. A writer checkpoint must succeed
        // immediately, proving no retained snapshot is blocking it.
        let expected = fixture
            .commit("later", ACCOUNT, REGION, &[("shared", "after")], &[], 2)
            .await?;
        let writer = writer(&fixture).await?;
        assert_eq!(
            checkpoint(&writer).await?,
            (0, 0, 0),
            "no retained snapshot should block truncation"
        );
        // When the real idle intake is cancelled and joined, no reader survives.
        close_worker(worker)?;
        assert_eq!(checkpoint(&writer).await?, (0, 0, 0));
        // Then writer/fresh reader remain functional after cleanup.
        let fresh = fixture.worker(None);
        assert_eq!(snapshot(&fresh).await?.1, expected);
        close_worker(fresh)?;
        writer.close().await;
        Ok(())
    })
}

/// UI-02: No retained snapshot. The worker thread performs a read, sends the result,
/// and closes its snapshot. A writer checkpoint must succeed immediately after the
/// read completes, because no long-lived reader holds the database open.
#[test]
fn given_real_runner_blocked_on_output_when_closed_then_join_and_no_retained_reader_complete()
-> TestResult {
    supervised_async(|| async {
        // Given the actual runner/startup and a rendezvous output channel. Receiving
        // graph status lets startup proceed to its snapshot send with no consumer.
        let fixture = Fixture::new().await?;
        fixture
            .commit("seed", ACCOUNT, REGION, &[("shared", "before")], &[], 1)
            .await?;
        let queued = queued(
            WorkerConfig {
                database: Some(fixture.path.clone()),
                graph_limits: DEFAULT_GRAPH_LIMITS,
                ..WorkerConfig::default()
            },
            1,
            0,
        );
        let worker = queued.release()?;
        let first = worker
            .events
            .as_ref()
            .ok_or("missing output")?
            .recv_timeout(Duration::from_secs(2))?;
        assert!(matches!(first.event, WorkerEvent::GraphStatus { .. }));
        fixture
            .commit("later", ACCOUNT, REGION, &[("shared", "after")], &[], 2)
            .await?;
        let writer = writer(&fixture).await?;
        // No retained snapshot: checkpoint succeeds immediately after read completes.
        assert_eq!(checkpoint(&writer).await?, (0, 0, 0));
        assert!(
            !worker
                .thread
                .as_ref()
                .ok_or("missing worker thread")?
                .is_finished()
        );
        // When output receiver disconnects before join, the production send unblocks.
        close_worker(worker)?;
        // Then retained SQL state was finalized, not merely an unowned thread exited.
        assert_eq!(checkpoint(&writer).await?, (0, 0, 0));
        writer.close().await;
        Ok(())
    })
}

#[test]
fn given_advanced_revision_when_real_startup_fails_then_both_terminal_events_are_unversioned()
-> TestResult {
    supervised(|| {
        // Given intake revisions already moved before the actual runner starts.
        let queued = queued(WorkerConfig::default(), 2, EVENT_CAPACITY);
        queued.worker.refresh(Filter {
            query: "already accepted".into(),
            ..Filter::default()
        })?;
        let worker = queued.release()?;
        let receiver = worker.events.as_ref().ok_or("missing output")?;
        let graph = receiver.recv_timeout(Duration::from_secs(2))?;
        let page = receiver.recv_timeout(Duration::from_secs(2))?;
        assert_eq!((graph.revision, page.revision), (None, None));
        match graph.event {
            WorkerEvent::Graph { graph, error } => {
                assert_eq!(graph, Graph::default());
                assert_eq!(
                    error.as_deref(),
                    Some("database unavailable · missing --database PATH")
                );
            }
            other => return Err(format!("expected startup graph, got {other:?}").into()),
        }
        match page.event {
            WorkerEvent::Snapshot {
                counts,
                rows,
                total,
                error,
            } => {
                assert_eq!((counts, rows, total), (Counts::default(), Vec::new(), 0));
                assert_eq!(error.as_deref(), Some("missing --database PATH"));
            }
            other => return Err(format!("expected startup snapshot, got {other:?}").into()),
        }
        // When failure reporting finishes, real intake closes and the thread joins.
        let intake = worker.commands.clone();
        close_worker(worker)?;
        assert!(
            intake.is_closed(),
            "startup failure must close actual command intake"
        );
        Ok(())
    })
}

#[test]
fn given_missing_path_or_invalid_schema_when_real_worker_starts_then_database_diagnostic_survives()
-> TestResult {
    supervised_async(|| async {
        let directory = super::isolation::temporary_directory()?;
        let invalid = directory.path().join("invalid.sqlite");
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&invalid)
                    .create_if_missing(true),
            )
            .await?;
        sqlx::query("CREATE TABLE unrelated(value TEXT)")
            .execute(&pool)
            .await?;
        pool.close().await;
        for (path, diagnostic) in [
            (
                directory.path().join("missing.sqlite"),
                "unable to open database file",
            ),
            (
                invalid,
                "unsupported schema version 0; expected 15 or 16 (open with the writer to migrate schema 14)",
            ),
        ] {
            let worker = DataWorker::start(
                WorkerConfig {
                    database: Some(path),
                    graph_limits: DEFAULT_GRAPH_LIMITS,
                    ..WorkerConfig::default()
                },
                SignalToUI::default(),
            );
            let event = super::real::fixtures::await_event(&worker, true).await?;
            match event {
                WorkerEvent::Snapshot {
                    counts,
                    rows,
                    total,
                    error,
                } => {
                    assert_eq!((counts, rows, total), (Counts::default(), Vec::new(), 0));
                    let error = error.ok_or("real startup failure omitted its diagnostic")?;
                    assert!(
                        error.contains(diagnostic),
                        "expected {diagnostic:?}, got {error:?}"
                    );
                }
                other => return Err(format!("expected actual startup failure: {other:?}").into()),
            }
            close_worker(worker)?;
        }
        Ok(())
    })
}
