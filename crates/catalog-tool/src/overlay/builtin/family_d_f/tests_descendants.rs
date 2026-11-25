use crate::classes::DenyReason;
use crate::overlay;
use crate::overlay::{
    Binding, FixedValue, Identity, MemberPath, OperationName, OverlayClass, Paginator, Scope,
    TokenPair,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn operation<'a>(
    policies: &'a [crate::overlay::ServiceOverlay],
    service: &str,
    name: &str,
) -> Result<&'a crate::overlay::OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .and_then(|policy| policy.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

const BOUND_CASES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "dsql",
        "ListStreams",
        "clusterIdentifier",
        "ListClusters",
        "identifier",
    ),
    ("detective", "ListMembers", "GraphArn", "ListGraphs", "Arn"),
    (
        "emr-serverless",
        "ListJobRuns",
        "applicationId",
        "ListApplications",
        "id",
    ),
    (
        "emr-serverless",
        "ListSessions",
        "applicationId",
        "ListApplications",
        "id",
    ),
    (
        "drs",
        "ListRecoveryPlanSteps",
        "recoveryPlanArn",
        "ListRecoveryPlans",
        "recoveryPlanArn",
    ),
    (
        "drs",
        "ListRecoveryPlanExecutionSteps",
        "recoveryPlanExecutionArn",
        "ListRecoveryPlanExecutions",
        "recoveryPlanExecutionArn",
    ),
];

#[test]
fn required_firewall_page_size_is_a_typed_bounded_integer_not_a_resource_filter() -> TestResult {
    let policies = overlay::embedded()?;
    for name in ["ListAppsLists", "ListProtocolsLists"] {
        let plan = operation(&policies, "fms", name)?;
        assert_eq!(plan.class, Some(OverlayClass::Enumerate));
        assert_eq!(
            plan.fixed.get(&MemberPath::new("MaxResults")?),
            Some(&FixedValue::Integer(100))
        );
        assert_eq!(plan.fixed.len(), 1);
        assert_eq!(plan.bind.len(), 0);
        assert_eq!(
            plan.identity,
            Some(Identity::Arn(MemberPath::new("ListArn")?))
        );
        assert_eq!(
            plan.paginator,
            Some(Paginator::Token(TokenPair {
                input: MemberPath::new("NextToken")?,
                output: MemberPath::new("NextToken")?,
                optional: false,
            }))
        );
    }
    Ok(())
}

#[test]
fn descendants_bind_real_selected_parent_identifiers_not_invented_arn_components() -> TestResult {
    let policies = overlay::embedded()?;
    for (service, name, input, parent, member) in BOUND_CASES {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.class, Some(OverlayClass::Expand), "{service}/{name}");
        assert_eq!(plan.bind.len(), 1);
        assert_eq!(
            plan.bind.get(&MemberPath::new(input)?),
            Some(&Binding::From {
                operation: OperationName::new(parent)?,
                path: MemberPath::new(member)?,
            })
        );
        assert_eq!(plan.scope, Some(Scope::Region));
    }
    assert!(
        operation(&policies, "dsql", "ListClusters")?
            .queryable
            .contains(&MemberPath::new("identifier")?)
    );
    assert_eq!(
        operation(&policies, "detective", "ListMembers")?.identity,
        Some(Identity::Key(vec![
            MemberPath::new("GraphArn")?,
            MemberPath::new("AccountId")?
        ]))
    );
    Ok(())
}

#[test]
fn endpoint_and_selenium_projections_cannot_hide_whole_response_programs_or_credentials()
-> TestResult {
    let policies = overlay::embedded()?;
    let cases = [
        ("emr-containers", "ListManagedEndpoints", DenyReason::Secret),
        ("emr-containers", "ListJobRuns", DenyReason::Content),
        ("emr", "ListSessions", DenyReason::Secret),
        ("emr", "GetSessionEndpoint", DenyReason::Secret),
        ("emr-serverless", "GetSessionEndpoint", DenyReason::Secret),
        ("device-farm", "ListTestGridSessions", DenyReason::Content),
        ("frauddetector", "GetRules", DenyReason::Content),
    ];
    for (service, name, reason) in cases {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.class, Some(OverlayClass::Deny(reason)));
        assert_eq!(plan.identity, None);
        assert_eq!(plan.items, None);
    }
    Ok(())
}

#[test]
fn shared_job_arns_and_unassured_export_urls_remain_explicitly_noncallable() -> TestResult {
    let policies = overlay::embedded()?;
    let attempts = operation(&policies, "emr-serverless", "ListJobRunAttempts")?;
    assert_eq!(attempts.class, Some(OverlayClass::Defer));
    assert!(
        attempts
            .defer_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("attempt is an integer"))
    );
    for name in [
        "DescribeExtensionPackAssociations",
        "DescribeMetadataModelAssessments",
        "DescribeMetadataModelConversions",
        "DescribeMetadataModelCreations",
        "DescribeMetadataModelExportsAsScript",
        "DescribeMetadataModelExportsToTarget",
        "DescribeMetadataModelImports",
    ] {
        let plan = operation(&policies, "database-migration-service", name)?;
        assert_eq!(plan.class, Some(OverlayClass::Defer));
        assert!(
            plan.defer_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("ExportSqlDetails.ObjectURL"))
        );
        assert_eq!(plan.identity, None);
    }
    Ok(())
}
