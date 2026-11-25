use super::support::{arn, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dynamodb-streams",
        vec![queried(
            inventory(
                "ListStreams",
                "Streams",
                arn("StreamArn")?,
                token("ExclusiveStartStreamArn", "LastEvaluatedStreamArn")?,
            )?,
            &["StreamArn", "TableName"],
        )?],
        &[super::s_dynamodb_streams_decisions_0::ENTRIES],
        &[
            ("GetRecords", DenyReason::Content),
            ("GetShardIterator", DenyReason::Secret),
        ],
    )
}
