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

#[path = "../../../../tests/support/base_catalog.rs"]
mod base_catalog;
mod fixture;

#[test]
fn given_qualified_root_when_catalog_selected_then_all_bootstrap_plans_survive() -> TestResult {
    let bytes = base_catalog::base()?;
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let required: BTreeSet<_> = catalog
        .plans()
        .iter()
        .filter(|plan| {
            matches!(
                plan.purpose(),
                PlanPurpose::AccountIdentity { .. }
                    | PlanPurpose::RegionDiscovery { .. }
                    | PlanPurpose::EndpointDiscovery
            )
        })
        .map(name)
        .collect();
    assert!(required.contains(&("sts", "GetCallerIdentity")));
    assert!(required.contains(&("ec2", "DescribeRegions")));
    let selected =
        RuntimeCatalog::from_bytes(bytes)?.select_services(&["ec2:DescribeVpcs".into()])?;
    let actual: BTreeSet<_> = selected.plans().iter().map(name).collect();
    assert!(required.is_subset(&actual));
    assert!(actual.contains(&("ec2", "DescribeVpcs")));
    Ok(())
}

#[test]
fn given_qualified_root_when_catalog_selected_then_unselected_siblings_are_removed() -> TestResult {
    let catalog = RuntimeCatalog::from_bytes(base_catalog::base()?)?;
    assert!(catalog.lookup("ec2", "DescribeSubnets").is_some());
    assert!(catalog.lookup("s3", "ListBuckets").is_some());
    let selected = catalog.select_services(&["ec2:DescribeVpcs".into()])?;
    assert!(selected.lookup("ec2", "DescribeVpcs").is_some());
    assert!(selected.lookup("ec2", "DescribeSubnets").is_none());
    assert!(selected.lookup("s3", "ListBuckets").is_none());
    Ok(())
}

#[test]
fn given_no_selectors_when_catalog_selected_then_plan_order_and_catalog_are_unchanged() -> TestResult
{
    let bytes = base_catalog::base()?;
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let expected: Vec<_> = catalog.plans().iter().map(name).collect();
    assert!(!expected.is_empty());
    let selected = RuntimeCatalog::from_bytes(bytes)?.select_services(&[])?;
    assert_eq!(
        selected.plans().iter().map(name).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(selected.raw_bytes(), bytes);
    assert_eq!(selected.schemas().len(), catalog.schemas().len());
    assert_eq!(selected.decisions(), catalog.decisions());
    Ok(())
}

#[test]
fn given_duplicate_service_operation_when_catalog_decoded_then_ambiguous_names_are_rejected()
-> TestResult {
    let bytes = base_catalog::base()?;
    RuntimeCatalog::from_bytes(bytes)?;
    let duplicated = fixture::duplicate_plan(bytes)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&duplicated),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}
