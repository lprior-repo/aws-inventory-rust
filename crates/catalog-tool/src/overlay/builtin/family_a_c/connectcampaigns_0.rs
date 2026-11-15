// Pinned source: .local/aws-models/models/connectcampaigns/service/2021-01-30/connectcampaigns-2021-01-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateCampaign", deferred("Model requires name, connectInstanceId, dialerConfig, outboundCallConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a campaign for the specified Amazon Connect account. This API is idempotent.")),
        ("DeleteCampaign", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a campaign from the specified Amazon Connect account.")),
        ("DeleteConnectInstanceConfig", deferred("Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a connect instance config from the specified AWS account.")),
        ("DeleteInstanceOnboardingJob", deferred("Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete the Connect Campaigns onboarding job for the specified Amazon Connect instance.")),
        // Model: Describes the specific campaign.
        ("DescribeCampaign", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("id")?, Binding::From { operation: name("ListCampaigns")?, path: path("id")? })].into_iter().collect(),
            tags: Some(path("tags")?),
            queryable: vec![path("arn")?],
            ..read("campaign", Identity::Arn(path("arn")?), Scope::Region, Paginator::None)?
        }),
        ("GetCampaignState", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get state of a campaign for the specified Amazon Connect account.")),
        ("GetCampaignStateBatch", deferred("Model requires campaignIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get state of campaigns for the specified Amazon Connect account.")),
        ("GetConnectInstanceConfig", deferred("Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the specific Connect instance config.")),
        ("GetInstanceOnboardingJobStatus", deferred("Model requires connectInstanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the specific instance onboarding job status.")),
        // Model: Provides summary information about the campaigns under the specified Amazon Connect account.
        ("ListCampaigns", OperationOverlay {
            queryable: vec![path("arn")?],
            ..read("campaignSummaryList", Identity::Arn(path("arn")?), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags for a resource.")),
        ("PauseCampaign", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Pauses a campaign for the specified Amazon Connect account.")),
        ("PutDialRequestBatch", deferred("Model requires id, dialRequests. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates dials requests for the specified campaign Amazon Connect account. This API is idempotent.")),
        ("ResumeCampaign", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a campaign for the specified Amazon Connect account.")),
        ("StartCampaign", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a campaign for the specified Amazon Connect account.")),
        ("StartInstanceOnboardingJob", deferred("Model requires connectInstanceId, encryptionConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Onboard the specific Amazon Connect instance to Connect Campaigns.")),
        ("StopCampaign", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a campaign for the specified Amazon Connect account.")),
        ("TagResource", deferred("Model requires arn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tag a resource.")),
        ("UntagResource", deferred("Model requires arn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untag a resource.")),
        ("UpdateCampaignDialerConfig", deferred("Model requires id, dialerConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the dialer config of a campaign. This API is idempotent.")),
        ("UpdateCampaignName", deferred("Model requires id, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name of a campaign. This API is idempotent.")),
        ("UpdateCampaignOutboundCallConfig", deferred("Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the outbound call config of a campaign. This API is idempotent.")),
    ])
}
