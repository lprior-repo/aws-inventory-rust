use super::*;
#[test]
fn given_parent_identity_when_indexed_then_every_producer_can_offer_the_original_consumer()
-> TestResult {
    let compiled = fixture::compile(
        "route-53",
        "2013-04-01",
        &["ListHostedZones", "GetHostedZone"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("route-53", "ListHostedZones")
        .ok_or("parent plan missing")?;
    let source = BindingSource::ParentIdentity {
        path: Box::default(),
    };
    let ordinal = catalog
        .plans()
        .iter()
        .position(|candidate| std::ptr::eq(candidate, plan))
        .ok_or("parent plan ordinal missing")?;
    let mut index = ConsumerIndex::build(&catalog)?;
    index.add_dependency(Consumer { ordinal, plan }, dependency(&source)?)?;
    let index = index.freeze();
    let expected = catalog
        .plans()
        .iter()
        .map(CallablePlan::operation_name)
        .collect::<Vec<_>>();
    assert_eq!(
        index
            .candidates("route-53", "ListHostedZones")
            .map(CallablePlan::operation_name)
            .collect::<Vec<_>>(),
        expected,
        "wildcard and exact consumers must merge once in original catalog order"
    );
    let candidates = index
        .candidates("other-service", "ListOtherParents")
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(
        candidates
            .first()
            .is_some_and(|candidate| std::ptr::eq(*candidate, plan))
    );
    Ok(())
}

#[test]
fn given_bounded_branch_metadata_when_dependencies_are_traversed_then_base_and_branches_are_not_lost()
-> TestResult {
    let base = [BindingPlan {
        path: Box::default(),
        source: BindingSource::Account,
    }];
    let branches = [ExecutionBranch {
        discriminator: "ACCOUNT".to_owned(),
        bindings: vec![BindingPlan {
            path: Box::default(),
            source: BindingSource::Fixed(BindingValue::String("ACCOUNT".to_owned())),
        }]
        .into_boxed_slice(),
        emission: None,
        row_policy: None,
        scope: PlanScope::Account,
        request_region: None,
    }];
    let dependencies = bindings(&base, &branches)
        .map(|binding| dependency(&binding.source))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(dependencies.len(), 2);
    assert!(
        dependencies
            .iter()
            .all(|dependency| matches!(dependency, Dependency::Independent))
    );
    Ok(())
}

#[test]
fn given_transform_depth_boundary_when_indexed_then_only_one_over_is_rejected() -> TestResult {
    let transformed = |depth| {
        (0..depth).fold(operation_source(), |source, _| BindingSource::Transformed {
            source: Box::new(source),
            transform: BindingTransform::StripPrefix("/".into()),
        })
    };
    let maximum = transformed(MAX_SOURCE_DEPTH.saturating_sub(1));
    assert!(matches!(dependency(&maximum)?, Dependency::Producer(_)));
    let excessive = transformed(MAX_SOURCE_DEPTH);
    assert!(matches!(
        dependency(&excessive),
        Err(ConsumerIndexError::SourceDepth)
    ));
    Ok(())
}

#[test]
fn given_bounded_index_growth_when_capacity_is_exhausted_then_error_preserves_existing_entries()
-> TestResult {
    let mut values = Vec::new();
    bounded_push(&mut values, 7, 1)?;
    assert!(matches!(
        bounded_push(&mut values, 8, 1),
        Err(ConsumerIndexError::Bounds)
    ));
    assert_eq!(values, vec![7]);
    let empty = ConsumerIndex::from_plans(&[])?;
    assert_eq!(empty.candidates("any", "ListAnything").count(), 0);
    Ok(())
}

fn stage_sources() -> [(usize, BindingSource); 4] {
    let stage = |branch: &str| BindingSource::ContextFromBranch {
        service: "staged-service".into(),
        operation: "ListParents".into(),
        branch: branch.into(),
        source: ContextSource::Request,
        path: Box::default(),
    };
    [
        (0, stage("DETAIL")),
        (
            0,
            BindingSource::Operation {
                service: "staged-service".into(),
                operation: "ListParents".into(),
                path: Box::default(),
            },
        ),
        (1, stage("TAGS")),
        (
            2,
            BindingSource::Transformed {
                source: Box::new(stage("DETAIL")),
                transform: BindingTransform::StripPrefix("/".into()),
            },
        ),
    ]
}

fn staged_index<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
    sources: &'a [(usize, BindingSource)],
) -> Result<ConsumerIndex<'a, 'catalog>, Box<dyn std::error::Error>> {
    let mut index = ConsumerIndex {
        skipped_parents: Vec::new(),
        entries: Vec::new(),
        wildcards: Vec::new(),
    };
    for (ordinal, source) in sources {
        let plan = catalog.plans().get(*ordinal).ok_or("plan missing")?;
        index.add_dependency(
            Consumer {
                ordinal: *ordinal,
                plan,
            },
            dependency(source)?,
        )?;
    }
    Ok(index.freeze())
}

#[test]
fn given_detail_source_stage_when_indexed_then_ordinary_and_transformed_consumers_fan_out_once()
-> TestResult {
    let compiled = fixture::compile(
        "accessanalyzer",
        "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let sources = stage_sources();
    let index = staged_index(&catalog, &sources)?;
    let first = catalog.plans().first().ok_or("first plan missing")?;
    let third = catalog.plans().get(2).ok_or("third plan missing")?;
    let candidates = |branch| {
        index
            .candidates_for_branch("staged-service", "ListParents", branch)
            .map(CallablePlan::operation_name)
            .collect::<Vec<_>>()
    };
    assert_eq!(candidates(None), vec![first.operation_name()]);
    assert_eq!(
        candidates(Some("DETAIL")),
        vec![first.operation_name(), third.operation_name()]
    );
    assert_eq!(
        index
            .candidates_for_branch("wrong-service", "ListParents", Some("DETAIL"))
            .count(),
        0
    );
    assert_eq!(
        index
            .candidates_for_branch("staged-service", "WrongOperation", Some("DETAIL"))
            .count(),
        0
    );
    Ok(())
}

#[test]
fn given_tags_or_unknown_source_stage_when_indexed_then_only_matching_or_ordinary_consumers_fan_out()
-> TestResult {
    let compiled = fixture::compile(
        "accessanalyzer",
        "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let sources = stage_sources();
    let index = staged_index(&catalog, &sources)?;
    let first = catalog.plans().first().ok_or("first plan missing")?;
    let second = catalog.plans().get(1).ok_or("second plan missing")?;
    let candidates = |branch| {
        index
            .candidates_for_branch("staged-service", "ListParents", Some(branch))
            .map(CallablePlan::operation_name)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        candidates("TAGS"),
        vec![first.operation_name(), second.operation_name()]
    );
    assert_eq!(
        candidates("UNKNOWN"),
        vec![first.operation_name()],
        "ordinary dependencies remain eligible for inherited source stages"
    );
    Ok(())
}

#[test]
fn given_overlapping_stage_and_wildcard_keys_when_merged_then_catalog_order_is_deduplicated()
-> TestResult {
    let compiled = fixture::compile(
        "route-53",
        "2013-04-01",
        &["ListHostedZones", "GetHostedZone"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.plans().first().ok_or("plan missing")?;
    let consumer = Consumer { ordinal: 0, plan };
    let mut index = ConsumerIndex {
        skipped_parents: Vec::new(),
        entries: Vec::new(),
        wildcards: Vec::new(),
    };
    let staged = BindingSource::ContextFromBranch {
        service: "source".into(),
        operation: "List".into(),
        branch: "READY".into(),
        source: ContextSource::SelectedItem,
        path: Box::default(),
    };
    index.add_dependency(consumer, dependency(&staged)?)?;
    index.add_dependency(consumer, Dependency::AnyProducer)?;
    index.add_dependency(consumer, Dependency::AnyProducer)?;
    let index = index.freeze();
    assert_eq!(
        index
            .candidates_for_branch("source", "List", Some("READY"))
            .count(),
        1
    );
    assert_eq!(
        index
            .candidates_for_branch("source", "List", Some("OTHER"))
            .count(),
        1
    );
    assert_eq!(
        index.candidates_for_branch("source", "List", None).count(),
        1
    );
    Ok(())
}
