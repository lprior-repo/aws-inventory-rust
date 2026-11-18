use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn node(id: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: id.to_owned(),
    }
}

fn edge(from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: node(from),
        to: node(to),
        relation: "depends_on".to_owned(),
        provenance: "model".to_owned(),
    }
}

#[test]
fn given_cycle_and_shortcut_when_finding_path_then_shortest_edges_keep_stored_direction()
-> TestResult {
    let edges = vec![
        edge("a", "b"),
        edge("b", "c"),
        edge("c", "a"),
        edge("d", "c"),
    ];
    let path = shortest(&edges, &node("a"), &node("d"), 3)?.ok_or("missing path")?;
    assert_eq!(path, vec![edge("c", "a"), edge("d", "c")]);
    Ok(())
}

#[test]
fn given_parallel_provenances_when_finding_path_then_nodes_are_visited_once() -> TestResult {
    let mut reviewed = edge("a", "b");
    reviewed.provenance = "reviewed".to_owned();
    let edges = vec![edge("a", "b"), reviewed, edge("b", "c")];
    assert_eq!(
        shortest(&edges, &node("a"), &node("c"), 2)?,
        Some(vec![edge("a", "b"), edge("b", "c")])
    );
    Ok(())
}

#[test]
fn given_depth_budget_when_target_is_farther_then_no_partial_path_is_returned() -> TestResult {
    let edges = vec![edge("a", "b"), edge("b", "c")];
    assert!(shortest(&edges, &node("a"), &node("c"), 1)?.is_none());
    assert!(shortest(&edges, &node("a"), &node("missing"), 3)?.is_none());
    assert_eq!(
        shortest(&edges, &node("a"), &node("a"), 3)?,
        Some(Vec::new())
    );
    Ok(())
}

#[test]
fn given_shared_id_when_target_has_another_account_then_identity_does_not_collapse() -> TestResult {
    let edges = vec![edge("a", "b")];
    let mut other = node("b");
    other.account = "222222222222".to_owned();
    assert!(shortest(&edges, &node("a"), &other, 1)?.is_none());
    Ok(())
}
