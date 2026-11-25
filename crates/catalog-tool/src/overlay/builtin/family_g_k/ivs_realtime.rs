use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListStages",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "stages",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListEncoderConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "encoderConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListStorageConfigurations",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "storageConfigurations",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListPublicKeys",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "publicKeys",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn", "name"],
            )?,
        ),
        (
            "ListIngestConfigurations",
            policy::query(
                policy::enumerate(
                    "ingestConfigurations",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn", "name", "stageArn"],
            )?,
        ),
        (
            "ListCompositions",
            policy::query(
                policy::tags(
                    policy::enumerate(
                        "compositions",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "tags",
                )?,
                &["arn", "stageArn"],
            )?,
        ),
    ];
    policy::finish(
        "ivs-realtime",
        approved,
        DEFERRED,
        "Stage, encoder, storage, public-key, ingest and composition summaries are metadata. Composition destinations contain only lifecycle identifiers/times. Ingest configuration detail returns stream keys and is denied. Participant/session/token paths remain unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "GetComposition",
    "GetEncoderConfiguration",
    "GetParticipant",
    "GetPublicKey",
    "GetStage",
    "GetStageSession",
    "GetStorageConfiguration",
    "ListParticipantEvents",
    "ListParticipantReplicas",
    "ListParticipants",
    "ListStageSessions",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateEncoderConfiguration",
    "CreateIngestConfiguration",
    "CreateParticipantToken",
    "CreateStage",
    "CreateStorageConfiguration",
    "DeleteEncoderConfiguration",
    "DeleteIngestConfiguration",
    "DeletePublicKey",
    "DeleteStage",
    "DeleteStorageConfiguration",
    "DisconnectParticipant",
    "ImportPublicKey",
    "RevokeConnectionToken",
    "StartComposition",
    "StartParticipantReplication",
    "StopComposition",
    "StopParticipantReplication",
    "TagResource",
    "UntagResource",
    "UpdateIngestConfiguration",
    "UpdateStage",
];

const SECRET: &[&str] = &["GetIngestConfiguration"];
