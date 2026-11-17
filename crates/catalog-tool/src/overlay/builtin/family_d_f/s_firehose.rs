// Pinned model: .local/aws-models/models/firehose/service/2015-08-04/firehose-2015-08-04.json
use super::support::{inventory, queried, reviewed};
use crate::overlay::{Identity, NameError, Paginator, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "firehose",
        vec![
            // Lists your Firehose streams in alphabetical order of their names.
            // Name-based pagination using the last returned name as the exclusive start cursor.
            queried(inventory(
                "ListDeliveryStreams",
                "DeliveryStreamNames",
                Identity::Template("arn:{partition}:firehose:{region}:{account}:deliverystream/{$}".to_owned()),
                Paginator::LastListItem {
                    input: crate::overlay::MemberPath::new("ExclusiveStartDeliveryStreamName")?,
                    output: crate::overlay::MemberPath::new("DeliveryStreamNames")?,
                    truncated: crate::overlay::MemberPath::new("HasMoreDeliveryStreams")?,
                },
            )?, &["$"])?,
        ],
        &[super::s_firehose_decisions_0::ENTRIES],
        &[
            // The whole response can contain SplunkDestinationDescription.HECToken.
            // Selecting names/ARNs cannot prevent receiving destination credentials.
            ("DescribeDeliveryStream", DenyReason::Secret),
            // Writes a single data record into an Firehose stream.
            ("PutRecord", DenyReason::Mutating),
            // Writes multiple data records into a Firehose stream in a single call, which can achieve higher throughput per producer than when writing single records.
            ("PutRecordBatch", DenyReason::Mutating),
        ],
    )
}
