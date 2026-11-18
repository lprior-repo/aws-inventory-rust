//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kinesis-video/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListStreams", policy::query(policy::enumerate("StreamInfoList", policy::arn("StreamARN")?, Some(("NextToken", "NextToken")))?, &["StreamARN"])?),
        ("ListSignalingChannels", policy::query(policy::enumerate("ChannelInfoList", policy::arn("ChannelARN")?, Some(("NextToken", "NextToken")))?, &["ChannelARN"])?),
        ("DescribeMappedResourceConfiguration", policy::query(policy::enumerate("MappedResourceConfigurationList", policy::arn("ARN")?, Some(("NextToken", "NextToken")))?, &["ARN", "Type"])?),
    ];
    policy::finish(
        "kinesis-video", approved, DEFERRED,
        "Stream and signaling-channel metadata are approved with NextToken. Media/edge configuration, endpoint/capability discovery and tags need separate scoped review; video contents are not collected.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeEdgeConfiguration", "DescribeImageGenerationConfiguration",
    "DescribeMediaStorageConfiguration",
    "DescribeNotificationConfiguration", "DescribeSignalingChannel", "DescribeStream",
    "DescribeStreamStorageConfiguration", "GetDataEndpoint", "GetSignalingChannelEndpoint",
    "ListEdgeAgentConfigurations", "ListTagsForResource", "ListTagsForStream",
];

const MUTATING: &[&str] = &[
    "CreateSignalingChannel", "CreateStream", "DeleteEdgeConfiguration", "DeleteSignalingChannel",
    "DeleteStream", "StartEdgeConfigurationUpdate", "TagResource", "TagStream", "UntagResource",
    "UntagStream", "UpdateDataRetention", "UpdateImageGenerationConfiguration",
    "UpdateMediaStorageConfiguration", "UpdateNotificationConfiguration", "UpdateSignalingChannel",
    "UpdateStream", "UpdateStreamStorageConfiguration",
];
