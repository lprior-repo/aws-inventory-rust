use super::*;
mod stages;
use crate::consumer_fixture as fixture;
use fixture::TestResult;
use inventory_core::catalog::runtime::{
    ArnResourceKind, BindingTransform, BindingValue, ContextSource, PlanScope, UrlResourceKind,
};
use std::collections::BTreeSet;

fn names(index: &ConsumerIndex<'_, '_>, service: &str, operation: &str) -> BTreeSet<String> {
    index
        .candidates(service, operation)
        .map(|plan| plan.operation_name().to_owned())
        .collect()
}

#[test]
fn given_correlated_analyzer_consumers_when_indexed_then_only_direct_producers_fan_out_once()
-> TestResult {
    let compiled = fixture::compile(
        "accessanalyzer",
        "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    assert_eq!(
        names(&index, "accessanalyzer", "ListAnalyzers"),
        BTreeSet::from(["ListArchiveRules".to_owned()])
    );
    assert_eq!(
        names(&index, "accessanalyzer", "ListArchiveRules"),
        BTreeSet::from(["GetArchiveRule".to_owned()])
    );
    assert_eq!(
        index
            .candidates("accessanalyzer", "ListArchiveRules")
            .count(),
        1,
        "request-context and selected-item inputs must not schedule the same child twice"
    );
    assert_eq!(
        index.candidates("accessanalyzer", "GetArchiveRule").count(),
        0
    );
    assert_eq!(
        index
            .candidates("unrelated-service", "ListArchiveRules")
            .count(),
        0
    );
    assert_eq!(
        index
            .candidates("accessanalyzer", "UnrelatedOperation")
            .count(),
        0
    );
    Ok(())
}

#[test]
fn given_transformed_zone_binding_when_indexed_then_original_producer_selects_borrowed_child()
-> TestResult {
    let compiled = fixture::compile(
        "route-53",
        "2013-04-01",
        &["ListHostedZones", "GetHostedZone", "ListHealthChecks"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    let child = catalog
        .lookup("route-53", "GetHostedZone")
        .ok_or("hosted zone child missing")?;
    assert!(
        child
            .bindings()
            .iter()
            .any(|binding| matches!(binding.source, BindingSource::Transformed { .. }))
    );
    let candidates = index
        .candidates("route-53", "ListHostedZones")
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(
        candidates
            .first()
            .is_some_and(|candidate| std::ptr::eq(*candidate, child)),
        "the index must borrow the original plan and branch metadata"
    );
    assert_eq!(index.candidates("route-53", "ListHealthChecks").count(), 0);
    assert_eq!(
        index.candidates("other-service", "ListHostedZones").count(),
        0
    );
    Ok(())
}

#[test]
fn given_role_tag_target_when_indexed_then_only_reviewed_parent_operation_selects_it() -> TestResult
{
    let compiled = fixture::compile(
        "iam",
        "2010-05-08",
        &["ListRoles", "GetRole", "ListRoleTags"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let index = ConsumerIndex::build(&catalog)?;
    let role_tags = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("role tags missing")?;
    assert_eq!(
        names(&index, "iam", "ListRoles"),
        BTreeSet::from(["GetRole".to_owned(), "ListRoleTags".to_owned()])
    );
    let expected = catalog
        .plans()
        .iter()
        .filter(|plan| matches!(plan.operation_name(), "GetRole" | "ListRoleTags"))
        .map(CallablePlan::operation_name)
        .collect::<Vec<_>>();
    assert_eq!(
        index
            .candidates("iam", "ListRoles")
            .map(CallablePlan::operation_name)
            .collect::<Vec<_>>(),
        expected
    );
    assert!(
        index
            .candidates("iam", "ListRoles")
            .any(|plan| std::ptr::eq(plan, role_tags))
    );
    assert_eq!(
        index.candidates("iam", "GetRole").count(),
        0,
        "sharing the resource kind is not authority to become a tag target"
    );
    assert_eq!(index.candidates("another-service", "ListRoles").count(), 0);
    Ok(())
}

#[test]
fn given_branched_root_when_indexed_then_fixed_branch_inputs_do_not_invent_row_consumers()
-> TestResult {
    let compiled = fixture::compile(
        "cloudwatch-logs",
        "2014-03-28",
        &["DescribeResourcePolicies"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let root = catalog
        .lookup("cloudwatch-logs", "DescribeResourcePolicies")
        .ok_or("policy plan absent")?;
    assert_eq!(
        root.branches()
            .iter()
            .map(|branch| branch.discriminator.as_str())
            .collect::<Vec<_>>(),
        vec!["ACCOUNT", "RESOURCE"]
    );
    let index = ConsumerIndex::build(&catalog)?;
    assert_eq!(
        index
            .candidates("cloudwatch-logs", "DescribeResourcePolicies")
            .count(),
        0
    );
    assert_eq!(index.candidates("unrelated", "ListItems").count(), 0);
    assert_eq!(
        root.branches().len(),
        2,
        "branch expansion remains with the existing scheduler"
    );
    Ok(())
}

fn operation_source() -> BindingSource {
    BindingSource::Operation {
        service: "modeled-service".into(),
        operation: "ListParents".into(),
        path: Box::default(),
    }
}

#[test]
fn given_each_supported_producer_source_when_resolved_then_exact_service_and_operation_are_retained()
-> TestResult {
    let sources = [
        operation_source(),
        BindingSource::OperationArn {
            service: "modeled-service".into(),
            operation: "ListParents".into(),
            path: Box::default(),
            arn_service: "sns".into(),
            resource_kind: ArnResourceKind::SnsTopic,
        },
        BindingSource::OperationUrl {
            service: "modeled-service".into(),
            operation: "ListParents".into(),
            path: Box::default(),
            resource_kind: UrlResourceKind::SqsQueue,
        },
        BindingSource::Context {
            service: "modeled-service".into(),
            operation: "ListParents".into(),
            path: Box::default(),
            source: ContextSource::Request,
        },
        BindingSource::Transformed {
            source: Box::new(operation_source()),
            transform: BindingTransform::StripPrefix("/parent/".into()),
        },
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

