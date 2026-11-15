// Pinned model: .local/aws-models/models/firehose/service/2015-08-04/firehose-2015-08-04.json
use super::support::{reviewed};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "firehose",
        vec![

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
