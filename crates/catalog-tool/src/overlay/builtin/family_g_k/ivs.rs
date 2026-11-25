use super::policy;
use crate::overlay::{
    MemberPath, MissingSelected, NameError, RowPredicate, RowValueType, ServiceOverlay,
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut stream_keys = policy::query(
        policy::tags(
            policy::bind(
                policy::enumerate(
                    "streamKeys",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                "channelArn",
                "ListChannels",
                "arn",
            )?,
            "tags",
        )?,
        &["arn", "channelArn"],
    )?;
    stream_keys
        .row_policy
        .assertions
        .push(RowPredicate::SelectedRequestEquals {
            selected: MemberPath::selected("channelArn")?,
            request: MemberPath::selected("channelArn")?,
            value_type: RowValueType::String,
            missing: MissingSelected::AllowAbsent,
        });
    let approved = vec![
        (
            "ListChannels",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "channels",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListRecordingConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "recordingConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListPlaybackKeyPairs",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "keyPairs",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListPlaybackRestrictionPolicies",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "playbackRestrictionPolicies",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListAdConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "adConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "name",
                    "postRollConfiguration.enabled",
                    "postRollConfiguration.durationSeconds",
                ],
            )?,
        ),
        (
            "ListFailoverSlateConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "failoverSlateConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "name",
                    "activationThresholdSeconds",
                    "minSlateDisplaySeconds",
                    "maxSlateDisplaySeconds",
                    "state",
                ],
            )?,
        ),
        ("ListStreamKeys", stream_keys),
    ];
    policy::finish(
        "ivs",
        approved,
        DEFERRED,
        "Channel/configuration/key-pair summaries are metadata. Unfiltered ad and failover-slate summaries enumerate account-owned regional metadata; nested ad configuration contains only a MediaTailor ARN and post-roll enabled/duration, and failover slate contains state/threshold/durations without media content. Channel details include SRT passphrase and stream-key reads return ingest credentials; these calls are denied in full. Session content and remaining families are not approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "GetAdConfiguration",
    "GetFailoverSlateConfiguration",
    "GetPlaybackKeyPair",
    "GetPlaybackRestrictionPolicy",
    "GetRecordingConfiguration",
    "GetStream",
    "ListStreamSessions",
    "ListStreams",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "BatchStartViewerSessionRevocation",
    "CreateAdConfiguration",
    "CreateChannel",
    "CreateFailoverSlateConfiguration",
    "CreatePlaybackRestrictionPolicy",
    "CreateRecordingConfiguration",
    "CreateStreamKey",
    "DeleteAdConfiguration",
    "DeleteChannel",
    "DeleteFailoverSlateConfiguration",
    "DeletePlaybackKeyPair",
    "DeletePlaybackRestrictionPolicy",
    "DeleteRecordingConfiguration",
    "DeleteStreamKey",
    "ImportPlaybackKeyPair",
    "InsertAdBreak",
    "PutMetadata",
    "StartViewerSessionRevocation",
    "StopStream",
    "TagResource",
    "UntagResource",
    "UpdateAdConfiguration",
    "UpdateChannel",
    "UpdateFailoverSlateConfiguration",
    "UpdatePlaybackRestrictionPolicy",
];

const SECRET: &[&str] = &[
    "BatchGetChannel",
    "BatchGetStreamKey",
    "GetChannel",
    "GetStreamKey",
    "GetStreamSession",
];
