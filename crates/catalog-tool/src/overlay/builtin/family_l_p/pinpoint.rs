use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateApp",
            "CreateCampaign",
            "CreateEmailTemplate",
            "CreateExportJob",
            "CreateImportJob",
            "CreateInAppTemplate",
            "CreateJourney",
            "CreatePushTemplate",
            "CreateRecommenderConfiguration",
            "CreateSegment",
            "CreateSmsTemplate",
            "CreateVoiceTemplate",
            "DeleteAdmChannel",
            "DeleteApnsChannel",
            "DeleteApnsSandboxChannel",
            "DeleteApnsVoipChannel",
            "DeleteApnsVoipSandboxChannel",
            "DeleteApp",
            "DeleteBaiduChannel",
            "DeleteCampaign",
            "DeleteEmailChannel",
            "DeleteEmailTemplate",
            "DeleteEndpoint",
            "DeleteEventStream",
            "DeleteGcmChannel",
            "DeleteInAppTemplate",
            "DeleteJourney",
            "DeletePushTemplate",
            "DeleteRecommenderConfiguration",
            "DeleteSegment",
            "DeleteSmsChannel",
            "DeleteSmsTemplate",
            "DeleteUserEndpoints",
            "DeleteVoiceChannel",
            "DeleteVoiceTemplate",
            "PutEventStream",
            "PutEvents",
            "RemoveAttributes",
            "SendMessages",
            "SendOTPMessage",
            "SendUsersMessages",
            "TagResource",
            "UntagResource",
            "UpdateAdmChannel",
            "UpdateApnsChannel",
            "UpdateApnsSandboxChannel",
            "UpdateApnsVoipChannel",
            "UpdateApnsVoipSandboxChannel",
            "UpdateApplicationSettings",
            "UpdateBaiduChannel",
            "UpdateCampaign",
            "UpdateEmailChannel",
            "UpdateEmailTemplate",
            "UpdateEndpoint",
            "UpdateEndpointsBatch",
            "UpdateGcmChannel",
            "UpdateInAppTemplate",
            "UpdateJourney",
            "UpdateJourneyState",
            "UpdatePushTemplate",
            "UpdateRecommenderConfiguration",
            "UpdateSegment",
            "UpdateSmsChannel",
            "UpdateSmsTemplate",
            "UpdateTemplateActiveVersion",
            "UpdateVoiceChannel",
            "UpdateVoiceTemplate",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAdmChannel",
            "GetApnsChannel",
            "GetApnsSandboxChannel",
            "GetApnsVoipChannel",
            "GetApnsVoipSandboxChannel",
            "GetBaiduChannel",
            "GetGcmChannel",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "GetApplicationSettings",
            "GetCampaigns",
            "GetChannels",
            "GetEmailChannel",
            "GetEventStream",
            "GetExportJobs",
            "GetImportJobs",
            "GetSegments",
            "GetSmsChannel",
            "GetVoiceChannel",
            "ListJourneys",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetApplicationDateRangeKpi"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, KpiName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetCampaign",
            "GetCampaignActivities",
            "GetCampaignVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, CampaignId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetCampaignDateRangeKpi"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, CampaignId, KpiName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetCampaignVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, CampaignId, Version); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetEmailTemplate",
            "GetEndpoint",
            "GetInAppMessages",
            "GetInAppTemplate",
            "GetPushTemplate",
            "GetSmsTemplate",
            "GetUserEndpoints",
            "GetVoiceTemplate",
            "ListTemplates",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetExportJob", "GetImportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JobId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetJourney", "GetJourneyExecutionMetrics", "GetJourneyRuns"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JourneyId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetJourneyDateRangeKpi"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JourneyId, KpiName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetJourneyExecutionActivityMetrics"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JourneyActivityId, JourneyId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetJourneyRunExecutionActivityMetrics"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JourneyActivityId, JourneyId, RunId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetJourneyRunExecutionMetrics"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, JourneyId, RunId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetSegment",
            "GetSegmentExportJobs",
            "GetSegmentImportJobs",
            "GetSegmentVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, SegmentId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetSegmentVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, SegmentId, Version); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTemplateVersions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TemplateName, TemplateType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["PhoneNumberValidate"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (NumberValidateRequest); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["VerifyOTPMessage"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ApplicationId, VerifyOTPMessageRequestParameters); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "GetApps",
        items: "ApplicationsResponse.Item",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("Token", "ApplicationsResponse.NextToken")),
        query: &["Arn", "Id", "Name"],
        tags: Some("tags"),
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetRecommenderConfigurations",
        items: "ListRecommenderConfigurationsResponse.Item",
        id: Id::Key(&["Id"]),
        scope: Scope::Region,
        tokens: Some(("Token", "ListRecommenderConfigurationsResponse.NextToken")),
        query: &[
            "Id",
            "Name",
            "RecommendationProviderIdType",
            "RecommendationProviderRoleArn",
            "RecommendationsDisplayName",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetApp",
        items: "ApplicationResponse",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: None,
        query: &["Arn", "Id", "Name"],
        tags: Some("tags"),
        bindings: &[("ApplicationId", "GetApps", "Id")],
        fixed: &[],
    },
    Resource {
        operation: "GetRecommenderConfiguration",
        items: "RecommenderConfigurationResponse",
        id: Id::Key(&["Id"]),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "Id",
            "Name",
            "RecommendationProviderIdType",
            "RecommendationProviderRoleArn",
            "RecommendationsDisplayName",
        ],
        tags: None,
        bindings: &[("RecommenderId", "GetRecommenderConfigurations", "Id")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pinpoint", GROUPS, RESOURCES)
}
