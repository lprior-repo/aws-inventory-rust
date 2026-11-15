use crate::classes::{DenyList, DenyReason};
use crate::overlay::{self, Binding, Identity, IdentityField, ItemSelector, MemberPath, OperationName,
    OperationOverlay, OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair};

type TestResult = Result<(), Box<dyn std::error::Error>>;

include!("test_model.rs");

const SERVICES: &[(&str, usize)] = &[
    ("data-pipeline", 19), ("database-migration-service", 119), ("databrew", 44),
    ("dataexchange", 37), ("datasync", 53), ("datazone", 195), ("dax", 21),
    ("deadline", 127), ("detective", 29), ("device-farm", 77), ("devops-agent", 63),
    ("devops-guru", 31), ("direct-connect", 73), ("directory-service-data", 17),
    ("directory-service", 80), ("dlm", 8), ("docdb-elastic", 19), ("docdb", 56),
    ("drs", 70), ("dsql", 16), ("dynamodb-streams", 4), ("ebs", 6),
    ("ec2-instance-connect", 2), ("ecr-public", 23), ("ecr", 62), ("efs", 31),
    ("eks-auth", 1), ("elastic-beanstalk", 47), ("elastic-load-balancing", 29),
    ("elasticache", 77), ("elasticsearch-service", 52), ("elementalinference", 24),
    ("emr-containers", 25), ("emr-serverless", 22), ("emr", 65), ("endusermessaging", 27),
    ("entityresolution", 38), ("eventbridge", 57), ("eventbridgev2", 25), ("evs", 24),
    ("firehose", 12), ("fis", 26), ("fms", 42), ("forecast", 63), ("forecastquery", 2),
    ("frauddetector", 73), ("freetier", 5), ("fsx", 48),
];

fn operation<'a>(policies: &'a [ServiceOverlay], service: &str, name: &str)
    -> Result<&'a OperationOverlay, Box<dyn std::error::Error>> {
    policies.iter().find(|policy| policy.service() == service)
        .and_then(|policy| policy.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing policy {service}/{name}").into())
}

#[test]
fn public_registry_dispositions_cover_every_pinned_d_f_service_and_operation() -> TestResult {
    // Given the exact pinned service/operation counts, excluding the first-wave services.
    let policies = overlay::embedded()?;
    let mut total = 0;
    let mut callable = 0;
    // When consumers read the public embedded registry.
    for (service, expected_count) in SERVICES {
        let policy = policies.iter().find(|p| p.service() == *service).ok_or(*service)?;
        assert_eq!(policy.operations().len(), *expected_count, "{service}");
        total += policy.operations().len();
        // Then every operation has an explicit disposition, never inferred read-name approval.
        for plan in policy.operations().values() {
            match plan.class {
                Some(OverlayClass::Enumerate | OverlayClass::Expand) => {
                    callable += 1;
                    assert_ne!(plan.items, None);
                    assert_ne!(plan.identity, None);
                    assert_ne!(plan.paginator, None);
                    assert_ne!(plan.scope, None);
                }
                Some(OverlayClass::Deny(_)) => assert_eq!(plan.items, None),
                Some(OverlayClass::Defer) => {
                    assert_ne!(plan.defer_reason.as_deref().unwrap_or_default(), "");
                    assert_eq!(plan.items, None);
                    assert_eq!(plan.identity, None);
                }
                other => return Err(format!("unexpected {service} disposition: {other:?}").into()),
            }
        }
    }
    assert_eq!(total, 2066);
    assert_eq!(callable, 201);
    Ok(())
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
        ("directory-service", "DescribeDirectories", DenyReason::Secret),
        ("docdb", "DescribeDBInstances", DenyReason::Secret),
        ("database-migration-service", "DescribeCertificates", DenyReason::Secret),
        ("database-migration-service", "DescribeEndpoints", DenyReason::Secret),
        ("database-migration-service", "DescribeReplicationTasks", DenyReason::Content),
        ("database-migration-service", "DescribeReplicationConfigs", DenyReason::Content),
        ("database-migration-service", "DescribeReplicationInstances", DenyReason::Content),
        ("databrew", "ListDatasets", DenyReason::Content),
        ("dataexchange", "ListJobs", DenyReason::Secret),
        ("dataexchange", "ListRevisionAssets", DenyReason::Secret),
        ("devops-agent", "ListServices", DenyReason::Secret),
        ("device-farm", "ListProjects", DenyReason::Secret),
        ("dynamodb-streams", "GetRecords", DenyReason::Content),
        ("forecastquery", "QueryForecast", DenyReason::Paid),
        ("frauddetector", "GetEventPrediction", DenyReason::Paid),
        ("ec2-instance-connect", "SendSSHPublicKey", DenyReason::Mutating),
        ("direct-connect", "DescribeVirtualInterfaces", DenyReason::Secret),
        ("emr-containers", "ListJobTemplates", DenyReason::Content),
        ("frauddetector", "GetExternalModels", DenyReason::Content),
    ];
    // When resolving those APIs, then projected metadata never disguises the whole call as safe.
    for (service, name, reason) in cases {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.class, Some(OverlayClass::Deny(reason)), "{service}/{name}");
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
        ("ecr", "DescribeImages", vec!["registryId", "repositoryName", "imageDigest"]),
        ("ecr-public", "DescribeImages", vec!["registryId", "repositoryName", "imageDigest"]),
        ("datazone", "ListProjects", vec!["domainId", "id"]),
        ("emr", "ListStudioSessionMappings", vec!["StudioId", "IdentityType", "IdentityId"]),
    ];
    // When policies choose selected-item keys, then sibling resources cannot alias by bare name/id.
    for (service, name, members) in cases {
        let expected = members.into_iter().map(MemberPath::new).collect::<Result<Vec<_>, _>>()?;
        assert_eq!(operation(&policies, service, name)?.identity, Some(Identity::Key(expected)));
    }
    assert_eq!(operation(&policies, "deadline", "ListQueues")?.identity, Some(Identity::CorrelatedKey(vec![
        IdentityField::Request(MemberPath::new("farmId")?),
        IdentityField::Selected(MemberPath::new("queueId")?),
    ])));
    assert_eq!(operation(&policies, "ecr", "DescribeImages")?.bind.get(&MemberPath::new("repositoryName")?),
        Some(&Binding::From { operation: OperationName::new("DescribeRepositories")?, path: MemberPath::new("repositoryName")? }));
    Ok(())
}

#[test]
fn pagination_keeps_actual_cursor_names_and_truncation_signal() -> TestResult {
    // Given services whose cursor names differ and whose outputs require an explicit more-results flag.
    let policies = overlay::embedded()?;
    let cases = [
        ("efs", "DescribeFileSystems", "Marker", "NextMarker"),
        ("dynamodb-streams", "ListStreams", "ExclusiveStartStreamArn", "LastEvaluatedStreamArn"),
        ("database-migration-service", "DescribeDataProviders", "Marker", "Marker"),
        ("ecr", "DescribeImages", "nextToken", "nextToken"),
    ];
    // When executing collection descriptors, then continuation never silently stops after the first page.
    for (service, name, input, output) in cases {
        assert_eq!(operation(&policies, service, name)?.paginator, Some(Paginator::Token(TokenPair {
            input: MemberPath::new(input)?, output: MemberPath::new(output)?, optional: false,
        })));
    }
    assert_eq!(operation(&policies, "data-pipeline", "ListPipelines")?.paginator,
        Some(Paginator::Tokens { pairs: vec![TokenPair { input: MemberPath::new("marker")?,
            output: MemberPath::new("marker")?, optional: false }],
            truncated: Some(MemberPath::new("hasMoreResults")?) }));
    Ok(())
}

#[test]
fn unsupported_last_element_cursor_and_parent_identity_are_explicit_not_fake_collection() -> TestResult {
    // Given runtime paths cannot extract a list's last element or join bound inputs into identity.
    let policies = overlay::embedded()?;
    // When these operations are resolved, then they remain noncallable with exact capability evidence.
    for (service, name, evidence) in [
        ("firehose", "ListDeliveryStreams", "last list element"),
        ("deadline", "ListWorkers", "credentials"),
        ("efs", "DescribeMountTargetSecurityGroups", "bound request parent"),
        ("eventbridge", "ListTargetsByRule", "bound input"),
        ("dax", "DescribeParameters", "parent-input identity"),
    ] {
        let plan = operation(&policies, service, name)?;
        assert_eq!(plan.class, Some(OverlayClass::Defer));
        assert!(plan.defer_reason.as_deref().ok_or("missing evidence")?.contains(evidence), "{service}/{name}");
        assert_eq!(plan.identity, None);
    }
    assert_eq!(operation(&policies, "ecr-public", "DescribeImages")?.scope, Some(Scope::Account));
    assert_eq!(operation(&policies, "ecr", "DescribeImages")?.scope, Some(Scope::Region));
    assert_eq!(operation(&policies, "elasticache", "DescribeGlobalReplicationGroups")?.scope, Some(Scope::Account));
    assert_eq!(operation(&policies, "direct-connect", "DescribeDirectConnectGatewayAttachments")?.scope, Some(Scope::Account));
    Ok(())
}

#[test]
fn public_checker_accepts_the_real_ecr_parent_and_composite_image_plan() -> TestResult {
    // Given a minimal Smithy fixture containing the real API names, required input and output members.
    let root = tempfile::tempdir()?;
    let models = root.path().join("models/ecr/service/2015-09-21");
    std::fs::create_dir_all(&models)?;
    std::fs::write(models.join("ecr.json"), ECR_MODEL)?;
    let embedded = overlay::embedded()?;
    let names = ["DescribeRepositories", "DescribeImages"];
    let selected = names.into_iter().map(|name| Ok((name, operation(&embedded, "ecr", name)?.clone())))
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let policy = ServiceOverlay::new("ecr", selected)?;
    // When the public model checker validates the actual policies, not hand-authored substitute policies.
    let checked = overlay::check_policies(root.path(), &[policy], &DenyList::builtin())?;
    let service = checked.services().first().ok_or("missing checked service")?;
    // Then both root and expanded APIs are callable, with the real composite identity preserved.
    assert_eq!(service.callable().len(), 2);
    let images = service.callable().iter().find(|operation| operation.operation() == "DescribeImages")
        .ok_or("missing image plan")?;
    assert_eq!(images.class(), overlay::EffectiveClass::Expand);
    assert_eq!(images.plan().identity, Some(Identity::Key(vec![MemberPath::new("registryId")?,
        MemberPath::new("repositoryName")?, MemberPath::new("imageDigest")?])));
    assert_eq!(images.plan().items, Some(ItemSelector::Member(MemberPath::new("imageDetails")?)));
    Ok(())
}

mod descendants {
    include!("tests_descendants.rs");
}

#[test]
fn search_domain_roots_partition_engine_types_without_changing_canonical_identities() -> TestResult {
    let policies = overlay::embedded()?;
    let engine = MemberPath::new("EngineType")?;
    for (service, value) in [("elasticsearch-service", "Elasticsearch"), ("opensearch", "OpenSearch")] {
        let plan = operation(&policies, service, "ListDomainNames")?;
        assert_eq!(plan.fixed.get(&engine), Some(&overlay::FixedValue::String(value.to_owned())));
        assert_eq!(plan.identity, Some(Identity::Key(vec![MemberPath::new("DomainName")?])));
    }
    Ok(())
}

#[test]
fn databrew_identity_uses_required_unique_names_not_optional_arns() -> TestResult {
    let policies = overlay::embedded()?;
    for name in ["ListJobs", "ListProjects", "ListRulesets", "ListSchedules"] {
        assert_eq!(operation(&policies, "databrew", name)?.identity,
            Some(Identity::Key(vec![MemberPath::new("Name")?])));
    }
    Ok(())
}
