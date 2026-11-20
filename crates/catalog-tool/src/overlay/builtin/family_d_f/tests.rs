use crate::classes::{DenyList, DenyReason};
use crate::overlay::{
    self, Binding, Identity, IdentityField, ItemSelector, MemberPath, OperationName,
    OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

include!("test_model.rs");

fn operation<'a>(
    policies: &'a [ServiceOverlay],
    service: &str,
    name: &str,
) -> Result<&'a OperationOverlay, Box<dyn std::error::Error>> {
    policies
        .iter()
        .find(|policy| policy.service() == service)
        .and_then(|policy| policy.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

#[test]
fn whole_response_credentials_content_and_paid_workloads_never_become_callable() -> TestResult {
    // Given model-documented secrets, stored programs, artifact capabilities and billed queries.
    let policies = overlay::embedded()?;
    let cases = [
        ("ecr", "GetAuthorizationToken", DenyReason::Secret),
        ("firehose", "DescribeDeliveryStream", DenyReason::Secret),
        ("ecr", "BatchGetImage", DenyReason::Content),
        ("eks-auth", "AssumeRoleForPodIdentity", DenyReason::Secret),
        (
            "directory-service",
            "DescribeDirectories",
            DenyReason::Secret,
        ),
        ("docdb", "DescribeDBInstances", DenyReason::Secret),
        (
            "database-migration-service",
            "DescribeCertificates",
            DenyReason::Secret,
        ),
        (
            "database-migration-service",
            "DescribeEndpoints",
            DenyReason::Secret,
        ),
        (
            "database-migration-service",
            "DescribeReplicationTasks",
            DenyReason::Content,
        ),
        (
            "database-migration-service",
            "DescribeReplicationConfigs",
            DenyReason::Content,
        ),
        (
            "database-migration-service",
            "DescribeReplicationInstances",
            DenyReason::Content,
        ),
        ("databrew", "ListDatasets", DenyReason::Content),
        ("dataexchange", "ListJobs", DenyReason::Secret),
        ("dataexchange", "ListRevisionAssets", DenyReason::Secret),
        ("devops-agent", "ListServices", DenyReason::Secret),
        ("device-farm", "ListProjects", DenyReason::Secret),
        ("dynamodb-streams", "GetRecords", DenyReason::Content),
        ("forecastquery", "QueryForecast", DenyReason::Paid),
        ("frauddetector", "GetEventPrediction", DenyReason::Paid),
        (
            "ec2-instance-connect",
            "SendSSHPublicKey",
            DenyReason::Mutating,
        ),
        (
            "direct-connect",
            "DescribeVirtualInterfaces",
            DenyReason::Secret,
        ),
        ("emr-containers", "ListJobTemplates", DenyReason::Content),
        ("frauddetector", "GetExternalModels", DenyReason::Content),
    ];
    // When resolving those APIs, then projected metadata never disguises the whole call as safe.
    for (service, name, reason) in cases {
        let plan = operation(&policies, service, name)?;
        assert_eq!(
            plan.class,
            Some(OverlayClass::Deny(reason)),
            "{service}/{name}"
        );
        assert_eq!(plan.items, None);
        assert_eq!(plan.identity, None);
        assert_eq!(plan.paginator, None);
        assert_eq!(plan.tags, None);
    }
    Ok(())
}

#[test]
fn descendants_preserve_parent_context_in_the_selected_resource_identity() -> TestResult {
    // Given identifiers whose uniqueness is limited to the parent resource.
    let policies = overlay::embedded()?;
    let cases = [
        (
            "ecr",
            "DescribeImages",
            vec!["registryId", "repositoryName", "imageDigest"],
        ),
        (
            "ecr-public",
            "DescribeImages",
            vec!["registryId", "repositoryName", "imageDigest"],
        ),
        ("datazone", "ListProjects", vec!["domainId", "id"]),
        (
            "emr",
            "ListStudioSessionMappings",
            vec!["StudioId", "IdentityType", "IdentityId"],
        ),
    ];
    // When policies choose selected-item keys, then sibling resources cannot alias by bare name/id.
    for (service, name, members) in cases {
        let expected = members
            .into_iter()
            .map(MemberPath::new)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            operation(&policies, service, name)?.identity,
            Some(Identity::Key(expected))
        );
    }
    assert_eq!(
        operation(&policies, "deadline", "ListQueues")?.identity,
        Some(Identity::CorrelatedKey(vec![
            IdentityField::Request(MemberPath::new("farmId")?),
            IdentityField::Selected(MemberPath::new("queueId")?),
        ]))
    );
    assert_eq!(
        operation(&policies, "ecr", "DescribeImages")?
            .bind
            .get(&MemberPath::new("repositoryName")?),
        Some(&Binding::From {
            operation: OperationName::new("DescribeRepositories")?,
            path: MemberPath::new("repositoryName")?
        })
    );
    Ok(())
}

#[test]
fn pagination_keeps_actual_cursor_names_and_truncation_signal() -> TestResult {
    // Given services whose cursor names differ and whose outputs require an explicit more-results flag.
    let policies = overlay::embedded()?;
    let cases = [
        ("efs", "DescribeFileSystems", "Marker", "NextMarker"),
        (
            "dynamodb-streams",
            "ListStreams",
            "ExclusiveStartStreamArn",
            "LastEvaluatedStreamArn",
        ),
        (
            "database-migration-service",
            "DescribeDataProviders",
            "Marker",
            "Marker",
        ),
        ("ecr", "DescribeImages", "nextToken", "nextToken"),
    ];
    // When executing collection descriptors, then continuation never silently stops after the first page.
    for (service, name, input, output) in cases {
        assert_eq!(
            operation(&policies, service, name)?.paginator,
            Some(Paginator::Token(TokenPair {
                input: MemberPath::new(input)?,
                output: MemberPath::new(output)?,
                optional: false,
            }))
        );
    }
    // EventBridgeV2 Expand operations have no pagination.
    for (service, name) in [
        ("eventbridgev2", "DescribeEventBus"),
        ("eventbridgev2", "DescribeEventSource"),
        ("eventbridgev2", "DescribeSubscriber"),
    ] {
        assert_eq!(
            operation(&policies, service, name)?.paginator,
            Some(Paginator::None)
        );
    }
    // EventBridgeV2 Describe operations are unpaginated in the pinned model and use Paginator::None.
    // Verify the overlay policies are correctly structured for these unpaginated operations.
    for (service, name) in [
        ("eventbridgev2", "DescribeEventBus"),
        ("eventbridgev2", "DescribeEventSource"),
        ("eventbridgev2", "DescribeSubscriber"),
    ] {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.paginator, Some(Paginator::None));
        assert!(plan.items.is_some());
        assert!(plan.identity.is_some());
    }
    assert_eq!(
        operation(&policies, "data-pipeline", "ListPipelines")?.paginator,
        Some(Paginator::Tokens {
            pairs: vec![TokenPair {
                input: MemberPath::new("marker")?,
                output: MemberPath::new("marker")?,
                optional: false
            }],
            truncated: Some(MemberPath::new("hasMoreResults")?)
        })
    );
    Ok(())
}

#[test]
fn unsupported_last_element_cursor_and_parent_identity_are_explicit_not_fake_collection()
-> TestResult {
    // Given runtime paths cannot extract a list's last element or join bound inputs into identity.
    let policies = overlay::embedded()?;
    // When these operations are resolved, then they remain noncallable with exact capability evidence.
    for (service, name, evidence) in [
        ("deadline", "ListWorkers", "credentials"),
        (
            "efs",
            "DescribeMountTargetSecurityGroups",
            "bound request parent",
        ),
        ("eventbridge", "ListTargetsByRule", "bound input"),
        ("dax", "DescribeParameters", "parent-input identity"),
        ("chatbot", "ListAssociations", "ChatConfiguration"),
    ] {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.class, Some(OverlayClass::Defer));
        assert!(
            plan.defer_reason
                .as_deref()
                .ok_or("missing evidence")?
                .contains(evidence),
            "{service}/{name}"
        );
        assert_eq!(plan.identity, None);
    }
    assert_eq!(
        operation(&policies, "ecr-public", "DescribeImages")?.scope,
        Some(Scope::Account)
    );
    assert_eq!(
        operation(&policies, "ecr", "DescribeImages")?.scope,
        Some(Scope::Region)
    );
    assert_eq!(
        operation(&policies, "elasticache", "DescribeGlobalReplicationGroups")?.scope,
        Some(Scope::Account)
    );
    assert_eq!(
        operation(
            &policies,
            "direct-connect",
            "DescribeDirectConnectGatewayAttachments"
        )?
        .scope,
        Some(Scope::Account)
    );
    Ok(())
}
