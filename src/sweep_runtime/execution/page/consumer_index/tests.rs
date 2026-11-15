#[path = "../../../../../crates/catalog-tool/src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod fixture;
use super::*;
use catalog_tool as fixture_catalog;
use fixture::TestResult;
use inventory_core::catalog::runtime::{
    ArnResourceKind, BindingTransform, BindingValue, ContextSource, PlanScope, UrlResourceKind,
};
use std::collections::BTreeSet;

fn names(index: &ConsumerIndex<'_, '_>, service: &str, operation: &str) -> BTreeSet<String> {
    index.candidates(service, operation).map(|plan| plan.operation_name().to_owned()).collect()
}

#[test]
fn given_correlated_analyzer_consumers_when_indexed_then_only_direct_producers_fan_out_once() -> TestResult {
    let compiled = fixture::compile("accessanalyzer", "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    assert_eq!(names(&index, "accessanalyzer", "ListAnalyzers"), BTreeSet::from(["ListArchiveRules".to_owned()]));
    assert_eq!(names(&index, "accessanalyzer", "ListArchiveRules"), BTreeSet::from(["GetArchiveRule".to_owned()]));
    assert_eq!(index.candidates("accessanalyzer", "ListArchiveRules").count(), 1,
        "request-context and selected-item inputs must not schedule the same child twice");
    assert_eq!(index.candidates("accessanalyzer", "GetArchiveRule").count(), 0);
    assert_eq!(index.candidates("unrelated-service", "ListArchiveRules").count(), 0);
    assert_eq!(index.candidates("accessanalyzer", "UnrelatedOperation").count(), 0);
    Ok(())
}

#[test]
fn given_transformed_zone_binding_when_indexed_then_original_producer_selects_borrowed_child() -> TestResult {
    let compiled = fixture::compile("route-53", "2013-04-01",
        &["ListHostedZones", "GetHostedZone", "ListHealthChecks"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    let child = catalog.lookup("route-53", "GetHostedZone").ok_or("hosted zone child missing")?;
    assert!(child.bindings().iter().any(|binding| matches!(binding.source, BindingSource::Transformed { .. })));
    let candidates = index.candidates("route-53", "ListHostedZones").collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(candidates.first().is_some_and(|candidate| std::ptr::eq(*candidate, child)),
        "the index must borrow the original plan and branch metadata");
    assert_eq!(index.candidates("route-53", "ListHealthChecks").count(), 0);
    assert_eq!(index.candidates("other-service", "ListHostedZones").count(), 0);
    Ok(())
}

#[test]
fn given_role_tag_target_when_indexed_then_only_reviewed_parent_operation_selects_it() -> TestResult {
    let compiled = fixture::compile("iam", "2010-05-08", &["ListRoles", "GetRole", "ListRoleTags"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    let role_tags = catalog.lookup("iam", "ListRoleTags").ok_or("role tags missing")?;
    assert_eq!(names(&index, "iam", "ListRoles"), BTreeSet::from(["GetRole".to_owned(), "ListRoleTags".to_owned()]));
    let expected = catalog.plans().iter().filter(|plan| matches!(plan.operation_name(), "GetRole" | "ListRoleTags"))
        .map(CallablePlan::operation_name).collect::<Vec<_>>();
    assert_eq!(index.candidates("iam", "ListRoles").map(CallablePlan::operation_name).collect::<Vec<_>>(), expected);
    assert!(index.candidates("iam", "ListRoles").any(|plan| std::ptr::eq(plan, role_tags)));
    assert_eq!(index.candidates("iam", "GetRole").count(), 0,
        "sharing the resource kind is not authority to become a tag target");
    assert_eq!(index.candidates("another-service", "ListRoles").count(), 0);
    Ok(())
}

#[test]
fn given_branched_root_when_indexed_then_fixed_branch_inputs_do_not_invent_row_consumers() -> TestResult {
    let compiled = fixture::compile("cloudwatch-logs", "2014-03-28", &["DescribeResourcePolicies"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let root = catalog.lookup("cloudwatch-logs", "DescribeResourcePolicies").ok_or("policy plan absent")?;
    assert_eq!(root.branches().iter().map(|branch| branch.discriminator.as_str()).collect::<Vec<_>>(),
        vec!["ACCOUNT", "RESOURCE"]);
    let index = ConsumerIndex::build(&catalog)?;
    assert_eq!(index.candidates("cloudwatch-logs", "DescribeResourcePolicies").count(), 0);
    assert_eq!(index.candidates("unrelated", "ListItems").count(), 0);
    assert_eq!(root.branches().len(), 2, "branch expansion remains with the existing scheduler");
    Ok(())
}

fn operation_source() -> BindingSource {
    BindingSource::Operation { service: "modeled-service".into(), operation: "ListParents".into(), path: Box::default() }
}

#[test]
fn given_each_supported_producer_source_when_resolved_then_exact_service_and_operation_are_retained() -> TestResult {
    let sources = [
        operation_source(),
        BindingSource::OperationArn { service: "modeled-service".into(), operation: "ListParents".into(),
            path: Box::default(), arn_service: "sns".into(), resource_kind: ArnResourceKind::SnsTopic },
        BindingSource::OperationUrl { service: "modeled-service".into(), operation: "ListParents".into(),
            path: Box::default(), resource_kind: UrlResourceKind::SqsQueue },
        BindingSource::Context { service: "modeled-service".into(), operation: "ListParents".into(),
            path: Box::default(), source: ContextSource::Request },
        BindingSource::Transformed { source: Box::new(operation_source()),
            transform: BindingTransform::StripPrefix("/parent/".into()) },
    ];
    sources.iter().try_for_each(|source| -> TestResult {
        match dependency(source)? {
            Dependency::Producer(producer) => {
                assert_eq!(producer.service, "modeled-service");
                assert_eq!(producer.operation, "ListParents");
                assert_eq!(producer.branch, None);
            }
            _ => return Err("modeled producer dependency was lost".into()),
        }
        Ok(())
    })
}

#[test]
fn given_parent_identity_when_indexed_then_every_producer_can_offer_the_original_consumer() -> TestResult {
    let compiled = fixture::compile("route-53", "2013-04-01", &["ListHostedZones", "GetHostedZone"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("route-53", "ListHostedZones").ok_or("parent plan missing")?;
    // ParentIdentity is a legacy runtime source, not emitted by today's overlay
    // compiler. Exercise its production dependency/index path with a real plan;
    // no row eligibility or scheduling outcome is fabricated here.
    let source = BindingSource::ParentIdentity { path: Box::default() };
    let ordinal = catalog.plans().iter().position(|candidate| std::ptr::eq(candidate, plan))
        .ok_or("parent plan ordinal missing")?;
    let mut index = ConsumerIndex::build(&catalog)?;
    index.add_dependency(Consumer { ordinal, plan }, dependency(&source)?)?;
    let index = index.freeze();
    let expected = catalog.plans().iter().map(CallablePlan::operation_name).collect::<Vec<_>>();
    assert_eq!(index.candidates("route-53", "ListHostedZones")
        .map(CallablePlan::operation_name).collect::<Vec<_>>(), expected,
        "wildcard and exact consumers must merge once in original catalog order");
    let candidates = index.candidates("other-service", "ListOtherParents").collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(candidates.first().is_some_and(|candidate| std::ptr::eq(*candidate, plan)));
    Ok(())
}

#[test]
fn given_bounded_branch_metadata_when_dependencies_are_traversed_then_base_and_branches_are_not_lost() -> TestResult {
    let base = [BindingPlan { path: Box::default(), source: BindingSource::Account }];
    let branches = [ExecutionBranch { discriminator: "ACCOUNT".to_owned(),
        bindings: vec![BindingPlan { path: Box::default(), source: BindingSource::Fixed(BindingValue::String("ACCOUNT".to_owned())) }].into_boxed_slice(),
        emission: None, row_policy: None, scope: PlanScope::Account, request_region: None }];
    let dependencies = bindings(&base, &branches).map(|binding| dependency(&binding.source)).collect::<Result<Vec<_>, _>>()?;
    assert_eq!(dependencies.len(), 2);
    assert!(dependencies.iter().all(|dependency| matches!(dependency, Dependency::Independent)));
    Ok(())
}

#[test]
fn given_transform_depth_boundary_when_indexed_then_only_one_over_is_rejected() -> TestResult {
    let transformed = |depth| (0..depth).fold(operation_source(), |source, _| BindingSource::Transformed {
        source: Box::new(source), transform: BindingTransform::StripPrefix("/".into()),
    });
    let maximum = transformed(MAX_SOURCE_DEPTH.saturating_sub(1));
    assert!(matches!(dependency(&maximum)?, Dependency::Producer(_)));
    let excessive = transformed(MAX_SOURCE_DEPTH);
    assert!(matches!(dependency(&excessive), Err(ConsumerIndexError::SourceDepth)));
    Ok(())
}

#[test]
fn given_bounded_index_growth_when_capacity_is_exhausted_then_error_preserves_existing_entries() -> TestResult {
    let mut values = Vec::new();
    bounded_push(&mut values, 7, 1)?;
    assert!(matches!(bounded_push(&mut values, 8, 1), Err(ConsumerIndexError::Bounds)));
    assert_eq!(values, vec![7]);
    let empty = ConsumerIndex::from_plans(&[])?;
    assert_eq!(empty.candidates("any", "ListAnything").count(), 0);
    Ok(())
}

#[test]
fn given_exact_source_stages_when_indexed_then_only_that_stage_or_ordinary_dependencies_fan_out() -> TestResult {
    let compiled = fixture::compile("accessanalyzer", "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let mut index = ConsumerIndex { entries: Vec::new(), wildcards: Vec::new() };
    let plans = catalog.plans();
    let first = plans.first().ok_or("first plan missing")?;
    let second = plans.get(1).ok_or("second plan missing")?;
    let third = plans.get(2).ok_or("third plan missing")?;
    let stage = |branch: &str| BindingSource::ContextFromBranch {
        service: "staged-service".into(), operation: "ListParents".into(), branch: branch.into(),
        source: ContextSource::Request, path: Box::default(),
    };
    // The first consumer has an ordinary dependency as well as an exact-stage
    // dependency. Neither binding order nor overlapping keys may duplicate work.
    index.add_dependency(Consumer { ordinal: 0, plan: first }, dependency(&stage("DETAIL"))?)?;
    let ordinary = BindingSource::Operation {
        service: "staged-service".into(), operation: "ListParents".into(), path: Box::default(),
    };
    index.add_dependency(Consumer { ordinal: 0, plan: first }, dependency(&ordinary)?)?;
    index.add_dependency(Consumer { ordinal: 1, plan: second }, dependency(&stage("TAGS"))?)?;
    let transformed = BindingSource::Transformed {
        source: Box::new(stage("DETAIL")), transform: BindingTransform::StripPrefix("/".into()),
    };
    index.add_dependency(Consumer { ordinal: 2, plan: third }, dependency(&transformed)?)?;
    let index = index.freeze();
    let candidates = |branch| index.candidates_for_branch("staged-service", "ListParents", branch)
        .map(CallablePlan::operation_name).collect::<Vec<_>>();
    assert_eq!(candidates(None), vec![first.operation_name()]);
    assert_eq!(candidates(Some("DETAIL")), vec![first.operation_name(), third.operation_name()]);
    assert_eq!(candidates(Some("TAGS")), vec![first.operation_name(), second.operation_name()]);
    assert_eq!(candidates(Some("UNKNOWN")), vec![first.operation_name()],
        "ordinary dependencies remain eligible for inherited source stages");
    assert_eq!(index.candidates_for_branch("wrong-service", "ListParents", Some("DETAIL")).count(), 0);
    assert_eq!(index.candidates_for_branch("staged-service", "WrongOperation", Some("DETAIL")).count(), 0);
    Ok(())
}

#[test]
fn given_overlapping_stage_and_wildcard_keys_when_merged_then_catalog_order_is_deduplicated() -> TestResult {
    let compiled = fixture::compile("route-53", "2013-04-01", &["ListHostedZones", "GetHostedZone"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.plans().first().ok_or("plan missing")?;
    let consumer = Consumer { ordinal: 0, plan };
    let mut index = ConsumerIndex { entries: Vec::new(), wildcards: Vec::new() };
    let staged = BindingSource::ContextFromBranch {
        service: "source".into(), operation: "List".into(), branch: "READY".into(),
        source: ContextSource::SelectedItem, path: Box::default(),
    };
    index.add_dependency(consumer, dependency(&staged)?)?;
    index.add_dependency(consumer, Dependency::AnyProducer)?;
    index.add_dependency(consumer, Dependency::AnyProducer)?;
    let index = index.freeze();
    assert_eq!(index.candidates_for_branch("source", "List", Some("READY")).count(), 1);
    assert_eq!(index.candidates_for_branch("source", "List", Some("OTHER")).count(), 1);
    assert_eq!(index.candidates_for_branch("source", "List", None).count(), 1);
    Ok(())
}
