use super::support::{inventory, queried, reviewed};
use crate::classes::DenyReason;
use crate::overlay::{Identity, NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "firehose",
        vec![queried(
            inventory(
                "ListDeliveryStreams",
                "DeliveryStreamNames",
                Identity::Template(
                    "arn:{partition}:firehose:{region}:{account}:deliverystream/{$}".to_owned(),
                ),
                Paginator::LastListItem {
                    input: crate::overlay::MemberPath::new("ExclusiveStartDeliveryStreamName")?,
                    output: crate::overlay::MemberPath::new("DeliveryStreamNames")?,
                    truncated: crate::overlay::MemberPath::new("HasMoreDeliveryStreams")?,
                },
            )?,
            &["$"],
        )?],
        &[super::s_firehose_decisions_0::ENTRIES],
        &[
            ("DescribeDeliveryStream", DenyReason::Secret),
            ("PutRecord", DenyReason::Mutating),
            ("PutRecordBatch", DenyReason::Mutating),
        ],
    )
}
