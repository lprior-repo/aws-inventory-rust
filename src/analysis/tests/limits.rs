use super::*;

#[tokio::test]
async fn graph_rejects_bad_caps_running_scans_and_handles_quoted_ids() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "quoted'scan",
        vec![resource("a' OR 1=1 --", "fixture", json!({}))],
        vec![],
        ScanStatus::Partial,
    )
    .await?;
    let result = graph::report(
        &store,
        "quoted'scan",
        "a' OR 1=1 --",
        None,
        options(Direction::Both, 1),
    )
    .await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(rows(&result, "nodes")?.len(), 1);
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            options(Direction::Both, 17)
        )
        .await
        .is_err()
    );
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            TraversalOptions {
                max_nodes: 0,
                ..options(Direction::Both, 1)
            }
        )
        .await
        .is_err()
    );
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            TraversalOptions {
                max_edges: 4001,
                ..options(Direction::Both, 1)
            }
        )
        .await
        .is_err()
    );
    store.begin_scan("running", 1).await?;
    assert!(
        graph::report(&store, "running", "a", None, options(Direction::Both, 1))
            .await
            .is_err()
    );
    store.close().await;
    Ok(())
}
