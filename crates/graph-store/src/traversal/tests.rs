use super::*;
use crate::Provenance;
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn key(id: &str) -> Result<NodeKey, Box<dyn std::error::Error>> {
    Ok(NodeKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: Kind::parse("AWS::EC2::Vpc")?,
        id: ResourceId::parse(id)?,
    })
}

fn edge(from: &NodeKey, to: &NodeKey) -> Result<EdgeRow, Box<dyn std::error::Error>> {
    Ok(EdgeRow {
        from: from.clone(),
        to: to.clone(),
        relation: Relation::parse("CONTAINS")?,
        provenance: Provenance::parse("model")?,
    })
}

#[test]
fn node_and_edge_budgets_mark_truncation_without_overfilling() -> TestResult {
    let root = key("root")?;
    let child = key("child")?;
    let limits = GraphLimits::new(2, 1, 1)?;
    let mut state = Traversal::new(&root, limits)?;
    state.add_layer(vec![edge(&root, &child)?])?;
    assert_eq!(state.truncation, Truncation::Budget);
    assert!(state.edges.is_empty());
    assert_eq!(state.visited, vec![root.clone()]);
    let mut state = Traversal::new(&root, GraphLimits::new(2, 3, 1)?)?;
    state.add_layer(vec![edge(&root, &child)?, edge(&root, &key("other")?)?])?;
    assert_eq!(state.truncation, Truncation::Budget);
    assert_eq!(state.edges.len(), 1);
    Ok(())
}

#[test]
fn cycle_parallel_and_self_edges_preserve_keys_without_duplicate_frontiers() -> TestResult {
    let root = key("root")?;
    let child = key("child")?;
    let limits = GraphLimits::new(3, 5, 5)?;
    let mut state = Traversal::new(&root, limits)?;
    let link = edge(&root, &child)?;
    state.add_layer(vec![link.clone(), link, edge(&root, &root)?])?;
    assert_eq!(state.edges.len(), 2);
    assert_eq!(state.frontier, vec![child]);
    assert_eq!(state.truncation, Truncation::Complete);
    Ok(())
}

#[test]
fn shortest_path_obeys_depth_and_does_not_claim_absence_after_truncation() -> TestResult {
    let a = key("a")?;
    let b = key("b")?;
    let c = key("c")?;
    let ab = edge(&a, &b)?;
    let bc = edge(&b, &c)?;
    let neighborhood = GraphNeighborhood {
        edges: vec![bc.clone(), ab.clone()],
        truncation: Truncation::Complete,
    };
    assert_eq!(
        shortest(&neighborhood, &a, &c, GraphLimits::new(2, 3, 2)?)?,
        PathResult::Path(vec![ab, bc])
    );
    assert_eq!(
        shortest(&neighborhood, &a, &c, GraphLimits::new(1, 3, 2)?)?,
        PathResult::NotFound
    );
    Ok(())
}

#[test]
fn truncated_search_does_not_claim_absence() -> TestResult {
    let truncated = GraphNeighborhood {
        edges: vec![],
        truncation: Truncation::Budget,
    };
    assert!(matches!(
        shortest(
            &truncated,
            &key("a")?,
            &key("c")?,
            GraphLimits::new(2, 3, 2)?
        ),
        Err(GraphError::Truncated)
    ));
    Ok(())
}

#[test]
fn given_exhausted_layer_budget_when_later_rows_arrive_then_they_are_not_processed() -> TestResult {
    let root = key("root")?;
    let child = key("child")?;
    let accepted = edge(&root, &child)?;
    let mut state = Traversal::new(&root, GraphLimits::new(2, 4, 1)?)?;
    state.add_layer(vec![
        accepted.clone(),
        edge(&root, &key("refused")?)?,
        edge(&key("foreign-from")?, &key("foreign-to")?)?,
    ])?;
    assert_eq!(state.edges, vec![accepted]);
    assert_eq!(state.visited, vec![root, child.clone()]);
    assert_eq!(state.frontier, vec![child]);
    assert_eq!(state.truncation, Truncation::Budget);
    Ok(())
}
