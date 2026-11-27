use super::*;
const DISCOVERY_RELATIONS: &[(&str, &str, &str, &str, &str, &str, bool)] = &[
    ("AWS::EKS::Cluster", "roleArn", "AWS::IAM::Role", "Arn", "ASSUMES_ROLE", "account", false),
    ("AWS::EKS::Cluster", "resourcesVpcConfig.subnetIds", "AWS::EC2::Subnet", "SubnetId", "IN_SUBNET", "account_region", false),
    ("AWS::EKS::Cluster", "resourcesVpcConfig.securityGroupIds", "AWS::EC2::SecurityGroup", "GroupId", "USES_SECURITY_GROUP", "account_region", false),
    ("AWS::ElasticLoadBalancingV2::Listener", "LoadBalancerArn", "AWS::ElasticLoadBalancingV2::LoadBalancer", "LoadBalancerArn", "IN_LOAD_BALANCER", "account_region", false),
    ("AWS::ElasticLoadBalancingV2::ListenerRule", "ListenerArn", "AWS::ElasticLoadBalancingV2::Listener", "ListenerArn", "IN_LISTENER", "account_region", true),
    ("AWS::EC2::VPNConnection", "VpnGatewayId", "AWS::EC2::VPNGateway", "VpnGatewayId", "USES_VPN_GATEWAY", "account_region", false),
    ("AWS::EC2::VPNConnection", "CustomerGatewayId", "AWS::EC2::CustomerGateway", "CustomerGatewayId", "USES_CUSTOMER_GATEWAY", "account_region", false),
];
#[test]
fn native_scalar_relationships_preserve_exact_join_paths_and_scope() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (source, path, target, target_path, label, scope) in [
        (
            "AWS::Events::ApiDestination",
            "ConnectionArn",
            "AWS::Events::Connection",
            "ConnectionArn",
            "USES_CONNECTION",
            "account_region",
        ),
        (
            "AWS::IoTFleetWise::DecoderManifest",
            "modelManifestArn",
            "AWS::IoTFleetWise::ModelManifest",
            "arn",
            "USES_MODEL_MANIFEST",
            "account_region",
        ),
        (
            "AWS::Lightsail::InstanceSnapshot",
            "fromInstanceArn",
            "AWS::Lightsail::Instance",
            "arn",
            "FROM_INSTANCE",
            "account_region",
        ),
        (
            "AWS::Wisdom::AssistantAssociation",
            "associationData.knowledgeBaseAssociation.knowledgeBaseArn",
            "AWS::Wisdom::KnowledgeBase",
            "knowledgeBaseArn",
            "USES_KNOWLEDGE_BASE",
            "account_region",
        ),
    ] {
        assert!(
            map.relations().iter().any(|row| row.source_kind == source
                && source_fields(row) == Some(path)
                && row.target_kinds.primary() == target
                && target_fields(row) == Some(target_path)
                && row.relation == label
                && row.scope == scope),
            "{source}.{path}"
        );
    }
    assert!(
        !map.relations()
            .iter()
            .any(|row| row.target_kinds.primary() == "AWS::DMS::Endpoint")
    );
    Ok(())
}

#[test]
fn late_family_children_retain_scalar_parent_joins_without_invitation_aliases() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    assert_eq!(
        map.operations()
            .iter()
            .find(|row| row.service == "detective" && row.operation == "ListMembers")
            .map(|row| row.kind.as_str()),
        Some("AWS::ApiOnly::DetectiveGraphMembership")
    );
    for (source, path, target, target_path, label) in [
        (
            "AWS::ApiOnly::DSQLStream",
            "clusterIdentifier",
            "AWS::DSQL::Cluster",
            "identifier",
            "IN_CLUSTER",
        ),
        (
            "AWS::ApiOnly::EMRServerlessJobRun",
            "applicationId",
            "AWS::EMRServerless::Application",
            "id",
            "IN_APPLICATION",
        ),
        (
            "AWS::ApiOnly::EMRServerlessSession",
            "applicationId",
            "AWS::EMRServerless::Application",
            "id",
            "IN_APPLICATION",
        ),
        (
            "AWS::ApiOnly::DetectiveGraphMembership",
            "GraphArn",
            "AWS::Detective::Graph",
            "Arn",
            "MEMBER_OF",
        ),
    ] {
        assert!(
            map.relations().iter().any(|row| row.source_kind == source
                && source_fields(row) == Some(path)
                && row.target_kinds.primary() == target
                && target_fields(row) == Some(target_path)
                && row.relation == label
                && row.scope == "account_region"),
            "{source}.{path}"
        );
    }
    Ok(())
}

#[test]
fn serverless_execution_roles_join_global_iam_roles_in_account_scope() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (source, path) in [
        ("AWS::ApiOnly::EMRServerlessJobRun", "executionRole"),
        ("AWS::ApiOnly::EMRServerlessSession", "executionRoleArn"),
    ] {
        assert!(
            map.relations().iter().any(|row| row.source_kind == source
                && source_fields(row) == Some(path)
                && row.target_kinds.primary() == "AWS::IAM::Role"
                && target_fields(row) == Some("Arn")
                && row.relation == "ASSUMES_ROLE"
                && row.scope == "account"),
            "{source}.{path}"
        );
    }
    Ok(())
}

#[test]
fn global_accelerator_children_preserve_routing_families_and_bound_parent_joins() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for (source, path, target, label) in [
        (
            "AWS::GlobalAccelerator::Listener",
            "AcceleratorArn",
            "AWS::GlobalAccelerator::Accelerator",
            "IN_ACCELERATOR",
        ),
        (
            "AWS::GlobalAccelerator::EndpointGroup",
            "ListenerArn",
            "AWS::GlobalAccelerator::Listener",
            "IN_LISTENER",
        ),
        (
            "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
            "AcceleratorArn",
            "AWS::ApiOnly::GlobalAcceleratorCustomRoutingAccelerator",
            "IN_ACCELERATOR",
        ),
        (
            "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup",
            "ListenerArn",
            "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener",
            "IN_LISTENER",
        ),
    ] {
        let joins = map
            .relations()
            .iter()
            .filter(|row| row.source_kind == source)
            .collect::<Vec<_>>();
        assert_eq!(joins.len(), 1, "{source}");
        let row = joins
            .first()
            .ok_or("missing accelerator child parent join")?;
        assert!(matches!(row.source, ReviewedRelationSource::Request(_)));
        assert_eq!(
            (
                source_fields(row).ok_or("missing source field selector")?,
                row.target_kinds.primary(),
                target_fields(row).ok_or("missing target field selector")?,
                row.relation.as_str(),
                row.scope.as_str()
            ),
            (path, target, path, label, "account")
        );
    }
    Ok(())
}

#[test]
fn archive_rules_use_the_pinned_cfn_kind_and_actual_bound_analyzer_name() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    assert!(
        matches!(map.lookup("AWS::AccessAnalyzer::ArchiveRule").map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind, .. }) if kind == "AWS::AccessAnalyzer::ArchiveRule")
    );
    let joins = map
        .relations()
        .iter()
        .filter(|row| row.source_kind == "AWS::AccessAnalyzer::ArchiveRule")
        .collect::<Vec<_>>();
    assert_eq!(joins.len(), 1);
    let row = joins.first().ok_or("missing archive-rule analyzer join")?;
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
            "analyzerName",
            "AWS::AccessAnalyzer::Analyzer",
            "name",
            "IN_ANALYZER",
            "account_region"
        )
    );
    Ok(())
}

#[test]
fn route53_signing_key_joins_the_native_kms_arn_across_regions() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    let mut joins = map
        .relations()
        .iter()
        .filter(|row| row.source_kind == "AWS::Route53::KeySigningKey");
    let row = joins.next().ok_or("missing Route53 signing-key KMS join")?;
    assert!(matches!(
        row.source,
        ReviewedRelationSource::SelectedOutput(_)
    ));
    assert_eq!(
        (
            source_fields(row).ok_or("missing source field selector")?,
            row.target_kinds.primary(),
            target_fields(row).ok_or("missing target field selector")?,
            row.relation.as_str(),
            row.scope.as_str()
        ),
        (
            "KmsArn",
            "AWS::KMS::Key",
            "KeyArn",
            "USES_KEY",
            "arn_region"
        )
    );
    assert!(joins.next().is_none());
    Ok(())
}

#[test]
fn route53_dnssec_emits_distinct_dnssec_and_key_resources_with_exact_leased_parent_rules()
-> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(
        map.operations()
            .iter()
            .find(|row| row.service == "route-53" && row.operation == "GetDNSSEC")
            .map(|row| row.kind.as_str()),
        Some("AWS::Route53::DNSSEC")
    );
    assert_eq!(map.additional_emissions().len(), 1);
    assert_eq!(
        map.additional_emission_kind("route-53", "GetDNSSEC", "key_signing_keys"),
        Some("AWS::Route53::KeySigningKey")
    );
    assert_eq!(
        map.additional_emission_kind("route-53", "GetDNSSEC", "missing"),
        None
    );
    assert_eq!(map.parent_relations().len(), 2);
    for kind in ["AWS::Route53::DNSSEC", "AWS::Route53::KeySigningKey"] {
        let parent = map
            .parent_relations()
            .iter()
            .find(|row| row.source_kind == kind)
            .ok_or("missing reviewed HostedZone parent")?;
        assert_eq!(
            parent.target_kinds.iter().collect::<Vec<_>>(),
            ["AWS::Route53::HostedZone"]
        );
        assert_eq!(
            (parent.relation.as_str(), parent.scope.as_str()),
            ("FOR_HOSTED_ZONE", "account")
        );
        assert!(
            !map.relations()
                .iter()
                .any(|row| row.source_kind == kind && row.relation == "FOR_HOSTED_ZONE")
        );
    }
    Ok(())
}
#[test]
fn discovered_resources_retain_their_parent_relationships() -> TestResult {
    let map = embedded(include_bytes!(
        "../../../../../../catalog/cloudcontrol-universal.json"
    ))?;
    for &(source, path, target, target_path, label, scope, request) in DISCOVERY_RELATIONS {
        assert!(
            map.relations().iter().any(|row| {
                row.source_kind == source
                    && source_fields(row) == Some(path)
                    && row.target_kinds.primary() == target
                    && target_fields(row) == Some(target_path)
                    && row.relation == label
                    && row.scope == scope
                    && matches!(row.source, ReviewedRelationSource::Request(_)) == request
            }),
            "{source}.{path}"
        );
    }
    Ok(())
}
