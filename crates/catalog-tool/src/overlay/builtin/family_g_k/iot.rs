//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iot/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{Identity, IdentityField, MemberPath, MissingSelected, NameError, OperationOverlay, RowPredicate, RowValueType, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListThingGroups", policy::query(policy::enumerate("thingGroups", policy::arn("groupArn")?, Some(("nextToken", "nextToken")))?, &["groupArn"])?),
        ("ListThingTypes", policy::query(policy::enumerate("thingTypes", policy::arn("thingTypeArn")?, Some(("nextToken", "nextToken")))?, &["thingTypeArn"])?),
        ("ListPolicies", policy::query(policy::enumerate("policies", policy::arn("policyArn")?, Some(("marker", "nextMarker")))?, &["policyArn", "policyName"])?),
        ("ListBillingGroups", policy::query(policy::enumerate("billingGroups", policy::arn("groupArn")?, Some(("nextToken", "nextToken")))?, &["groupArn"])?),
        ("ListRoleAliases", policy::query(policy::enumerate("roleAliases", policy::key("$")?, Some(("marker", "nextMarker")))?, &["$"])?),
        ("ListDomainConfigurations", policy::query(policy::enumerate("domainConfigurations", policy::arn("domainConfigurationArn")?, Some(("marker", "nextMarker")))?, &["domainConfigurationArn"])?),
        ("ListSecurityProfiles", policy::query(policy::enumerate("securityProfileIdentifiers", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, &["arn", "name"])?),
        // ARN resource forms are specified by the IoT service authorization reference:
        // https://docs.aws.amazon.com/service-authorization/latest/reference/list_iot.html#list_iot-resources-for-iam-policies
        ("ListCustomMetrics", policy::query(policy::enumerate("metricNames", Identity::Template("arn:{partition}:iot:{region}:{account}:custommetric/{$}".to_owned()), Some(("nextToken", "nextToken")))?, &["$"])?),
        ("ListDimensions", policy::query(policy::enumerate("dimensionNames", Identity::Template("arn:{partition}:iot:{region}:{account}:dimension/{$}".to_owned()), Some(("nextToken", "nextToken")))?, &["$"])?),
        // These list summaries contain identifiers/configuration metadata, never the
        // certificate PEM, command payload, job document, template body or script.
        ("ListAuthorizers", policy::query(policy::enumerate("authorizers", policy::arn("authorizerArn")?, Some(("marker", "nextMarker")))?, &["authorizerArn", "authorizerName"])?),
        ("ListCACertificates", policy::query(policy::enumerate("certificates", policy::arn("certificateArn")?, Some(("marker", "nextMarker")))?, &["certificateArn", "certificateId"])?),
        ("ListCertificates", policy::query(policy::enumerate("certificates", policy::arn("certificateArn")?, Some(("marker", "nextMarker")))?, &["certificateArn", "certificateId"])?),
        // No paginator trait exists for this modeled nextToken pair.
        ("ListCertificateProviders", policy::query(policy::enumerate("certificateProviders", policy::arn("certificateProviderArn")?, Some(("nextToken", "nextToken")))?, &["certificateProviderArn", "certificateProviderName"])?),
        ("ListCommands", policy::query(policy::enumerate("commands", policy::arn("commandArn")?, Some(("nextToken", "nextToken")))?, &["commandArn", "commandId"])?),
        ("ListFleetMetrics", policy::query(policy::enumerate("fleetMetrics", policy::arn("metricArn")?, Some(("nextToken", "nextToken")))?, &["metricArn", "metricName"])?),
        ("ListMitigationActions", policy::query(policy::enumerate("actionIdentifiers", policy::arn("actionArn")?, Some(("nextToken", "nextToken")))?, &["actionArn", "actionName"])?),
        ("ListJobTemplates", policy::query(policy::enumerate("jobTemplates", policy::arn("jobTemplateArn")?, Some(("nextToken", "nextToken")))?, &["jobTemplateArn", "jobTemplateId"])?),
        ("ListPackages", policy::query(policy::enumerate("packageSummaries", policy::key("packageName")?, Some(("nextToken", "nextToken")))?, &["packageName"])?),
        ("ListProvisioningTemplates", policy::query(policy::enumerate("templates", policy::arn("templateArn")?, Some(("nextToken", "nextToken")))?, &["templateArn", "templateName"])?),
        ("ListScheduledAudits", policy::query(policy::enumerate("scheduledAudits", policy::arn("scheduledAuditArn")?, Some(("nextToken", "nextToken")))?, &["scheduledAuditArn", "scheduledAuditName"])?),
        ("ListStreams", policy::query(policy::enumerate("streams", policy::arn("streamArn")?, Some(("nextToken", "nextToken")))?, &["streamArn", "streamId"])?),
        ("ListTopicRules", policy::query(policy::enumerate("rules", policy::arn("ruleArn")?, Some(("nextToken", "nextToken")))?, &["ruleArn", "ruleName"])?),
        ("ListOTAUpdates", policy::query(policy::enumerate("otaUpdates", policy::arn("otaUpdateArn")?, Some(("nextToken", "nextToken")))?, &["otaUpdateArn", "otaUpdateId"])?),
        ("ListJobs", policy::query(policy::enumerate("jobs", policy::arn("jobArn")?, Some(("nextToken", "nextToken")))?, &["jobArn", "jobId"])?),
        ("ListPackageVersions", package_versions()?),
        ("ListPolicyVersions", policy::query(policy::bind(policy::enumerate("policyVersions", qualified("policyName", "versionId")?, None)?, "policyName", "ListPolicies", "policyName")?, &["versionId"])?),
        // The complete modeled summary is integer versionId/creationDate/isDefaultVersion;
        // provisioning template documents are returned only by the unapproved detail call.
        ("ListProvisioningTemplateVersions", policy::query(policy::bind(policy::enumerate("versions", Identity::CorrelatedKey(vec![
            IdentityField::Request(crate::overlay::MemberPath::selected("templateName")?),
            IdentityField::SelectedInteger { path: crate::overlay::MemberPath::selected("versionId")?, minimum: i64::from(i32::MIN), maximum: i64::from(i32::MAX) },
        ]), Some(("nextToken", "nextToken")))?, "templateName", "ListProvisioningTemplates", "templateName")?, &["versionId"])?),
        ("ListAuditSuppressions", policy::query(policy::enumerate("suppressions", Identity::CorrelatedKey(vec![
            IdentityField::Selected(crate::overlay::MemberPath::selected("checkName")?),
            IdentityField::SelectedDigest(crate::overlay::MemberPath::selected("resourceIdentifier")?),
        ]), Some(("nextToken", "nextToken")))?, &["checkName"])?),
    ];
    policy::finish(
        "iot", approved, DEFERRED,
        "Account-owned identifier/configuration list summaries and parent-bound package/policy/provisioning-template version metadata are approved. Scalar metric/dimension names construct documented ARNs from trusted account/region context only. ListThings documents charges and is denied; DescribeEndpoint creates an endpoint and is denied; job/template bodies and credentials remain denied. Thing inventory is not claimed.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT), (policy::DenyReason::Secret, SECRET), (policy::DenyReason::Paid, PAID)],
    )
}

fn package_versions() -> Result<OperationOverlay, NameError> {
    let mut value = policy::query(policy::bind(policy::enumerate("packageVersionSummaries", qualified("packageName", "versionName")?,
        Some(("nextToken", "nextToken")))?, "packageName", "ListPackages", "packageName")?, &["packageName", "versionName"])?;
    value.row_policy.assertions.push(RowPredicate::SelectedRequestEquals {
        selected: MemberPath::selected("packageName")?,
        request: MemberPath::selected("packageName")?,
        value_type: RowValueType::String,
        missing: MissingSelected::AllowAbsent,
    });
    Ok(value)
}

fn qualified(parent: &str, member: &str) -> Result<Identity, NameError> {
    Ok(Identity::CorrelatedKey(vec![
        IdentityField::Request(crate::overlay::MemberPath::selected(parent)?),
        IdentityField::Selected(crate::overlay::MemberPath::selected(member)?),
    ]))
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeAccountAuditConfiguration", "DescribeAuditFinding", "DescribeAuditMitigationActionsTask",
    "DescribeAuditSuppression", "DescribeAuditTask", "DescribeAuthorizer", "DescribeBillingGroup",
    "DescribeCertificate", "DescribeCertificateProvider", "DescribeCustomMetric",
    "DescribeDefaultAuthorizer", "DescribeDetectMitigationActionsTask", "DescribeDimension",
    "DescribeDomainConfiguration", "DescribeEncryptionConfiguration", "DescribeEventConfigurations",
    "DescribeFleetMetric", "DescribeIndex", "DescribeJob", "DescribeJobExecution",
    "DescribeMitigationAction", "DescribeRoleAlias", "DescribeScheduledAudit", "DescribeSecurityProfile",
    "DescribeStream", "DescribeThing", "DescribeThingGroup", "DescribeThingType",
    "GetBehaviorModelTrainingSummaries", "GetBucketsAggregation", "GetCardinality", "GetCommandExecution",
    "GetEffectivePolicies", "GetIndexingConfiguration", "GetLoggingOptions", "GetOTAUpdate", "GetPackage",
    "GetPackageConfiguration", "GetPercentiles", "GetPolicy", "GetPolicyVersion", "GetRegistrationCode",
    "GetStatistics", "GetThingConnectivityData", "GetTopicRuleDestination", "GetV2LoggingOptions",
    "ListActiveViolations", "ListAttachedPolicies", "ListAuditFindings",
    "ListAuditMitigationActionsExecutions", "ListAuditMitigationActionsTasks",
    "ListAuditTasks",
    "ListCertificatesByCA", "ListCommandExecutions",
    "ListDetectMitigationActionsExecutions", "ListDetectMitigationActionsTasks",
    "ListIndices", "ListJobExecutionsForJob", "ListJobExecutionsForThing",
    "ListManagedJobTemplates", "ListMetricValues",
    "ListOutgoingCertificates",
    "ListPolicyPrincipals", "ListPrincipalPolicies", "ListPrincipalThings",
    "ListPrincipalThingsV2",
    "ListRelatedResourcesForAuditFinding", "ListSbomValidationResults",
    "ListSecurityProfilesForTarget", "ListTagsForResource",
    "ListTargetsForPolicy", "ListTargetsForSecurityProfile", "ListThingGroupsForThing",
    "ListThingPrincipals", "ListThingPrincipalsV2", "ListThingRegistrationTaskReports",
    "ListThingRegistrationTasks", "ListThingsInBillingGroup", "ListThingsInThingGroup",
    "ListTopicRuleDestinations", "ListV2LoggingLevels", "ListViolationEvents",
    "SearchIndex", "TestAuthorization", "ValidateSecurityProfileBehaviors",
];

const MUTATING: &[&str] = &[
    "AcceptCertificateTransfer", "AddThingToBillingGroup", "AddThingToThingGroup",
    "AssociateSbomWithPackageVersion", "AssociateTargetsWithJob", "AttachPolicy", "AttachPrincipalPolicy",
    "AttachSecurityProfile", "AttachThingPrincipal", "CancelAuditMitigationActionsTask", "CancelAuditTask",
    "CancelCertificateTransfer", "CancelDetectMitigationActionsTask", "CancelJob", "CancelJobExecution",
    "ClearDefaultAuthorizer", "ConfirmTopicRuleDestination", "CreateAuditSuppression", "CreateAuthorizer",
    "CreateBillingGroup", "CreateCertificateFromCsr", "CreateCertificateProvider", "CreateCommand",
    "CreateCustomMetric", "CreateDimension", "CreateDomainConfiguration", "CreateDynamicThingGroup",
    "CreateFleetMetric", "CreateJob", "CreateJobTemplate", "CreateKeysAndCertificate",
    "CreateMitigationAction", "CreateOTAUpdate", "CreatePackage", "CreatePackageVersion", "CreatePolicy",
    "CreatePolicyVersion", "CreateProvisioningClaim", "CreateProvisioningTemplate",
    "CreateProvisioningTemplateVersion", "CreateRoleAlias", "CreateScheduledAudit", "CreateSecurityProfile",
    "CreateStream", "CreateThing", "CreateThingGroup", "CreateThingType", "CreateTopicRule",
    "CreateTopicRuleDestination", "DeleteAccountAuditConfiguration", "DeleteAuditSuppression",
    "DeleteAuthorizer", "DeleteBillingGroup", "DeleteCACertificate", "DeleteCertificate",
    "DeleteCertificateProvider", "DeleteCommand", "DeleteCommandExecution", "DeleteCustomMetric",
    "DeleteDimension", "DeleteDomainConfiguration", "DeleteDynamicThingGroup", "DeleteFleetMetric",
    "DeleteJob", "DeleteJobExecution", "DeleteJobTemplate", "DeleteMitigationAction", "DeleteOTAUpdate",
    "DeletePackage", "DeletePackageVersion", "DeletePolicy", "DeletePolicyVersion",
    "DeleteProvisioningTemplate", "DeleteProvisioningTemplateVersion", "DeleteRegistrationCode",
    "DeleteRoleAlias", "DeleteScheduledAudit", "DeleteSecurityProfile", "DeleteStream", "DeleteThing",
    "DeleteThingGroup", "DeleteThingType", "DeleteTopicRule", "DeleteTopicRuleDestination",
    "DeleteV2LoggingLevel", "DeprecateThingType", "DescribeEndpoint", "DetachPolicy",
    "DetachPrincipalPolicy", "DetachSecurityProfile", "DetachThingPrincipal", "DisableTopicRule",
    "DisassociateSbomFromPackageVersion", "EnableTopicRule", "PutVerificationStateOnViolation",
    "RegisterCACertificate", "RegisterCertificate", "RegisterCertificateWithoutCA", "RegisterThing",
    "RejectCertificateTransfer", "RemoveThingFromBillingGroup", "RemoveThingFromThingGroup",
    "ReplaceTopicRule", "SetDefaultAuthorizer", "SetDefaultPolicyVersion", "SetLoggingOptions",
    "SetV2LoggingLevel", "SetV2LoggingOptions", "StartAuditMitigationActionsTask",
    "StartDetectMitigationActionsTask", "StartOnDemandAuditTask", "StartThingRegistrationTask",
    "StopThingRegistrationTask", "TagResource", "TestInvokeAuthorizer", "TransferCertificate",
    "UntagResource", "UpdateAccountAuditConfiguration", "UpdateAuditSuppression", "UpdateAuthorizer",
    "UpdateBillingGroup", "UpdateCACertificate", "UpdateCertificate", "UpdateCertificateProvider",
    "UpdateCommand", "UpdateCustomMetric", "UpdateDimension", "UpdateDomainConfiguration",
    "UpdateDynamicThingGroup", "UpdateEncryptionConfiguration", "UpdateEventConfigurations",
    "UpdateFleetMetric", "UpdateIndexingConfiguration", "UpdateJob", "UpdateMitigationAction",
    "UpdatePackage", "UpdatePackageConfiguration", "UpdatePackageVersion", "UpdateProvisioningTemplate",
    "UpdateRoleAlias", "UpdateScheduledAudit", "UpdateSecurityProfile", "UpdateStream", "UpdateThing",
    "UpdateThingGroup", "UpdateThingGroupsForThing", "UpdateThingType", "UpdateTopicRuleDestination",
];

const CONTENT: &[&str] = &[
    "DescribeCACertificate", "DescribeJobTemplate", "DescribeManagedJobTemplate",
    "DescribeProvisioningTemplate", "DescribeProvisioningTemplateVersion", "DescribeThingRegistrationTask",
    "GetCommand", "GetJobDocument", "GetPackageVersion",
];

const SECRET: &[&str] = &[
    "GetTopicRule",
];

const PAID: &[&str] = &[
    "ListThings",
];
