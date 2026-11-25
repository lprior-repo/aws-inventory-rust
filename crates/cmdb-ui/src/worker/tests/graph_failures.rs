use super::super::*;
use super::isolation::supervised_async;
use super::real::fixtures::{self, Fixture, TestResult, root, snapshot};

fn configured_worker(fixture: &Fixture, endpoint: String, name: &str) -> DataWorker {
    DataWorker::start(
        WorkerConfig {
            database: Some(fixture.path.to_string_lossy().into_owned()),
            graph_url: Some(endpoint),
            graph_name: Some(name.to_owned()),
            ..WorkerConfig::default()
        },
        SignalToUI::default(),
    )
}

async fn assert_terminal_failure(worker: &DataWorker, diagnostic: &str) -> TestResult {
    worker.expand(root())?;
    let event = tokio::time::timeout(Duration::from_secs(5), fixtures::await_event(worker, false))
        .await??;
    let WorkerEvent::Graph { graph, error } = event else {
        return Err(format!("expected terminal graph failure, got {event:?}").into());
    };
    let expected_graph = Graph {
        tag_root: Some(fixtures::digest(&root())?),
        ..Graph::default()
    };
    assert_eq!(graph, expected_graph);
    assert_eq!(error.as_deref(), Some(diagnostic));
    let mut data = crate::data::Data::default();
    data.set_graph(graph, error);
    assert_eq!(data.graph_status(), format!("Graph offline · {diagnostic}"));

    worker.expand(root())?;
    let failure = tokio::time::timeout(Duration::from_secs(5), fixtures::graph(worker))
        .await?
        .err()
        .ok_or("expected graph helper to report the terminal failure")?;
    assert_eq!(
        failure.to_string(),
        format!(
            "expected successful real graph, got Graph {{ graph: {:?}, error: Some({diagnostic:?}) }}",
            expected_graph
        )
    );
    Ok(())
}

#[test]
fn given_invalid_graph_configuration_when_expanding_then_reconnection_failure_is_terminal()
-> TestResult {
    supervised_async(|| async {
        let fixture = Fixture::new().await?;
        let worker = configured_worker(&fixture, "redis://127.0.0.1:6379".to_owned(), "");
        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!(counts, Counts::default());
        assert_eq!(rows, Vec::<Row>::new());
        assert_eq!(total, 0);

        assert_terminal_failure(&worker, "invalid graph configuration · graph name").await
    })
}

#[test]
fn given_unreachable_graph_when_expanding_then_terminal_failure_reaches_graph_helper() -> TestResult
{
    supervised_async(|| async {
        let socket = tokio::net::TcpSocket::new_v4()?;
        socket.bind("127.0.0.1:0".parse()?)?;
        let endpoint = format!("redis://{}", socket.local_addr()?);
        let client = redis::Client::open(endpoint.as_str())?;
        let source = tokio::time::timeout(
            Duration::from_secs(5),
            client.get_multiplexed_async_connection(),
        )
        .await?
        .err()
        .ok_or("expected the closed graph endpoint to refuse connections")?;
        let diagnostic = format!("connection/query unavailable · {source}");
        let fixture = Fixture::new().await?;
        let worker = configured_worker(&fixture, endpoint, "inventory");
        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!(counts, Counts::default());
        assert_eq!(rows, Vec::<Row>::new());
        assert_eq!(total, 0);

        assert_terminal_failure(&worker, &diagnostic).await
    })
}

#[test]
fn given_worker_without_graph_when_starting_then_first_snapshot_is_not_a_graph_failure()
-> TestResult {
    supervised_async(|| async {
        let fixture = Fixture::new().await?;
        let expected = fixture
            .commit(
                "without-graph",
                fixtures::ACCOUNT,
                fixtures::REGION,
                &[("shared", "SQL inventory remains available")],
                &[],
                1,
            )
            .await?;
        let worker = fixture.worker(None);

        let (counts, rows, total) = snapshot(&worker).await?;
        assert_eq!(
            counts,
            Counts {
                accounts: 1,
                regions: 1,
                services: 1,
                resources: 1,
            }
        );
        assert_eq!(rows, expected);
        assert_eq!(total, 1);
        Ok(())
    })
}

#[test]
fn given_unconsumed_refresh_output_when_enqueuing_next_command_then_leftover_is_named() -> TestResult
{
    supervised_async(|| async {
        let expected_graph = Graph {
            tag_root: Some(fixtures::digest(&root())?),
            ..Graph::default()
        };
        for consumed_snapshot in [true, false] {
            for next_refresh in [false, true] {
                let fixture = Fixture::new().await?;
                let worker = fixture.worker(None);
                assert_eq!(snapshot(&worker).await?, (Counts::default(), Vec::new(), 0));
                worker.expand(root())?;
                let initial = fixtures::await_event(&worker, false).await?;
                assert!(matches!(
                    initial,
                    WorkerEvent::Graph { graph, error: Some(error) }
                        if graph == expected_graph
                            && error == "not configured (--graph-url and --graph-name)"
                ));
                worker.refresh(Filter::default())?;
                let consumed = fixtures::await_event(&worker, consumed_snapshot).await?;
                assert_eq!(
                    matches!(consumed, WorkerEvent::Snapshot { .. }),
                    consumed_snapshot
                );
                let result = if next_refresh {
                    worker.refresh(Filter::default())
                } else {
                    worker.expand(root())
                };
                let error = result
                    .err()
                    .ok_or("expected an unconsumed output to reject the command")?;
                let leftover = if consumed_snapshot {
                    "Graph"
                } else {
                    "Snapshot"
                };
                assert_eq!(
                    error,
                    format!("real worker command would abandon unconsumed {leftover}")
                );
            }
        }
        Ok(())
    })
}
