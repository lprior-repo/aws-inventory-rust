//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kinesis/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListStreams", policy::query(policy::enumerate("StreamSummaries", policy::arn("StreamARN")?, Some(("NextToken", "NextToken")))?, &["StreamARN"])?),
        ("ListChannels", policy::query(policy::enumerate("ChannelSummaries", policy::arn("ChannelARN")?, Some(("NextToken", "NextToken")))?, &["ChannelARN"])?),
        ("ListStreamConsumers", policy::query(policy::bind(policy::enumerate("Consumers", policy::arn("ConsumerARN")?, Some(("NextToken", "NextToken")))?, "StreamARN", "ListStreams", "StreamARN")?, &["ConsumerARN"])?),
        ("DescribeStreamSummary", policy::query(policy::bind(policy::enumerate("StreamDescriptionSummary", policy::arn("StreamARN")?, None)?, "StreamARN", "ListStreams", "StreamARN")?, &["StreamARN"])?),
    ];
    policy::finish(
        "kinesis", approved, DEFERRED,
        "Stream/channel summaries, stream-bound consumers and stream-summary detail are metadata. Record retrieval/iterators are denied. Remaining shards, policies and tag traversal are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeAccountSettings", "DescribeChannel", "DescribeLimits", "DescribeStream",
    "DescribeStreamConsumer", "GetResourcePolicy", "ListShards", "ListTagsForResource", "ListTagsForStream",
];

const MUTATING: &[&str] = &[
    "AddTagsToStream", "CreateChannel", "CreateStream", "DecreaseStreamRetentionPeriod", "DeleteChannel",
    "DeleteResourcePolicy", "DeleteStream", "DeregisterStreamConsumer", "DisableEnhancedMonitoring",
    "EnableEnhancedMonitoring", "IncreaseStreamRetentionPeriod", "MergeShards", "PutRecord", "PutRecords",
    "PutResourcePolicy", "RegisterStreamConsumer", "RemoveTagsFromStream", "SplitShard",
    "StartStreamEncryption", "StopStreamEncryption", "SubscribeToShard", "TagResource", "UntagResource",
    "UpdateAccountSettings", "UpdateChannel", "UpdateMaxRecordSize", "UpdateShardCount", "UpdateStreamMode",
    "UpdateStreamRecordDistributionStrategy", "UpdateStreamWarmThroughput",
];

const CONTENT: &[&str] = &[
    "GetRecords", "GetShardIterator",
];
