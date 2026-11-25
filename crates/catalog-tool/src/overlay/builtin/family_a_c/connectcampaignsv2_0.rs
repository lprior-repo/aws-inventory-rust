use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateCampaign",
            deferred(
                "Model requires name, connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a campaign for the specified Amazon Connect account. This API is idempotent.",
            ),
        ),
        (
            "DeleteCampaign",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a campaign from the specified Amazon Connect account.",
            ),
        ),
        (
            "DeleteCampaignChannelSubtypeConfig",
            deferred(
                "Model requires id, channelSubtype. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the channel subtype config of a campaign. This API is idempotent.",
            ),
        ),
        (
            "DeleteCampaignCommunicationLimits",
            deferred(
                "Model requires id, config. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the communication limits config for a campaign. This API is idempotent.",
            ),
        ),
        (
            "DeleteCampaignCommunicationTime",
            deferred(
                "Model requires id, config. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the communication time config for a campaign. This API is idempotent.",
            ),
        ),
        (
            "DeleteCampaignEntryLimits",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the entry limits config for a campaign. This API is idempotent.",
            ),
        ),
        (
            "DeleteConnectInstanceConfig",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a connect instance config from the specified AWS account.",
            ),
        ),
        (
            "DeleteConnectInstanceIntegration",
            deferred(
                "Model requires connectInstanceId, integrationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete the integration for the specified Amazon Connect instance.",
            ),
        ),
        (
            "DeleteInstanceOnboardingJob",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete the Connect Campaigns onboarding job for the specified Amazon Connect instance.",
            ),
        ),
        (
            "DescribeCampaign",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("id")?,
                    Binding::From {
                        operation: name("ListCampaigns")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                exclude_response_members: vec![
                    path(
                        "campaign.channelSubtypeConfig.email.defaultOutboundConfig.connectSourceEmailAddress",
                    )?,
                    path(
                        "campaign.channelSubtypeConfig.telephony.defaultOutboundConfig.connectSourcePhoneNumber",
                    )?,
                ],
                ..read(
                    "campaign",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetCampaignState",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get state of a campaign for the specified Amazon Connect account.",
            ),
        ),
        (
            "GetCampaignStateBatch",
            deferred(
                "Model requires campaignIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get state of campaigns for the specified Amazon Connect account.",
            ),
        ),
        (
            "GetConnectInstanceConfig",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the specific Connect instance config.",
            ),
        ),
        (
            "GetInstanceCommunicationLimits",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the instance communication limits.",
            ),
        ),
        (
            "GetInstanceOnboardingJobStatus",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the specific instance onboarding job status.",
            ),
        ),
        (
            "ListCampaigns",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "campaignSummaryList",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListConnectInstanceIntegrations",
            deferred(
                "Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides summary information about the integration under the specified Connect instance.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags for a resource.",
            ),
        ),
        (
            "PauseCampaign",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Pauses a campaign for the specified Amazon Connect account.",
            ),
        ),
        (
            "PutConnectInstanceIntegration",
            deferred(
                "Model requires connectInstanceId, integrationConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Put or update the integration for the specified Amazon Connect instance.",
            ),
        ),
        (
            "PutInstanceCommunicationLimits",
            deferred(
                "Model requires connectInstanceId, communicationLimitsConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Put the instance communication limits. This API is idempotent.",
            ),
        ),
        (
            "PutOutboundRequestBatch",
            deferred(
                "Model requires id, outboundRequests. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates outbound requests for the specified campaign Amazon Connect account. This API is idempotent.",
            ),
        ),
        (
            "PutProfileOutboundRequestBatch",
            deferred(
                "Model requires id, profileOutboundRequests. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Takes in a list of profile outbound requests to be placed as part of an outbound campaign. This API is idempotent.",
            ),
        ),
        (
            "ResumeCampaign",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a campaign for the specified Amazon Connect account.",
            ),
        ),
        (
            "StartCampaign",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a campaign for the specified Amazon Connect account.",
            ),
        ),
        (
            "StartInstanceOnboardingJob",
            deferred(
                "Model requires connectInstanceId, encryptionConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Onboard the specific Amazon Connect instance to Connect Campaigns.",
            ),
        ),
        (
            "StopCampaign",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a campaign for the specified Amazon Connect account.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires arn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tag a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires arn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untag a resource.",
            ),
        ),
        (
            "UpdateCampaignChannelSubtypeConfig",
            deferred(
                "Model requires id, channelSubtypeConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the channel subtype config of a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignCommunicationLimits",
            deferred(
                "Model requires id, communicationLimitsOverride. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the communication limits config for a campaign. This API is idempotent.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateCampaignCommunicationTime",
            deferred(
                "Model requires id, communicationTimeConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the communication time config for a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignEntryLimits",
            deferred(
                "Model requires id, entryLimitsConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the entry limits config for a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignFlowAssociation",
            deferred(
                "Model requires id, connectCampaignFlowArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the campaign flow associated with a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignName",
            deferred(
                "Model requires id, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name of a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignSchedule",
            deferred(
                "Model requires id, schedule. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the schedule for a campaign. This API is idempotent.",
            ),
        ),
        (
            "UpdateCampaignSource",
            deferred(
                "Model requires id, source. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the campaign source with a campaign. This API is idempotent.",
            ),
        ),
    ])
}
