use super::*;
use cmdb_view::{GraphConnection, GraphLimits, GraphNode, ViewError};

#[tokio::test]
async fn given_graph_disconnect_when_traversal_fails_then_sql_remains_available() -> TestResult {
    let dir = tempdir()?;
    let db = populated(&dir.path().join("inventory.sqlite")).await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let graph =
        GraphConnection::connect(&format!("redis://{}/", listener.local_addr()?), "inventory")?;
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await?;
        drop(socket);
        Ok::<_, std::io::Error>(())
    });
    let root = GraphNode {
        account: "111111111111".to_owned(),
        region: "global".to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: "shared".to_owned(),
    };
    let bounds = GraphLimits {
        depth: 1,
        nodes: 3,
        edges: 2,
    };
    assert!(matches!(
        graph.neighbors(&root, bounds).await,
        Err(ViewError::Backend(inventory_graph::GraphError::Transport(
            _
        )))
    ));
    server.await??;
    let page = db
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 4,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, 4);
    assert_eq!(db.counts().await?.resources, 4);
    assert!(db.detail(&root).await?.is_some());
    Ok(())
}

#[tokio::test]
async fn given_missing_database_when_opening_reader_then_no_file_is_created() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("absent.sqlite");
    assert!(matches!(
        Database::open(path.to_str().ok_or("non-UTF-8 database path")?).await,
        Err(ViewError::Sql(_))
    ));
    assert!(!path.exists());
    Ok(())
}
