use super::policy;
use crate::overlay::{Identity, IdentityField, MemberPath, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListStreams",
            policy::query(
                policy::enumerate(
                    "StreamInfoList",
                    policy::arn("StreamARN")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["StreamARN"],
            )?,
        ),
        (
            "ListSignalingChannels",
            policy::query(
                policy::enumerate(
                    "ChannelInfoList",
                    policy::arn("ChannelARN")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["ChannelARN"],
            )?,
        ),
        (
            "DescribeMappedResourceConfiguration",
            policy::bind(
                policy::query(
                    policy::enumerate(
                        "MappedResourceConfigurationList",
                        mapped_identity()?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    &["ARN", "Type"],
                )?,
                "StreamARN",
                "ListStreams",
                "StreamARN",
            )?,
        ),
    ];
    policy::finish(
        "kinesis-video",
        approved,
        DEFERRED,
        "Stream and signaling-channel metadata are approved with NextToken. Media/edge configuration, endpoint/capability discovery and tags need separate scoped review; video contents are not collected.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

fn mapped_identity() -> Result<Identity, NameError> {
    Ok(Identity::CorrelatedKey(vec![
        IdentityField::Request(MemberPath::new("StreamARN")?),
        IdentityField::Selected(MemberPath::new("Type")?),
        IdentityField::Selected(MemberPath::new("ARN")?),
    ]))
}

const DEFERRED: &[&str] = &[
    "DescribeEdgeConfiguration",
    "DescribeImageGenerationConfiguration",
    "DescribeMediaStorageConfiguration",
    "DescribeNotificationConfiguration",
    "DescribeSignalingChannel",
    "DescribeStream",
    "DescribeStreamStorageConfiguration",
    "GetDataEndpoint",
    "GetSignalingChannelEndpoint",
    "ListEdgeAgentConfigurations",
    "ListTagsForResource",
    "ListTagsForStream",
];

const MUTATING: &[&str] = &[
    "CreateSignalingChannel",
    "CreateStream",
    "DeleteEdgeConfiguration",
    "DeleteSignalingChannel",
    "DeleteStream",
    "StartEdgeConfigurationUpdate",
    "TagResource",
    "TagStream",
    "UntagResource",
    "UntagStream",
    "UpdateDataRetention",
    "UpdateImageGenerationConfiguration",
    "UpdateMediaStorageConfiguration",
    "UpdateNotificationConfiguration",
    "UpdateSignalingChannel",
    "UpdateStream",
    "UpdateStreamStorageConfiguration",
];
