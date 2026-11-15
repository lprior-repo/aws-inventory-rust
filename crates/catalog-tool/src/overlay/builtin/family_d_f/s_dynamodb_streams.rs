// Pinned model: .local/aws-models/models/dynamodb-streams/service/2012-08-10/dynamodb-streams-2012-08-10.json
use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dynamodb-streams",
        vec![
        // Returns an array of stream ARNs associated with the current account and endpoint.
        queried(inventory("ListStreams", "Streams", arn("StreamArn")?, token("ExclusiveStartStreamArn", "LastEvaluatedStreamArn")?)?, &["StreamArn", "TableName"])?,
        ],
        &[super::s_dynamodb_streams_decisions_0::ENTRIES],
        &[
            // Retrieves the stream records from a given shard.
            ("GetRecords", DenyReason::Content),
            // Returns a shard iterator.
            ("GetShardIterator", DenyReason::Secret),
        ],
    )
}
