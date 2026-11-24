use super::*;

#[tokio::test]
pub(super) async fn graph_limits_reject_before_network_access() -> TestResult {
    let connection = cmdb_view::GraphConnection::connect("redis://127.0.0.1:6379", "inventory")?;
    let node = cmdb_view::GraphNode {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::EC2::Instance".to_owned(),
        id: "i-1".to_owned(),
    };
    let result = connection
        .neighbors(
            &node,
            cmdb_view::GraphLimits {
                depth: 0,
                nodes: 1,
                edges: 1,
            },
        )
        .await;
    assert!(matches!(result, Err(cmdb_view::ViewError::GraphLimits)));
    Ok(())
}

#[tokio::test]
pub(super) async fn sqlite_edges_page_by_full_identity_and_lod_summary_is_exact() -> TestResult {
    let dir = tempdir()?;
    let db = populated(&dir.path().join("inventory.sqlite")).await?;
    let node = cmdb_view::GraphNode {
        account: "111111111111".to_owned(),
        region: "global".to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: "shared".to_owned(),
    };
    let first = db
        .edges_page(
            &node,
            cmdb_view::EdgePageRequest {
                limit: 1,
                after: None,
            },
        )
        .await?;
    let cursor = first.next.clone().ok_or("expected edge cursor")?;
    let second = db
        .edges_page(
            &node,
            cmdb_view::EdgePageRequest {
                limit: 1,
                after: Some(cursor),
            },
        )
        .await?;
    let summary = db.lod_summary().await?;
    assert_eq!(first.total_matching, 2);
    assert_eq!(second.edges.len(), 1);
    assert_eq!(summary.resources, 4);
    assert_eq!(summary.edges, 2);
    assert_eq!(summary.connected_resources, 3);
    assert_eq!(summary.isolated_resources, 1);
    Ok(())
}

pub(super) async fn postgres_fixture() -> TestResult<super::postgres_support::Fixture> {
    super::postgres_support::Fixture::new().await
}
