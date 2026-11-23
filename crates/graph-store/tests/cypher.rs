mod common;

use common::{TestResult, edge, key, row};
use inventory_graph::{GraphError, GraphLimits, MAX_EDGES, Provenance, cypher};

#[test]
fn node_uses_full_scoped_digest_and_escaped_parameters() -> TestResult {
    let source = key("123456789012", "vpc-'\\-1")?;
    let query = cypher::node(&row(&source))?;
    assert!(query.contains("name='O\\'Brien\\\\Cloud\\n'"));
    assert!(query.contains("MERGE (n:Resource {key:$key})"));
    let other = key("000000000000", "vpc-'\\-1")?;
    assert_ne!(cypher::node(&row(&source))?, cypher::node(&row(&other))?);
    assert!(query.contains(
        "content_hash='0707070707070707070707070707070707070707070707070707070707070707'"
    ));
    Ok(())
}

#[test]
fn replacement_preserves_label_provenance_and_both_edge_directions() -> TestResult {
    let source = key("123456789012", "vpc-a")?;
    let target = key("123456789012", "vpc-b")?;
    let edge = edge(&target, &source, "reviewed-rule")?;
    let query = cypher::replace_edges(&source, &[edge])?;
    assert!(query.contains("(owner)-[old]-(other:Resource) DELETE old"));
    assert!(query.contains("[:`CONTAINS` {relation:$relation0,provenance:$provenance0}]"));
    assert!(query.contains("relation0='CONTAINS'"));
    assert!(query.contains("provenance0='reviewed-rule'"));
    assert!(cypher::delete(&source)?.contains("DETACH DELETE n"));
    assert!(cypher::replace_edges(&source, &[])?.ends_with("RETURN owner.key"));
    Ok(())
}

#[test]
fn invalid_replacements_and_unbounded_values_fail_before_io() -> TestResult {
    let owner = key("123456789012", "vpc-a")?;
    let other = key("123456789012", "vpc-b")?;
    let unrelated = edge(&other, &other, "model")?;
    assert!(matches!(
        cypher::replace_edges(&owner, &[unrelated]),
        Err(GraphError::UnrelatedEdge)
    ));
    let edges = vec![edge(&owner, &other, "model")?; MAX_EDGES.saturating_add(1)];
    assert!(matches!(
        cypher::replace_edges(&owner, &edges),
        Err(GraphError::Limits)
    ));
    let mut invalid = row(&owner);
    invalid.name = Some("x".repeat(4097));
    assert!(matches!(
        cypher::node(&invalid),
        Err(GraphError::InvalidRow)
    ));
    assert!(Provenance::parse("").is_err());
    assert!(Provenance::parse("model\n").is_err());
    Ok(())
}

#[test]
fn cursor_queries_are_exclusive_bounded_and_scoped() -> TestResult {
    let source = key("123456789012", "vpc-a")?;
    let target = key("000000000000", "vpc-a")?;
    let edge = edge(&source, &target, "model")?;
    let nodes = cypher::node_keys(Some(&source), 5)?;
    let edges = cypher::edge_keys(Some(&edge), 7)?;
    assert!(nodes.contains("WHERE n.key > $after"));
    assert!(nodes.ends_with("ORDER BY n.key LIMIT 5"));
    assert!(edges.contains("r.provenance > $provenance0"));
    assert!(edges.ends_with("ORDER BY s.key,t.key,r.relation,r.provenance LIMIT 7"));
    assert!(cypher::node_keys(None, 0).is_err());
    assert!(cypher::edge_keys(None, 2001).is_err());
    Ok(())
}

#[test]
fn traversal_limits_cannot_represent_invalid_budgets() {
    assert!(GraphLimits::new(0, 1, 1).is_err());
    assert!(GraphLimits::new(4, 1, 1).is_err());
    assert!(GraphLimits::new(1, 0, 1).is_err());
    assert!(GraphLimits::new(1, 501, 1).is_err());
    assert!(GraphLimits::new(1, 1, 0).is_err());
    assert!(GraphLimits::new(1, 1, 2001).is_err());
    assert!(GraphLimits::new(3, 500, 2000).is_ok());
}
