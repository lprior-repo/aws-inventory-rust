use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateMediaCapturePipeline",
            deferred(
                "Model requires SourceType, SourceArn, SinkType, SinkArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a media pipeline.",
            ),
        ),
        (
            "CreateMediaConcatenationPipeline",
            deferred(
                "Model requires Sources, Sinks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a media concatenation pipeline.",
            ),
        ),
        (
            "CreateMediaInsightsPipeline",
            deferred(
                "Model requires MediaInsightsPipelineConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a media insights pipeline.",
            ),
        ),
        (
            "CreateMediaInsightsPipelineConfiguration",
            deferred(
                "Model requires MediaInsightsPipelineConfigurationName, ResourceAccessRoleArn, Elements. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A structure that contains the static configurations for a media insights pipeline.",
            ),
        ),
        (
            "CreateMediaLiveConnectorPipeline",
            deferred(
                "Model requires Sources, Sinks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a media live connector pipeline in an Amazon Chime SDK meeting.",
            ),
        ),
        (
            "CreateMediaPipelineKinesisVideoStreamPool",
            deferred(
                "Model requires StreamConfiguration, PoolName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Kinesis Video Stream pool for use with media stream pipelines. If a meeting uses an opt-in Region as its MediaRegion , the KVS stream must be in that same Region. For example, if a meeting uses the af-south-1…",
            ),
        ),
        (
            "CreateMediaStreamPipeline",
            deferred(
                "Model requires Sources, Sinks. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a streaming media pipeline.",
            ),
        ),
        (
            "DeleteMediaCapturePipeline",
            deferred(
                "Model requires MediaPipelineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the media pipeline.",
            ),
        ),
        (
            "DeleteMediaInsightsPipelineConfiguration",
            deferred(
                "Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified configuration settings.",
            ),
        ),
        (
            "DeleteMediaPipeline",
            deferred(
                "Model requires MediaPipelineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the media pipeline.",
            ),
        ),
        (
            "DeleteMediaPipelineKinesisVideoStreamPool",
            deferred(
                "Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Amazon Kinesis Video Stream pool.",
            ),
        ),
        (
            "GetMediaCapturePipeline",
            deferred(
                "Model requires MediaPipelineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing media pipeline.",
            ),
        ),
        (
            "GetMediaInsightsPipelineConfiguration",
            deferred(
                "Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the configuration settings for a media insights pipeline.",
            ),
        ),
        (
            "GetMediaPipeline",
            deferred(
                "Model requires MediaPipelineId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an existing media pipeline.",
            ),
        ),
        (
            "GetMediaPipelineKinesisVideoStreamPool",
            deferred(
                "Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an Kinesis video stream pool.",
            ),
        ),
        (
            "GetSpeakerSearchTask",
            deferred(
                "Model requires Identifier, SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of the specified speaker search task.",
            ),
        ),
        (
            "GetVoiceToneAnalysisTask",
            deferred(
                "Model requires Identifier, VoiceToneAnalysisTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a voice tone analysis task.",
            ),
        ),
        (
            "ListMediaCapturePipelines",
            OperationOverlay {
                queryable: vec![path("MediaPipelineArn")?],
                ..read(
                    "MediaCapturePipelines",
                    Identity::Arn(path("MediaPipelineArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListMediaInsightsPipelineConfigurations",
            OperationOverlay {
                queryable: vec![path("MediaInsightsPipelineConfigurationArn")?],
                ..read(
                    "MediaInsightsPipelineConfigurations",
                    Identity::Arn(path("MediaInsightsPipelineConfigurationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListMediaPipelineKinesisVideoStreamPools",
            OperationOverlay {
                queryable: vec![path("PoolArn")?],
                ..read(
                    "KinesisVideoStreamPools",
                    Identity::Arn(path("PoolArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListMediaPipelines",
            OperationOverlay {
                queryable: vec![path("MediaPipelineArn")?],
                ..read(
                    "MediaPipelines",
                    Identity::Arn(path("MediaPipelineArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags available for a media pipeline.",
            ),
        ),
        (
            "StartSpeakerSearchTask",
            deferred(
                "Model requires Identifier, VoiceProfileDomainArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a speaker search task. Before starting any speaker search tasks, you must provide all notices and obtain all consents from the speaker as required under applicable privacy and biometrics laws, and as required under the AWS…",
            ),
        ),
        (
            "StartVoiceToneAnalysisTask",
            deferred(
                "Model requires Identifier, LanguageCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a voice tone analysis task. For more information about voice tone analysis, see Using Amazon Chime SDK voice analytics in the Amazon Chime SDK Developer Guide . Before starting any voice tone analysis tasks, you must…",
            ),
        ),
        (
            "StopSpeakerSearchTask",
            deferred(
                "Model requires Identifier, SpeakerSearchTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a speaker search task.",
            ),
        ),
        (
            "StopVoiceToneAnalysisTask",
            deferred(
                "Model requires Identifier, VoiceToneAnalysisTaskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a voice tone analysis task.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The ARN of the media pipeline that you want to tag. Consists of the pipeline's endpoint region, resource ID, and pipeline ID.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes any tags from a media pipeline.",
            ),
        ),
        (
            "UpdateMediaInsightsPipelineConfiguration",
            deferred(
                "Model requires Identifier, ResourceAccessRoleArn, Elements. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the media insights pipeline's configuration settings.",
            ),
        ),
        (
            "UpdateMediaInsightsPipelineStatus",
            deferred(
                "Model requires Identifier, UpdateStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of a media insights pipeline.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![(
        "UpdateMediaPipelineKinesisVideoStreamPool",
        deferred(
            "Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Amazon Kinesis Video Stream pool in a media pipeline.",
        ),
    )])
}
