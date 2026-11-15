use super::*;
use crate::kind_map::{KindTarget, ReviewedRelation, ReviewedRelationSource, ReviewedRelationTarget, ReviewedValuePath};
use crate::names::Exceptions;
use sha2::{Digest, Sha256};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn source_fields(row: &ReviewedRelation) -> Option<&str> {
    match &row.source {
        ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(path))
        | ReviewedRelationSource::Request(ReviewedValuePath::Fields(path)) => Some(path),
        _ => None,
    }
}

fn target_fields(row: &ReviewedRelation) -> Option<&str> {
    match &row.target {
        ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(path)) => Some(path),
        _ => None,
    }
}

fn scalar_relation_row(row: &ReviewedRelation) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("{}\t{}\t{}\t{}\t{}\t{}\n", row.source_kind,
        source_fields(row).ok_or("legacy relation lost its modeled source")?, row.target_kinds.primary(),
        target_fields(row).ok_or("legacy relation lost its modeled target")?, row.relation, row.scope))
}

fn digest(rows: impl Iterator<Item = String>) -> String {
    let mut rows = rows.collect::<Vec<_>>();
    rows.sort();
    format!("{:x}", Sha256::digest(rows.concat().as_bytes()))
}

#[test]
fn embedded_baseline_preserves_all_313_exact_operation_mappings() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.operations().len(), 325);
    let actual = digest(map.operations().iter()
        .filter(|row| !NEW_SAFE_METADATA_OPERATIONS.iter().any(|(service, operation, _)| row.service == *service && row.operation == *operation))
        .map(|row| format!("{}\t{}\t{}\n", row.service, row.operation, row.kind)));
    assert_eq!(actual, "57bff7790dbbf197c41bd3225369ae32aa605116f9d656e602347f44614d967f");
    Ok(())
}

#[test]
fn embedded_baseline_preserves_168_canonical_cfn_kinds() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.entries().len(), 168 + CENTRAL_ADDED_CFN_KINDS.len());
    assert!(map.entries().iter().all(|entry| matches!(&entry.target, KindTarget::Mapped { kind, .. } if kind == &entry.cfn_type)));
    let actual = digest(map.entries().iter().filter(|row| !CENTRAL_ADDED_CFN_KINDS.contains(&row.cfn_type.as_str())).filter_map(|row| match &row.target {
        KindTarget::Mapped { kind, .. } => Some(format!("{}\tmap\t{}\t-\n", row.cfn_type, kind)),
        KindTarget::Unmapped { .. } => None,
    }));
    assert_eq!(actual, "c6fe97c29d1b9fa212d24dae314c9d3c909c3170958ebb80d8fa0c9c8bd0bb6d");
    Ok(())
}

#[test]
fn embedded_baseline_preserves_all_113_api_only_reasons() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.api_only().len(), 113 + CENTRAL_ADDED_API_KINDS.len());
    let actual = digest(map.api_only().iter()
        .filter(|row| !CENTRAL_ADDED_API_KINDS.contains(&row.kind.as_str()))
        .map(|row| format!("{}\tkind\t{}\n", row.kind, row.reason)));
    assert_eq!(actual, "ca17681b5cab4b5ebfa4c8eb05fa830631d062857f95408d1bd70d2ad98d76ea");
    Ok(())
}

#[test]
fn embedded_baseline_preserves_all_42_exact_scalar_relationships() -> TestResult {
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.relations().len(), 50);
    let actual = digest(map.relations().iter()
        .filter(|row| matches!(row.source, ReviewedRelationSource::SelectedOutput(_))
            && !["AWS::ApiOnly::IAMServiceSpecificCredential", "AWS::Route53::KeySigningKey"].contains(&row.source_kind.as_str()))
        .map(scalar_relation_row).collect::<Result<Vec<_>, _>>()?.into_iter());
    assert_eq!(actual, "4fdae8425ac3fd43b1c542574bbcd25b0048ca0684e0abdc30b7931e49802082");
    Ok(())
}

#[test]
fn embedded_registry_accounts_for_every_pinned_cfn_type_without_duplicate_canonical_nodes() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    assert_eq!(map.entries().len(), 1_881);
    assert!(map.entries().windows(2).all(|pair| matches!(pair, [left, right] if left.cfn_type < right.cfn_type)));
    assert!(map.operations().windows(2).all(|pair| matches!(pair, [left, right] if (&left.service, &left.operation) < (&right.service, &right.operation))));
    Ok(())
}

#[test]
fn native_list_and_detail_views_share_the_exact_canonical_kind() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    for (service, list, detail, kind) in [
        ("gamelift", "ListAliases", "DescribeAlias", "AWS::GameLift::Alias"),
        ("mq", "ListBrokers", "DescribeBroker", "AWS::AmazonMQ::Broker"),
        ("transfer", "ListServers", "DescribeServer", "AWS::Transfer::Server"),
        ("iam", "ListRoles", "GetRole", "AWS::IAM::Role"),
        ("medical-imaging", "ListDatastores", "GetDatastore", "AWS::HealthImaging::Datastore"),
        ("license-manager", "ListLicenseConfigurations", "GetLicenseConfiguration", "AWS::LicenseManager::LicenseConfiguration"),
        ("transfer", "ListAgreements", "DescribeAgreement", "AWS::Transfer::Agreement"),
        ("s3-control", "ListStorageLensGroups", "GetStorageLensGroup", "AWS::S3::StorageLensGroup"),
        ("wisdom", "ListAssistants", "GetAssistant", "AWS::Wisdom::Assistant"),
    ] {
        for operation in [list, detail] {
            assert_eq!(map.operations().iter().find(|row| row.service == service && row.operation == operation).map(|row| row.kind.as_str()), Some(kind), "{service}.{operation}");
        }
    }
    Ok(())
}

#[test]
fn pinned_schema_absences_remain_explicit_api_only_resources() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    for (service, operation, kind) in [
        ("devicefarm", "ListInstanceProfiles", "AWS::ApiOnly::DeviceFarmInstanceProfile"),
        ("transcribe", "ListLanguageModels", "AWS::ApiOnly::TranscribeLanguageModel"),
        ("workspaces", "DescribeWorkspaceBundles", "AWS::ApiOnly::WorkSpacesBundle"),
    ] {
        assert_eq!(map.operations().iter().find(|row| row.service == service && row.operation == operation).map(|row| row.kind.as_str()), Some(kind));
        assert!(map.api_only().iter().any(|row| row.kind == kind && !row.reason.is_empty()));
    }
    Ok(())
}

#[test]
fn native_scalar_relationships_preserve_exact_join_paths_and_scope() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    for (source, path, target, target_path, label, scope) in [
        ("AWS::Events::ApiDestination", "ConnectionArn", "AWS::Events::Connection", "ConnectionArn", "USES_CONNECTION", "account_region"),
        ("AWS::IoTFleetWise::DecoderManifest", "modelManifestArn", "AWS::IoTFleetWise::ModelManifest", "arn", "USES_MODEL_MANIFEST", "account_region"),
        ("AWS::Lightsail::InstanceSnapshot", "fromInstanceArn", "AWS::Lightsail::Instance", "arn", "FROM_INSTANCE", "account_region"),
        ("AWS::Wisdom::AssistantAssociation", "associationData.knowledgeBaseAssociation.knowledgeBaseArn", "AWS::Wisdom::KnowledgeBase", "knowledgeBaseArn", "USES_KNOWLEDGE_BASE", "account_region"),
    ] {
        assert!(map.relations().iter().any(|row| row.source_kind == source && source_fields(row) == Some(path) && row.target_kinds.primary() == target && target_fields(row) == Some(target_path) && row.relation == label && row.scope == scope), "{source}.{path}");
    }
    assert!(!map.relations().iter().any(|row| row.target_kinds.primary() == "AWS::DMS::Endpoint"));
    Ok(())
}

#[test]
fn all_six_reviewed_families_preserve_every_exact_operation_assignment() -> TestResult {
    let additions = [
        ("apigatewayv2", "ListPortals", "AWS::ApiOnly::ApiGatewayV2Portal"),
        ("bedrock", "ListInferenceProfiles", "AWS::Bedrock::ApplicationInferenceProfile"),
        ("iam", "GetRole", "AWS::IAM::Role"),
        ("iam", "ListRoles", "AWS::IAM::Role"),
        ("iot", "ListSecurityProfiles", "AWS::IoT::SecurityProfile"),
        ("iottwinmaker", "ListEntities", "AWS::IoTTwinMaker::Entity"),
        ("license-manager", "GetLicenseConfiguration", "AWS::LicenseManager::LicenseConfiguration"),
        ("medical-imaging", "GetDatastore", "AWS::HealthImaging::Datastore"),
        ("s3-control", "GetAccessGrantsInstance", "AWS::S3::AccessGrantsInstance"),
        ("s3-control", "GetAccessGrant", "AWS::S3::AccessGrant"),
        ("s3-control", "GetAccessGrantsLocation", "AWS::S3::AccessGrantsLocation"),
        ("s3-control", "GetStorageLensGroup", "AWS::S3::StorageLensGroup"),
        ("transfer", "DescribeAgreement", "AWS::Transfer::Agreement"),
    ];
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    assert_eq!(map.operations().len(), 1_750 + additions.len() + NEW_SAFE_METADATA_OPERATIONS.len());
    for &(service, operation, kind) in additions.iter().chain(NEW_SAFE_METADATA_OPERATIONS) {
        assert_eq!(map.operations().iter().find(|row| row.service == service && row.operation == operation).map(|row| row.kind.as_str()), Some(kind), "{service}.{operation}");
    }
    // Preserve every pre-repair assignment while reviewing the explicit MSK cutover separately.
    let original = map.operations().iter()
        .filter(|row| !additions.iter().chain(NEW_SAFE_METADATA_OPERATIONS).any(|(service, operation, _)| row.service == *service && row.operation == *operation))
        .map(|row| {
            let kind = if row.service == "kafka" && row.operation == "ListClustersV2" {
                "AWS::ApiOnly::MSKCluster"
            } else {
                row.kind.as_str()
            };
            format!("{}\t{}\t{kind}\n", row.service, row.operation)
        });
    assert_eq!(digest(original), "e0f7910fd2845e3282f98dc18a13d4cf86786f70fb37cc198e7c9001aebaa940");
    assert_eq!(map.source_sha256(), &crate::kind_map::fingerprint::digest(&map));
    Ok(())
}

#[test]
fn msk_mixed_list_uses_registered_canonical_variants_not_an_api_only_alias() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    assert_eq!(map.operations().iter().find(|row| row.service == "kafka" && row.operation == "ListClustersV2").map(|row| row.kind.as_str()), Some("AWS::MSK::Cluster"));
    for kind in ["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster", "AWS::IAM::Role", "AWS::IoT::SecurityProfile", "AWS::IoTTwinMaker::Entity"] {
        assert!(matches!(map.lookup(kind).map(|entry| &entry.target), Some(KindTarget::Mapped { kind: target, .. }) if target == kind));
    }
    assert!(!map.api_only().iter().any(|row| row.kind == "AWS::ApiOnly::MSKCluster"));
    assert!(!map.relations().iter().any(|row| row.target_kinds.primary() == "AWS::ApiOnly::MSKCluster"));
    let edge = map.relations().iter().find(|row| row.source_kind == "AWS::MSK::VpcConnection"
        && source_fields(row) == Some("TargetClusterArn") && row.relation == "CONNECTS_TO").ok_or("missing MSK target group")?;
    assert_eq!(edge.target_kinds.iter().collect::<Vec<_>>(), ["AWS::MSK::Cluster", "AWS::MSK::ServerlessCluster"]);
    Ok(())
}

#[test]
fn late_family_children_retain_scalar_parent_joins_without_invitation_aliases() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    assert_eq!(map.operations().iter().find(|row| row.service == "detective" && row.operation == "ListMembers").map(|row| row.kind.as_str()), Some("AWS::ApiOnly::DetectiveGraphMembership"));
    for (source, path, target, target_path, label) in [
        ("AWS::ApiOnly::DSQLStream", "clusterIdentifier", "AWS::DSQL::Cluster", "identifier", "IN_CLUSTER"),
        ("AWS::ApiOnly::EMRServerlessJobRun", "applicationId", "AWS::EMRServerless::Application", "id", "IN_APPLICATION"),
        ("AWS::ApiOnly::EMRServerlessSession", "applicationId", "AWS::EMRServerless::Application", "id", "IN_APPLICATION"),
        ("AWS::ApiOnly::DetectiveGraphMembership", "GraphArn", "AWS::Detective::Graph", "Arn", "MEMBER_OF"),
    ] {
        assert!(map.relations().iter().any(|row| row.source_kind == source && source_fields(row) == Some(path) && row.target_kinds.primary() == target && target_fields(row) == Some(target_path) && row.relation == label && row.scope == "account_region"), "{source}.{path}");
    }
    Ok(())
}

#[test]
fn serverless_execution_roles_join_global_iam_roles_in_account_scope() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    for (source, path) in [
        ("AWS::ApiOnly::EMRServerlessJobRun", "executionRole"),
        ("AWS::ApiOnly::EMRServerlessSession", "executionRoleArn"),
    ] {
        assert!(map.relations().iter().any(|row| row.source_kind == source && source_fields(row) == Some(path) && row.target_kinds.primary() == "AWS::IAM::Role" && target_fields(row) == Some("Arn") && row.relation == "ASSUMES_ROLE" && row.scope == "account"), "{source}.{path}");
    }
    Ok(())
}

const NEW_SAFE_METADATA_OPERATIONS: &[(&str, &str, &str)] = &[
    ("timestream-write", "ListDatabases", "AWS::Timestream::Database"),
    ("timestream-write", "DescribeDatabase", "AWS::Timestream::Database"),
    ("timestream-write", "ListTables", "AWS::Timestream::Table"),
    ("timestream-write", "DescribeTable", "AWS::Timestream::Table"),
    ("timestream-write", "ListBatchLoadTasks", "AWS::ApiOnly::TimestreamBatchLoadTask"),
    ("timestream-write", "DescribeBatchLoadTask", "AWS::ApiOnly::TimestreamBatchLoadTask"),
    ("timestream-query", "ListScheduledQueries", "AWS::Timestream::ScheduledQuery"),
    ("timestream-query", "DescribeScheduledQuery", "AWS::Timestream::ScheduledQuery"),
    ("wafv2", "ListIPSets", "AWS::WAFv2::IPSet"),
    ("wafv2", "GetIPSet", "AWS::WAFv2::IPSet"),
    ("wafv2", "ListLoggingConfigurations", "AWS::WAFv2::LoggingConfiguration"),
    ("wafv2", "GetLoggingConfiguration", "AWS::WAFv2::LoggingConfiguration"),
    ("wafv2", "ListManagedRuleSets", "AWS::ApiOnly::Wafv2ManagedRuleSet"),
    ("wafv2", "GetManagedRuleSet", "AWS::ApiOnly::Wafv2ManagedRuleSet"),
    ("wafv2", "ListRegexPatternSets", "AWS::WAFv2::RegexPatternSet"),
    ("wafv2", "GetRegexPatternSet", "AWS::WAFv2::RegexPatternSet"),
    ("wafv2", "ListRuleGroups", "AWS::WAFv2::RuleGroup"),
    ("wafv2", "GetRuleGroup", "AWS::WAFv2::RuleGroup"),
    ("wafv2", "ListWebACLs", "AWS::WAFv2::WebACL"),
    ("wafv2", "GetWebACL", "AWS::WAFv2::WebACL"),
    ("transfer", "ListAccesses", "AWS::ApiOnly::TransferAccess"),
    ("transfer", "DescribeAccess", "AWS::ApiOnly::TransferAccess"),
    ("transfer", "DescribeUser", "AWS::Transfer::User"),
    ("transfer", "DescribeHostKey", "AWS::Transfer::HostKey"),
    ("workspaces-web", "ListTrustStoreCertificates", "AWS::ApiOnly::WorkSpacesWebTrustStoreCertificate"),
    ("workspaces-web", "GetTrustStoreCertificate", "AWS::ApiOnly::WorkSpacesWebTrustStoreCertificate"),
    ("route-53-domains", "ListDomains", "AWS::ApiOnly::Route53Domain"),
    ("route53-recovery-readiness", "ListCells", "AWS::Route53RecoveryReadiness::Cell"),
    ("route53-recovery-readiness", "GetCell", "AWS::Route53RecoveryReadiness::Cell"),
    ("route53-recovery-readiness", "ListReadinessChecks", "AWS::Route53RecoveryReadiness::ReadinessCheck"),
    ("route53-recovery-readiness", "GetReadinessCheck", "AWS::Route53RecoveryReadiness::ReadinessCheck"),
    ("route53-recovery-readiness", "ListRecoveryGroups", "AWS::Route53RecoveryReadiness::RecoveryGroup"),
    ("route53-recovery-readiness", "GetRecoveryGroup", "AWS::Route53RecoveryReadiness::RecoveryGroup"),
    ("route53-recovery-readiness", "ListResourceSets", "AWS::Route53RecoveryReadiness::ResourceSet"),
    ("route53-recovery-readiness", "GetResourceSet", "AWS::Route53RecoveryReadiness::ResourceSet"),
    ("s3-control", "ListMultiRegionAccessPoints", "AWS::S3::MultiRegionAccessPoint"),
    ("s3-control", "ListAccessPointsForDirectoryBuckets", "AWS::S3::AccessPoint"),
    ("sqs", "GetQueueAttributes", "AWS::SQS::Queue"),
    ("sns", "ListSubscriptions", "AWS::SNS::Subscription"),
    ("sns", "ListSubscriptionsByTopic", "AWS::SNS::Subscription"),
    ("sns", "GetSubscriptionAttributes", "AWS::SNS::Subscription"),
    ("sns", "GetTopicAttributes", "AWS::SNS::Topic"),
    ("sns", "GetDataProtectionPolicy", "AWS::SNS::Topic"),
    ("route-53", "GetHostedZone", "AWS::Route53::HostedZone"),
    ("route-53", "GetReusableDelegationSet", "AWS::Route53::DelegationSet"),
    ("route-53", "ListTrafficPolicyInstancesByHostedZone", "AWS::Route53::TrafficPolicyInstance"),
    ("global-accelerator", "ListListeners", "AWS::GlobalAccelerator::Listener"),
    ("global-accelerator", "ListEndpointGroups", "AWS::GlobalAccelerator::EndpointGroup"),
    ("global-accelerator", "ListCustomRoutingListeners", "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener"),
    ("global-accelerator", "ListCustomRoutingEndpointGroups", "AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup"),
    ("accessanalyzer", "ListArchiveRules", "AWS::AccessAnalyzer::ArchiveRule"),
    ("accessanalyzer", "GetArchiveRule", "AWS::AccessAnalyzer::ArchiveRule"),
    ("iot", "ListCustomMetrics", "AWS::IoT::CustomMetric"),
    ("iot", "DescribeCustomMetric", "AWS::IoT::CustomMetric"),
    ("iot", "ListDimensions", "AWS::IoT::Dimension"),
    ("lambda", "ListFunctionsByCodeSigningConfig", "AWS::Lambda::Function"),
    ("organizations", "ListDelegatedServicesForAccount", "AWS::ApiOnly::OrganizationsDelegatedService"),
    ("iam", "ListAccountAliases", "AWS::ApiOnly::IAMAccountAlias"),
    ("iam", "ListServiceSpecificCredentials", "AWS::ApiOnly::IAMServiceSpecificCredential"),
    ("iam", "ListGroupPolicies", "AWS::IAM::GroupPolicy"),
    ("iam", "ListRolePolicies", "AWS::IAM::RolePolicy"),
    ("iam", "ListUserPolicies", "AWS::IAM::UserPolicy"),
    ("iam", "ListPolicyVersions", "AWS::ApiOnly::IAMPolicyVersion"),
    ("kms", "ListKeyPolicies", "AWS::ApiOnly::KMSKeyPolicy"),
    ("s3", "ListDirectoryBuckets", "AWS::S3Express::DirectoryBucket"),
    ("rds", "DescribeDBSnapshots", "AWS::RDS::DBSnapshot"),
    ("route-53", "GetDNSSEC", "AWS::Route53::DNSSEC"),
];

#[test]
fn global_accelerator_children_preserve_routing_families_and_bound_parent_joins() -> TestResult {
    // Given the complete pinned registry, when compiling the reviewed mappings.
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    // Then standard and custom routing children join only to their own parent family.
    for (source, path, target, label) in [
        ("AWS::GlobalAccelerator::Listener", "AcceleratorArn", "AWS::GlobalAccelerator::Accelerator", "IN_ACCELERATOR"),
        ("AWS::GlobalAccelerator::EndpointGroup", "ListenerArn", "AWS::GlobalAccelerator::Listener", "IN_LISTENER"),
        ("AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener", "AcceleratorArn", "AWS::ApiOnly::GlobalAcceleratorCustomRoutingAccelerator", "IN_ACCELERATOR"),
        ("AWS::ApiOnly::GlobalAcceleratorCustomRoutingEndpointGroup", "ListenerArn", "AWS::ApiOnly::GlobalAcceleratorCustomRoutingListener", "IN_LISTENER"),
    ] {
        let joins = map.relations().iter().filter(|row| row.source_kind == source).collect::<Vec<_>>();
        assert_eq!(joins.len(), 1, "{source}");
        let row = joins.first().ok_or("missing accelerator child parent join")?;
        assert!(matches!(row.source, ReviewedRelationSource::Request(_)));
        assert_eq!((source_fields(row).ok_or("missing source field selector")?, row.target_kinds.primary(), target_fields(row).ok_or("missing target field selector")?, row.relation.as_str(), row.scope.as_str()),
            (path, target, path, label, "account"));
    }
    Ok(())
}

#[test]
fn archive_rules_use_the_pinned_cfn_kind_and_actual_bound_analyzer_name() -> TestResult {
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    assert!(matches!(map.lookup("AWS::AccessAnalyzer::ArchiveRule").map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind, .. }) if kind == "AWS::AccessAnalyzer::ArchiveRule"));
    let joins = map.relations().iter().filter(|row| row.source_kind == "AWS::AccessAnalyzer::ArchiveRule").collect::<Vec<_>>();
    assert_eq!(joins.len(), 1);
    let row = joins.first().ok_or("missing archive-rule analyzer join")?;
    assert!(matches!(row.source, ReviewedRelationSource::Request(_)));
    assert_eq!((source_fields(row).ok_or("missing source field selector")?, row.target_kinds.primary(), target_fields(row).ok_or("missing target field selector")?, row.relation.as_str(), row.scope.as_str()),
        ("analyzerName", "AWS::AccessAnalyzer::Analyzer", "name", "IN_ANALYZER", "account_region"));
    Ok(())
}

#[test]
fn iot_metric_and_dimension_metadata_use_the_exact_pinned_cfn_kinds() -> TestResult {
    // Given the pinned denominator, when compiling native metric and dimension mappings.
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    // Then scalar list identities and metric detail stay in their precise CFN kinds.
    for kind in ["AWS::IoT::CustomMetric", "AWS::IoT::Dimension"] {
        assert!(matches!(map.lookup(kind).map(|entry| &entry.target),
            Some(KindTarget::Mapped { kind: target, .. }) if target == kind), "{kind}");
        assert!(!map.api_only().iter().any(|row| row.kind == kind), "{kind}");
    }
    for (operation, kind) in [
        ("ListCustomMetrics", "AWS::IoT::CustomMetric"),
        ("DescribeCustomMetric", "AWS::IoT::CustomMetric"),
        ("ListDimensions", "AWS::IoT::Dimension"),
    ] {
        assert_eq!(map.operations().iter().find(|row| row.service == "iot" && row.operation == operation)
            .map(|row| row.kind.as_str()), Some(kind), "{operation}");
    }
    Ok(())
}

#[test]
fn central_metadata_expansions_preserve_function_kind_and_delegated_account_origin() -> TestResult {
    // Given the central reviewed mappings, when compiling the full pinned denominator.
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    // Then code-signing discovery identifies functions, not a new signing association kind.
    assert!(matches!(map.lookup("AWS::Lambda::Function").map(|entry| &entry.target),
        Some(KindTarget::Mapped { kind, .. }) if kind == "AWS::Lambda::Function"));
    assert!(map.api_only().iter().any(|row| row.kind == "AWS::ApiOnly::OrganizationsDelegatedService"
        && row.reason.contains("account-to-service-principal")));
    let joins = map.relations().iter().filter(|row| row.source_kind == "AWS::ApiOnly::OrganizationsDelegatedService").collect::<Vec<_>>();
    assert_eq!(joins.len(), 1);
    let row = joins.first().ok_or("missing delegated-service parent join")?;
    // The bound request carries AccountId; selected service summaries do not.
    assert!(matches!(row.source, ReviewedRelationSource::Request(_)));
    assert_eq!((source_fields(row).ok_or("missing source field selector")?, row.target_kinds.primary(), target_fields(row).ok_or("missing target field selector")?, row.relation.as_str(), row.scope.as_str()),
        ("AccountId", "AWS::ApiOnly::OrganizationsDelegatedAdministrator", "Id", "DELEGATED_FOR_ACCOUNT", "account"));
    Ok(())
}

const CENTRAL_ADDED_CFN_KINDS: &[&str] = &[
    "AWS::Lambda::Function", "AWS::IAM::GroupPolicy", "AWS::IAM::Role",
    "AWS::IAM::RolePolicy", "AWS::IAM::UserPolicy",
    "AWS::S3Express::DirectoryBucket", "AWS::RDS::DBSnapshot", "AWS::Route53::KeySigningKey", "AWS::Route53::DNSSEC",
];
const CENTRAL_ADDED_API_KINDS: &[&str] = &[
    "AWS::ApiOnly::OrganizationsDelegatedService", "AWS::ApiOnly::IAMAccountAlias",
    "AWS::ApiOnly::IAMServiceSpecificCredential", "AWS::ApiOnly::IAMPolicyVersion",
    "AWS::ApiOnly::KMSKeyPolicy",
];

#[test]
fn iam_and_kms_metadata_children_preserve_exact_kinds_and_reference_origins() -> TestResult {
    // Origin remains an explicit typed namespace, not a parallel request-only collection.
    // Given the pinned CFN denominator, when compiling the reviewed metadata attachments.
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    for &kind in CENTRAL_ADDED_CFN_KINDS {
        assert!(matches!(map.lookup(kind).map(|entry| &entry.target),
            Some(KindTarget::Mapped { kind: target, .. }) if target == kind), "{kind}");
    }
    for &kind in CENTRAL_ADDED_API_KINDS {
        assert_eq!(map.api_only().iter().filter(|entry| entry.kind == kind).count(), 1, "{kind}");
    }
    // Then each child joins via its actual selected field or bound request, never a fabricated column.
    for (kind, origin, source, target, path, label, scope) in [
        ("AWS::ApiOnly::IAMServiceSpecificCredential", false, "UserName", "AWS::IAM::User", "UserName", "HAS_USER", "account"),
        ("AWS::IAM::GroupPolicy", true, "GroupName", "AWS::IAM::Group", "GroupName", "POLICY_OF", "account"),
        ("AWS::IAM::RolePolicy", true, "RoleName", "AWS::IAM::Role", "RoleName", "POLICY_OF", "account"),
        ("AWS::IAM::UserPolicy", true, "UserName", "AWS::IAM::User", "UserName", "POLICY_OF", "account"),
        ("AWS::ApiOnly::IAMPolicyVersion", true, "PolicyArn", "AWS::IAM::ManagedPolicy", "Arn", "VERSION_OF", "account"),
        ("AWS::ApiOnly::KMSKeyPolicy", true, "KeyId", "AWS::KMS::Key", "KeyArn", "POLICY_OF", "account_region"),
    ] {
        let joins = map.relations().iter().filter(|row| row.source_kind == kind).collect::<Vec<_>>();
        assert_eq!(joins.len(), 1, "{kind}");
        let row = joins.first().ok_or("missing IAM or KMS metadata parent join")?;
        assert_eq!(matches!(row.source, ReviewedRelationSource::Request(_)), origin, "{kind}");
        assert_eq!((source_fields(row).ok_or("missing source field selector")?, row.target_kinds.primary(), target_fields(row).ok_or("missing target field selector")?, row.relation.as_str(), row.scope.as_str()),
            (source, target, path, label, scope), "{kind}");
    }
    Ok(())
}

#[test]
fn route53_signing_key_joins_the_native_kms_arn_across_regions() -> TestResult {
    // Given a global Route53 key-signing key and its regional KMS key.
    let map = embedded(include_bytes!("../../../../../catalog/cloudcontrol-universal.json"))?;
    let mut joins = map.relations().iter()
        .filter(|row| row.source_kind == "AWS::Route53::KeySigningKey");
    // When resolving reviewed joins, use the selected ARN and actual ListKeys KeyArn field.
    let row = joins.next().ok_or("missing Route53 signing-key KMS join")?;
    assert!(matches!(row.source, ReviewedRelationSource::SelectedOutput(_)));
    assert_eq!((source_fields(row).ok_or("missing source field selector")?, row.target_kinds.primary(), target_fields(row).ok_or("missing target field selector")?, row.relation.as_str(), row.scope.as_str()),
        ("KmsArn", "AWS::KMS::Key", "KeyArn", "USES_KEY", "account"));
    // Then no unnormalized HostedZoneId-to-Id equality is silently admitted.
    assert!(joins.next().is_none());
    Ok(())
}

#[test]
fn route53_dnssec_emits_distinct_dnssec_and_key_resources_with_exact_leased_parent_rules() -> TestResult {
    // Given the reviewed source, when compiling DNSSEC's independent emitters.
    let map = SOURCE.compile(&Exceptions::builtin())?;
    assert_eq!(map.operations().iter().find(|row| row.service == "route-53" && row.operation == "GetDNSSEC")
        .map(|row| row.kind.as_str()), Some("AWS::Route53::DNSSEC"));
    assert_eq!(map.additional_emissions().len(), 1);
    assert_eq!(map.additional_emission_kind("route-53", "GetDNSSEC", "key_signing_keys"),
        Some("AWS::Route53::KeySigningKey"));
    assert_eq!(map.additional_emission_kind("route-53", "GetDNSSEC", "missing"), None);
    // Then both resources link only to the actual leased HostedZone, not raw /hostedzone/ ID equality.
    assert_eq!(map.parent_relations().len(), 2);
    for kind in ["AWS::Route53::DNSSEC", "AWS::Route53::KeySigningKey"] {
        let parent = map.parent_relations().iter().find(|row| row.source_kind == kind)
            .ok_or("missing reviewed HostedZone parent")?;
        assert_eq!(parent.target_kinds.iter().collect::<Vec<_>>(), ["AWS::Route53::HostedZone"]);
        assert_eq!((parent.relation.as_str(), parent.scope.as_str()), ("FOR_HOSTED_ZONE", "account"));
        assert!(!map.relations().iter().any(|row| row.source_kind == kind && row.relation == "FOR_HOSTED_ZONE"));
    }
    Ok(())
}
