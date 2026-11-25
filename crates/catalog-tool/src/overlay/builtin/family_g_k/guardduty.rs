use super::policy;
use crate::overlay::{
    Identity, IdentityField, MemberPath, NameError, OperationOverlay, OverlayClass, ServiceOverlay,
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListDetectors",
            policy::query(
                policy::enumerate(
                    "DetectorIds",
                    policy::key("$")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["$"],
            )?,
        ),
        (
            "ListMalwareProtectionPlans",
            policy::query(
                policy::enumerate(
                    "MalwareProtectionPlans",
                    policy::key("MalwareProtectionPlanId")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["MalwareProtectionPlanId"],
            )?,
        ),
        ("ListFilters", detector_child("FilterNames", "$")?),
        (
            "ListPublishingDestinations",
            detector_child("Destinations", "DestinationId")?,
        ),
        ("ListIPSets", inherited_sets()),
        ("ListThreatIntelSets", inherited_sets()),
        ("ListThreatEntitySets", inherited_sets()),
        ("ListTrustedEntitySets", inherited_sets()),
    ];
    policy::finish(
        "guardduty",
        approved,
        DEFERRED,
        "Detector identifiers, detector-qualified filter names and publishing destination metadata are safe; findings and investigations remain content. Set ownership must not be inferred from a member detector.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

fn detector_child(items: &str, id: &str) -> Result<OperationOverlay, NameError> {
    let identity = Identity::CorrelatedKey(vec![
        IdentityField::Request(MemberPath::new("DetectorId")?),
        IdentityField::Selected(MemberPath::selected(id)?),
    ]);
    policy::query(
        policy::bind(
            policy::enumerate(items, identity, Some(("NextToken", "NextToken")))?,
            "DetectorId",
            "ListDetectors",
            "$",
        )?,
        &[id],
    )
}

fn inherited_sets() -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Pinned ListIPSets/ListThreatIntelSets/ListThreatEntitySets/ListTrustedEntitySets return administrator-owned IDs when called by a member. Responses contain no owner account or owning detector. GetAdministratorAccount does not provide the administrator detector ID and includes an invitation validation value. The member DetectorId cannot establish canonical set ownership; no local-account ARN or owned-resource row may be synthesized from these inherited references.".to_owned()),
        ..super::super::empty()
    }
}

const DEFERRED: &[&str] = &[
    "DescribeMalwareScans",
    "DescribeOrganizationConfiguration",
    "DescribePublishingDestination",
    "GetAdministratorAccount",
    "GetCoverageStatistics",
    "GetCustomDetectionRule",
    "GetCustomDetectionRuleAssociation",
    "GetCustomDetectionRuleOrgConfiguration",
    "GetDetector",
    "GetFilter",
    "GetFindingsStatistics",
    "GetIPSet",
    "GetInvitationsCount",
    "GetMalwareProtectionPlan",
    "GetMalwareScan",
    "GetMalwareScanSettings",
    "GetMasterAccount",
    "GetMemberDetectors",
    "GetMembers",
    "GetOrganizationStatistics",
    "GetRemainingFreeTrialDays",
    "GetThreatEntitySet",
    "GetThreatIntelSet",
    "GetTrustedEntitySet",
    "GetUsageStatistics",
    "ListCoverage",
    "ListCustomDetectionRuleAssociations",
    "ListCustomDetectionRuleOrgConfigurations",
    "ListCustomDetectionRules",
    "ListFindings",
    "ListInvestigations",
    "ListInvitations",
    "ListMalwareScans",
    "ListMembers",
    "ListOrganizationAdminAccounts",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AcceptAdministratorInvitation",
    "AcceptInvitation",
    "ArchiveFindings",
    "CreateCustomDetectionRuleAssociation",
    "CreateCustomDetectionRuleOrgConfiguration",
    "CreateDetector",
    "CreateFilter",
    "CreateIPSet",
    "CreateInvestigation",
    "CreateMalwareProtectionPlan",
    "CreateMembers",
    "CreatePublishingDestination",
    "CreateSampleFindings",
    "CreateThreatEntitySet",
    "CreateThreatIntelSet",
    "CreateTrustedEntitySet",
    "DeclineInvitations",
    "DeleteCustomDetectionRuleAssociation",
    "DeleteCustomDetectionRuleOrgConfiguration",
    "DeleteDetector",
    "DeleteFilter",
    "DeleteIPSet",
    "DeleteInvitations",
    "DeleteMalwareProtectionPlan",
    "DeleteMembers",
    "DeletePublishingDestination",
    "DeleteThreatEntitySet",
    "DeleteThreatIntelSet",
    "DeleteTrustedEntitySet",
    "DisableOrganizationAdminAccount",
    "DisassociateFromAdministratorAccount",
    "DisassociateFromMasterAccount",
    "DisassociateMembers",
    "EnableOrganizationAdminAccount",
    "InviteMembers",
    "SendObjectMalwareScan",
    "StartMalwareScan",
    "StartMonitoringMembers",
    "StopMonitoringMembers",
    "TagResource",
    "UnarchiveFindings",
    "UntagResource",
    "UpdateCustomDetectionRuleAssociation",
    "UpdateCustomDetectionRuleOrgConfiguration",
    "UpdateDetector",
    "UpdateFilter",
    "UpdateFindingsFeedback",
    "UpdateIPSet",
    "UpdateMalwareProtectionPlan",
    "UpdateMalwareScanSettings",
    "UpdateMemberDetectors",
    "UpdateOrganizationConfiguration",
    "UpdatePublishingDestination",
    "UpdateThreatEntitySet",
    "UpdateThreatIntelSet",
    "UpdateTrustedEntitySet",
];

const CONTENT: &[&str] = &[
    "GetFindings",
    "GetInvestigation",
    "ListInvestigationRecords",
];
