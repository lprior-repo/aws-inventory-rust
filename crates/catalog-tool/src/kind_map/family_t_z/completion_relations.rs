use crate::kind_map::{CompoundRelationMapping, ParentRelationMapping, RelationComponentMapping, RelationGuardMapping, RelationMapping, RelationPath, RelationSource, RelationTarget};

pub(super) const PARENTS: &[ParentRelationMapping<'static>] = &[
    ParentRelationMapping { source_kind: "AWS::VpcLattice::AccessLogSubscription", target_kinds: &["AWS::VpcLattice::Service", "AWS::VpcLattice::ServiceNetwork"], relation: "FOR_RESOURCE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::WellArchitected::AgentGoal", target_kinds: &["AWS::WellArchitected::AgentProfile"], relation: "IN_PROFILE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailGroupMember", target_kinds: &["AWS::ApiOnly::WorkMailGroup"], relation: "IN_GROUP", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailResourceDelegate", target_kinds: &["AWS::ApiOnly::WorkMailResource"], relation: "FOR_RESOURCE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailAlias", target_kinds: &["AWS::ApiOnly::WorkMailUser", "AWS::ApiOnly::WorkMailGroup", "AWS::ApiOnly::WorkMailResource"], relation: "FOR_ENTITY", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailEntityGroup", target_kinds: &["AWS::ApiOnly::WorkMailUser", "AWS::ApiOnly::WorkMailGroup"], relation: "FOR_ENTITY", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailMailboxPermission", target_kinds: &["AWS::ApiOnly::WorkMailUser", "AWS::ApiOnly::WorkMailResource"], relation: "FOR_ENTITY", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VpcLatticeTargetRegistration", target_kinds: &["AWS::VpcLattice::TargetGroup"], relation: "IN_TARGET_GROUP", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::VpcLattice::Rule", target_kinds: &["AWS::VpcLattice::Listener"], relation: "IN_LISTENER", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailGroup", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailUser", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailResource", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailImpersonationRole", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailMailDomain", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailAvailabilityConfiguration", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailMobileDeviceAccessOverride", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailPersonalAccessToken", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailAccessControlRule", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkMailMobileDeviceAccessRule", target_kinds: &["AWS::ApiOnly::WorkMailOrganization"], relation: "IN_ORGANIZATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::Wisdom::Content", target_kinds: &["AWS::Wisdom::KnowledgeBase"], relation: "IN_KNOWLEDGE_BASE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::Wisdom::QuickResponse", target_kinds: &["AWS::Wisdom::KnowledgeBase"], relation: "IN_KNOWLEDGE_BASE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WisdomImportJob", target_kinds: &["AWS::Wisdom::KnowledgeBase"], relation: "IN_KNOWLEDGE_BASE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WickrBot", target_kinds: &["AWS::Wickr::Network"], relation: "IN_NETWORK", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VoiceIDWatchlist", target_kinds: &["AWS::VoiceID::Domain"], relation: "IN_DOMAIN", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VoiceIDSpeaker", target_kinds: &["AWS::VoiceID::Domain"], relation: "IN_DOMAIN", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VoiceIDFraudster", target_kinds: &["AWS::VoiceID::Domain"], relation: "IN_DOMAIN", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VoiceIDFraudsterRegistrationJob", target_kinds: &["AWS::VoiceID::Domain"], relation: "IN_DOMAIN", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::VoiceIDSpeakerEnrollmentJob", target_kinds: &["AWS::VoiceID::Domain"], relation: "IN_DOMAIN", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesConnectClientAddIn", target_kinds: &["AWS::WorkSpaces::Directory"], relation: "IN_DIRECTORY", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesPoolSession", target_kinds: &["AWS::WorkSpaces::WorkspacesPool"], relation: "IN_POOL", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesApplicationAssociation", target_kinds: &["AWS::ApiOnly::WorkSpacesApplication"], relation: "FOR_APPLICATION", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesBundleAssociation", target_kinds: &["AWS::ApiOnly::WorkSpacesBundle"], relation: "FOR_BUNDLE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesImageAssociation", target_kinds: &["AWS::ApiOnly::WorkSpacesImage"], relation: "FOR_IMAGE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesWorkspaceAssociation", target_kinds: &["AWS::WorkSpaces::Workspace"], relation: "FOR_WORKSPACE", scope: "account_region" },
    ParentRelationMapping { source_kind: "AWS::WAFRegional::WebACLAssociation", target_kinds: &["AWS::WAFRegional::WebACL"], relation: "FOR_WEB_ACL", scope: "account_region" },
];

pub(super) const OUTPUT: &[RelationMapping<'static>] = &[
    RelationMapping { source_kind: "AWS::VpcLattice::Rule", source: RelationSource::Request(RelationPath::Fields("serviceIdentifier")), target_kind: "AWS::VpcLattice::Service", target: RelationTarget::SelectedOutput(RelationPath::Fields("id")), relation: "IN_SERVICE", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkMailAlias", source: RelationSource::Request(RelationPath::Fields("OrganizationId")), target_kind: "AWS::ApiOnly::WorkMailOrganization", target: RelationTarget::SelectedOutput(RelationPath::Fields("OrganizationId")), relation: "IN_ORGANIZATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkMailGroupMember", source: RelationSource::Request(RelationPath::Fields("OrganizationId")), target_kind: "AWS::ApiOnly::WorkMailOrganization", target: RelationTarget::SelectedOutput(RelationPath::Fields("OrganizationId")), relation: "IN_ORGANIZATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkMailEntityGroup", source: RelationSource::Request(RelationPath::Fields("OrganizationId")), target_kind: "AWS::ApiOnly::WorkMailOrganization", target: RelationTarget::SelectedOutput(RelationPath::Fields("OrganizationId")), relation: "IN_ORGANIZATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkMailMailboxPermission", source: RelationSource::Request(RelationPath::Fields("OrganizationId")), target_kind: "AWS::ApiOnly::WorkMailOrganization", target: RelationTarget::SelectedOutput(RelationPath::Fields("OrganizationId")), relation: "IN_ORGANIZATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkMailResourceDelegate", source: RelationSource::Request(RelationPath::Fields("OrganizationId")), target_kind: "AWS::ApiOnly::WorkMailOrganization", target: RelationTarget::SelectedOutput(RelationPath::Fields("OrganizationId")), relation: "IN_ORGANIZATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::VpcLattice::AccessLogSubscription", source: RelationSource::SelectedOutput(RelationPath::Fields("destinationArn")), target_kind: "AWS::Logs::LogGroup", target: RelationTarget::CanonicalId, relation: "LOGS_TO", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::VpcLattice::AccessLogSubscription", source: RelationSource::SelectedOutput(RelationPath::Fields("destinationArn")), target_kind: "AWS::KinesisFirehose::DeliveryStream", target: RelationTarget::CanonicalId, relation: "LOGS_TO", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::VpcLattice::AccessLogSubscription", source: RelationSource::SelectedOutput(RelationPath::Fields("destinationArn")), target_kind: "AWS::S3::Bucket", target: RelationTarget::CanonicalId, relation: "LOGS_TO", scope: "account", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WAFLoggingConfiguration", source: RelationSource::SelectedOutput(RelationPath::Fields("ResourceArn")), target_kind: "AWS::WAF::WebACL", target: RelationTarget::SelectedOutput(RelationPath::Fields("WebACLArn")), relation: "FOR_WEB_ACL", scope: "account", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WAFRegionalLoggingConfiguration", source: RelationSource::SelectedOutput(RelationPath::Fields("ResourceArn")), target_kind: "AWS::WAFRegional::WebACL", target: RelationTarget::SelectedOutput(RelationPath::Fields("WebACLArn")), relation: "FOR_WEB_ACL", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WAFLoggingConfiguration", source: RelationSource::SelectedOutput(RelationPath::Fields("LogDestinationConfigs")), target_kind: "AWS::KinesisFirehose::DeliveryStream", target: RelationTarget::CanonicalId, relation: "LOGS_TO", scope: "arn_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WAFRegionalLoggingConfiguration", source: RelationSource::SelectedOutput(RelationPath::Fields("LogDestinationConfigs")), target_kind: "AWS::KinesisFirehose::DeliveryStream", target: RelationTarget::CanonicalId, relation: "LOGS_TO", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesApplicationAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::WorkSpaces::Workspace", target: RelationTarget::SelectedOutput(RelationPath::Fields("WorkspaceId")), relation: "ASSOCIATES_WORKSPACE", scope: "account_region", guard: Some(RelationGuardMapping { source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceType")), equals: "WORKSPACE" }) },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesApplicationAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::ApiOnly::WorkSpacesBundle", target: RelationTarget::SelectedOutput(RelationPath::Fields("BundleId")), relation: "ASSOCIATES_BUNDLE", scope: "account_region", guard: Some(RelationGuardMapping { source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceType")), equals: "BUNDLE" }) },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesApplicationAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::ApiOnly::WorkSpacesImage", target: RelationTarget::SelectedOutput(RelationPath::Fields("ImageId")), relation: "ASSOCIATES_IMAGE", scope: "account_region", guard: Some(RelationGuardMapping { source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceType")), equals: "IMAGE" }) },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesBundleAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::ApiOnly::WorkSpacesApplication", target: RelationTarget::SelectedOutput(RelationPath::Fields("ApplicationId")), relation: "ASSOCIATES_APPLICATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesImageAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::ApiOnly::WorkSpacesApplication", target: RelationTarget::SelectedOutput(RelationPath::Fields("ApplicationId")), relation: "ASSOCIATES_APPLICATION", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::WorkSpacesWorkspaceAssociation", source: RelationSource::SelectedOutput(RelationPath::Fields("AssociatedResourceId")), target_kind: "AWS::ApiOnly::WorkSpacesApplication", target: RelationTarget::SelectedOutput(RelationPath::Fields("ApplicationId")), relation: "ASSOCIATES_APPLICATION", scope: "account_region", guard: None },
];


pub(super) const COMPOUND: &[CompoundRelationMapping<'static>] = &[
    CompoundRelationMapping {
        source_kind: "AWS::ApiOnly::WorkMailEntityGroup", target_kind: "AWS::ApiOnly::WorkMailGroup",
        relation: "IN_GROUP", scope: "account_region",
        components: &[
            RelationComponentMapping {
                source: RelationSource::Request(RelationPath::Fields("OrganizationId")),
                target: RelationTarget::BoundContext(RelationPath::Fields("OrganizationId")),
            },
            RelationComponentMapping {
                source: RelationSource::SelectedOutput(RelationPath::Fields("GroupId")),
                target: RelationTarget::SelectedOutput(RelationPath::Fields("Id")),
            },
        ],
    },
];
