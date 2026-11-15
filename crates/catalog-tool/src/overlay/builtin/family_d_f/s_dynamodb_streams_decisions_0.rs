// Explicit dispositions from .local/aws-models/models/dynamodb-streams/service/2012-08-10/dynamodb-streams-2012-08-10.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("DescribeStream", "Requires StreamArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about a stream, including the current status of the stream, its Amazon Resource Name (ARN), the composition of its shards, and its corresponding DynamoDB table."),
];
