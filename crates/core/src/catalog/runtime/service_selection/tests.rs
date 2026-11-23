use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_whole_and_qualified_services_when_selected_then_only_named_roots_match() -> TestResult {
    let names = [
        ("ec2", "DescribeVpcs"),
        ("ec2", "DescribeSubnets"),
        ("s3", "ListBuckets"),
    ];
    assert_eq!(roots(&names, &["ec2".into()])?, BTreeSet::from([0, 1]));
    assert_eq!(
        roots(&names, &["ec2:DescribeSubnets".into(), "s3".into()])?,
        BTreeSet::from([1, 2])
    );
    Ok(())
}

#[test]
fn given_unknown_or_empty_selector_when_selected_then_no_broad_fallback_occurs() {
    let names = [("ec2", "DescribeVpcs")];
    for selector in [
        "",
        "unknown",
        "ec2:",
        ":DescribeVpcs",
        "ec2:Missing",
        "ec2:DescribeVpcs:extra",
    ] {
        assert!(matches!(
            roots(&names, &[selector.into()]),
            Err(RuntimeCatalogError::Selection(_))
        ));
    }
}

#[test]
fn given_transitive_ancestors_when_selected_then_unselected_descendants_and_siblings_stay_disabled()
-> TestResult {
    let mut graph = vec![Vec::new(); 6];
    connect(&mut graph, 1, 0)?;
    connect(&mut graph, 2, 1)?;
    connect(&mut graph, 3, 0)?;
    connect(&mut graph, 0, 4)?;
    connect(&mut graph, 5, 4)?;
    assert_eq!(
        closure(&graph, BTreeSet::from([1]))?,
        BTreeSet::from([0, 1, 4])
    );
    assert_eq!(closure(&graph, BTreeSet::from([4]))?, BTreeSet::from([4]));
    assert_eq!(
        closure(&graph, BTreeSet::from([2, 3]))?,
        BTreeSet::from([0, 1, 2, 3, 4])
    );
    Ok(())
}

#[test]
fn given_cyclic_bindings_when_selected_then_each_plan_is_visited_once() -> TestResult {
    let graph = vec![vec![1], vec![2], vec![0], vec![]];
    assert_eq!(
        closure(&graph, BTreeSet::from([0]))?,
        BTreeSet::from([0, 1, 2])
    );
    Ok(())
}
