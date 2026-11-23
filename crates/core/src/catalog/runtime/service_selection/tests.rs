use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_whole_and_qualified_services_when_selected_then_only_named_roots_match() -> TestResult {
    let names = [("ec2", "DescribeVpcs"), ("ec2", "DescribeSubnets"), ("s3", "ListBuckets")];
    assert_eq!(roots(&names, &["ec2".into()])?, BTreeSet::from([0, 1]));
    assert_eq!(roots(&names, &["ec2:DescribeSubnets".into(), "s3".into()])?, BTreeSet::from([1, 2]));
    Ok(())
}

#[test]
fn given_unknown_or_empty_selector_when_selected_then_no_broad_fallback_occurs() {
    let names = [("ec2", "DescribeVpcs")];
    ["", "unknown", "ec2:", ":DescribeVpcs", "ec2:Missing", "ec2:DescribeVpcs:extra"].iter().for_each(|selector| {
        assert!(matches!(roots(&names, &[(*selector).into()]), Err(RuntimeCatalogError::Selection(_))));
    });
}

#[test]
fn given_transitive_children_and_shared_parent_when_selected_then_closure_preserves_dependencies() -> TestResult {
    let mut graph = vec![Vec::new(); 6];
    connect(&mut graph, 1, 0, false)?;
    connect(&mut graph, 2, 1, false)?;
    connect(&mut graph, 3, 0, false)?;
    connect(&mut graph, 0, 4, true)?;
    connect(&mut graph, 5, 4, true)?;
    assert_eq!(closure(&graph, BTreeSet::from([1]))?, BTreeSet::from([0, 1, 2, 3, 4]));
    assert_eq!(closure(&graph, BTreeSet::from([4]))?, BTreeSet::from([4]));
    Ok(())
}

#[test]
fn given_cyclic_bindings_when_selected_then_each_plan_is_visited_once() -> TestResult {
    let graph = vec![vec![1], vec![2], vec![0], vec![]];
    assert_eq!(closure(&graph, BTreeSet::from([0]))?, BTreeSet::from([0, 1, 2]));
    Ok(())
}
