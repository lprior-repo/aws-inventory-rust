use super::*;
#[test]
fn native_list_and_detail_views_share_the_exact_canonical_kind() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (service, list, detail, kind) in [
        (
            "gamelift",
            "ListAliases",
            "DescribeAlias",
            "AWS::GameLift::Alias",
        ),
        (
            "mq",
            "ListBrokers",
            "DescribeBroker",
            "AWS::AmazonMQ::Broker",
        ),
        (
            "transfer",
            "ListServers",
            "DescribeServer",
            "AWS::Transfer::Server",
        ),
        ("iam", "ListRoles", "GetRole", "AWS::IAM::Role"),
        (
            "medical-imaging",
            "ListDatastores",
            "GetDatastore",
            "AWS::HealthImaging::Datastore",
        ),
        (
            "license-manager",
            "ListLicenseConfigurations",
            "GetLicenseConfiguration",
            "AWS::LicenseManager::LicenseConfiguration",
        ),
        (
            "transfer",
            "ListAgreements",
            "DescribeAgreement",
            "AWS::Transfer::Agreement",
        ),
        (
            "s3-control",
            "ListStorageLensGroups",
            "GetStorageLensGroup",
            "AWS::S3::StorageLensGroup",
        ),
        (
            "wisdom",
            "ListAssistants",
            "GetAssistant",
            "AWS::Wisdom::Assistant",
        ),
    ] {
        for operation in [list, detail] {
            assert_eq!(
                map.operations()
                    .iter()
                    .find(|row| row.service == service && row.operation == operation)
                    .map(|row| row.kind.as_str()),
                Some(kind),
                "{service}.{operation}"
            );
        }
    }
    Ok(())
}

#[test]
fn pinned_schema_absences_remain_explicit_api_only_resources() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (service, operation, kind, absent_cfn) in [
        (
            "device-farm",
            "ListInstanceProfiles",
            "AWS::ApiOnly::DeviceFarmInstanceProfile",
            "AWS::DeviceFarm::InstanceProfile",
        ),
        (
            "transcribe",
            "ListLanguageModels",
            "AWS::ApiOnly::TranscribeLanguageModel",
            "AWS::Transcribe::LanguageModel",
        ),
        (
            "workspaces",
            "DescribeWorkspaceBundles",
            "AWS::ApiOnly::WorkSpacesBundle",
            "AWS::WorkSpaces::Bundle",
        ),
    ] {
        assert_eq!(
            map.operations()
                .iter()
                .find(|row| row.service == service && row.operation == operation)
                .map(|row| row.kind.as_str()),
            Some(kind),
            "{service}.{operation}"
        );
        assert!(
            map.api_only()
                .iter()
                .any(|row| row.kind == kind && !row.reason.is_empty()),
            "{kind}"
        );
        assert!(map.lookup(absent_cfn).is_none(), "{absent_cfn}");
    }
    Ok(())
}

#[test]
fn given_reviewed_metadata_operations_when_registry_compiles_then_expected_kinds_are_retained()
-> TestResult {
    let additions = [
        (
            "apigatewayv2",
            "ListPortals",
            "AWS::ApiOnly::ApiGatewayV2Portal",
        ),
        (
            "bedrock",
            "ListInferenceProfiles",
            "AWS::Bedrock::ApplicationInferenceProfile",
        ),
        ("iam", "GetRole", "AWS::IAM::Role"),
        ("iam", "ListRoles", "AWS::IAM::Role"),
        ("iot", "ListSecurityProfiles", "AWS::IoT::SecurityProfile"),
        ("iottwinmaker", "ListEntities", "AWS::IoTTwinMaker::Entity"),
        (
            "license-manager",
            "GetLicenseConfiguration",
            "AWS::LicenseManager::LicenseConfiguration",
        ),
        (
            "medical-imaging",
            "GetDatastore",
            "AWS::HealthImaging::Datastore",
        ),
        (
            "s3-control",
            "GetAccessGrantsInstance",
            "AWS::S3::AccessGrantsInstance",
        ),
        ("s3-control", "GetAccessGrant", "AWS::S3::AccessGrant"),
        (
            "s3-control",
            "GetAccessGrantsLocation",
            "AWS::S3::AccessGrantsLocation",
        ),
        (
            "s3-control",
            "GetStorageLensGroup",
            "AWS::S3::StorageLensGroup",
        ),
        ("transfer", "DescribeAgreement", "AWS::Transfer::Agreement"),
        ("sagemaker", "ListApps", "AWS::ApiOnly::SageMakerApp"),
        (
            "ssm-contacts",
            "ListContactChannels",
            "AWS::ApiOnly::SSMContactChannel",
        ),
        (
            "migrationhubstrategy",
            "ListServers",
            "AWS::ApiOnly::MigrationStrategyServer",
        ),
        (
            "mediapackage",
            "ListVodAssets",
            "AWS::ApiOnly::MediaPackageVodAsset",
        ),
        (
            "database-migration-service",
            "DescribeEndpoints",
            "AWS::DMS::Endpoint",
        ),
    ];
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for &(service, operation, kind) in additions.iter().chain(NEW_SAFE_METADATA_OPERATIONS) {
        assert_eq!(
            map.operations()
                .iter()
                .find(|row| row.service == service && row.operation == operation)
                .map(|row| row.kind.as_str()),
            Some(kind),
            "{service}.{operation}"
        );
    }
    Ok(())
}

#[test]
fn msk_mixed_list_uses_registered_canonical_variants_not_an_api_only_alias() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    assert_eq!(
        map.operations()
            .iter()
            .find(|row| row.service == "kafka" && row.operation == "ListClustersV2")
            .map(|row| row.kind.as_str()),
        Some("AWS::MSK::Cluster")
    );
    for kind in [
        "AWS::MSK::Cluster",
        "AWS::MSK::ServerlessCluster",
        "AWS::IAM::Role",
        "AWS::IoT::SecurityProfile",
        "AWS::IoTTwinMaker::Entity",
    ] {
        assert!(
            matches!(map.lookup(kind).map(|entry| &entry.target), Some(KindTarget::Mapped { kind: target, .. }) if target == kind)
        );
    }
    assert!(
        !map.api_only()
            .iter()
            .any(|row| row.kind == "AWS::ApiOnly::MSKCluster")
    );
    assert!(
        !map.relations()
            .iter()
            .any(|row| row.target_kinds.primary() == "AWS::ApiOnly::MSKCluster")
    );
    let edge = map
        .relations()
        .iter()
        .find(|row| {
            row.source_kind == "AWS::MSK::VpcConnection"
                && source_fields(row) == Some("TargetClusterArn")
                && row.relation == "CONNECTS_TO"
        })
        .ok_or("missing MSK target group")?;
    assert_eq!(
        edge.target_kinds.iter().collect::<Vec<_>>(),
        ["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster"]
    );
    Ok(())
}

#[test]
fn iot_metric_and_dimension_metadata_use_the_exact_pinned_cfn_kinds() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for kind in ["AWS::IoT::CustomMetric", "AWS::IoT::Dimension"] {
        assert!(
            matches!(map.lookup(kind).map(|entry| &entry.target),
            Some(KindTarget::Mapped { kind: target, .. }) if target == kind),
            "{kind}"
        );
        assert!(!map.api_only().iter().any(|row| row.kind == kind), "{kind}");
    }
    for (operation, kind) in [
        ("ListCustomMetrics", "AWS::IoT::CustomMetric"),
        ("DescribeCustomMetric", "AWS::IoT::CustomMetric"),
        ("ListDimensions", "AWS::IoT::Dimension"),
    ] {
        assert_eq!(
            map.operations()
                .iter()
                .find(|row| row.service == "iot" && row.operation == operation)
                .map(|row| row.kind.as_str()),
            Some(kind),
            "{operation}"
        );
    }
    Ok(())
}

#[test]
fn central_metadata_expansions_preserve_function_kind_and_delegated_account_origin() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    assert!(
        matches!(map.lookup("AWS::Lambda::Function").map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind, .. }) if kind == "AWS::Lambda::Function")
    );
    assert!(map.api_only().iter().any(|row| row.kind
        == "AWS::ApiOnly::OrganizationsDelegatedService"
        && row.reason.contains("account-to-service-principal")));
    let joins = map
        .relations()
        .iter()
        .filter(|row| row.source_kind == "AWS::ApiOnly::OrganizationsDelegatedService")
        .collect::<Vec<_>>();
    assert_eq!(joins.len(), 1);
    let row = joins
        .first()
        .ok_or("missing delegated-service parent join")?;
    assert!(matches!(row.source, ReviewedRelationSource::Request(_)));
    assert_eq!(
        (
            source_fields(row).ok_or("missing source field selector")?,
            row.target_kinds.primary(),
            target_fields(row).ok_or("missing target field selector")?,
            row.relation.as_str(),
            row.scope.as_str()
        ),
        (
            "AccountId",
            "AWS::ApiOnly::OrganizationsDelegatedAdministrator",
            "Id",
            "DELEGATED_FOR_ACCOUNT",
            "account"
        )
    );
    Ok(())
}

#[test]
fn iam_and_kms_metadata_children_preserve_exact_kinds_and_reference_origins() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for &kind in CENTRAL_ADDED_CFN_KINDS {
        assert!(
            matches!(map.lookup(kind).map(|entry| &entry.target),
            Some(KindTarget::Mapped { kind: target, .. }) if target == kind),
            "{kind}"
        );
    }
    for &kind in CENTRAL_ADDED_API_KINDS {
        assert_eq!(
            map.api_only()
                .iter()
                .filter(|entry| entry.kind == kind)
                .count(),
            1,
            "{kind}"
        );
    }
    for (kind, origin, source, target, path, label, scope) in [
        (
            "AWS::ApiOnly::IAMServiceSpecificCredential",
            false,
            "UserName",
            "AWS::IAM::User",
            "UserName",
            "HAS_USER",
            "account",
        ),
        (
            "AWS::IAM::GroupPolicy",
            true,
            "GroupName",
            "AWS::IAM::Group",
            "GroupName",
            "POLICY_OF",
            "account",
        ),
        (
            "AWS::IAM::RolePolicy",
            true,
            "RoleName",
            "AWS::IAM::Role",
            "RoleName",
            "POLICY_OF",
            "account",
        ),
        (
            "AWS::IAM::UserPolicy",
            true,
            "UserName",
            "AWS::IAM::User",
            "UserName",
            "POLICY_OF",
            "account",
        ),
        (
            "AWS::ApiOnly::IAMPolicyVersion",
            true,
            "PolicyArn",
            "AWS::IAM::ManagedPolicy",
            "Arn",
            "VERSION_OF",
            "account",
        ),
        (
            "AWS::ApiOnly::KMSKeyPolicy",
            true,
            "KeyId",
            "AWS::KMS::Key",
            "KeyArn",
            "POLICY_OF",
            "account_region",
        ),
    ] {
        let joins = map
            .relations()
            .iter()
            .filter(|row| row.source_kind == kind)
            .collect::<Vec<_>>();
        assert_eq!(joins.len(), 1, "{kind}");
        let row = joins
            .first()
            .ok_or("missing IAM or KMS metadata parent join")?;
        assert_eq!(
            matches!(row.source, ReviewedRelationSource::Request(_)),
            origin,
            "{kind}"
        );
        assert_eq!(
            (
                source_fields(row).ok_or("missing source field selector")?,
                row.target_kinds.primary(),
                target_fields(row).ok_or("missing target field selector")?,
                row.relation.as_str(),
                row.scope.as_str()
            ),
            (source, target, path, label, scope),
            "{kind}"
        );
    }
    Ok(())
}
#[test]
fn secret_safe_discovery_operations_have_canonical_resource_kinds() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (service, operation, kind) in [
        ("eks", "DescribeCluster", "AWS::EKS::Cluster"),
        (
            "elastic-load-balancing-v2",
            "DescribeListeners",
            "AWS::ElasticLoadBalancingV2::Listener",
        ),
        (
            "elastic-load-balancing-v2",
            "DescribeRules",
            "AWS::ElasticLoadBalancingV2::ListenerRule",
        ),
        ("ec2", "DescribeVpnConnections", "AWS::EC2::VPNConnection"),
        ("lambda", "ListVersionsByFunction", "AWS::Lambda::Version"),
        ("iam", "ListVirtualMFADevices", "AWS::IAM::VirtualMFADevice"),
    ] {
        assert_eq!(
            map.operations()
                .iter()
                .find(|row| row.service == service && row.operation == operation)
                .map(|row| row.kind.as_str()),
            Some(kind),
            "{service}.{operation}"
        );
        assert!(
            matches!(map.lookup(kind).map(|entry| &entry.target), Some(KindTarget::Mapped { kind: mapped, .. }) if mapped == kind),
            "{kind}"
        );
    }
    Ok(())
}
