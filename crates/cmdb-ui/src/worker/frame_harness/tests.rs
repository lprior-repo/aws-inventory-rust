use crate::worker::tests::isolation::supervised_async;
use crate::worker::tests::real::fixtures::{ACCOUNT, Fixture, REGION, TestResult, edge, root};
use crate::worker::{WorkerConfig, WorkerEvent};

async fn fixture() -> TestResult<Fixture> {
    let fixture = Fixture::new().await?;
    let from = root();
    let mut to = root();
    to.id = "child".to_owned();
    let link = edge(&from, &to, "references", "reference");
    fixture
        .commit(
            "frame-harness",
            ACCOUNT,
            REGION,
            &[("shared", "Root"), ("child", "Child")],
            &[link],
            1,
        )
        .await?;
    Ok(fixture)
}

#[test]
fn given_inventory_database_when_harness_starts_then_real_rows_and_edges_reach_dag() -> TestResult {
    supervised_async(|| async {
        let fixture = fixture().await?;
        let config = WorkerConfig {
            database: Some(fixture.path.to_string_lossy().into_owned()),
            frame_harness: Some(crate::frame_harness::FrameConfig::parse("20")?),
            ..WorkerConfig::default()
        };
        let (_, event) =
            super::super::setup(fixture.path.to_string_lossy().into_owned(), config).await?;
        assert_graph(event)
    })
}

fn assert_graph(event: WorkerEvent) -> TestResult {
    let WorkerEvent::HarnessGraph {
        graph,
        resources,
        edges,
    } = event
    else {
        return Err("startup did not deliver a harness graph".into());
    };
    assert_eq!(resources, 2);
    assert_eq!(edges, 1);
    assert_eq!(graph.nodes().len(), 2);
    assert_eq!(graph.edges().len(), 1);
    assert!(graph.nodes().iter().any(|node| node.label() == "Root"));
    assert!(graph.nodes().iter().any(|node| node.label() == "Child"));
    Ok(())
}
