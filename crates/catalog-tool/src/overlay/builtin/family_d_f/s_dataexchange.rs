use super::support::{arn, bound, fixed_value, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{FixedValue, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dataexchange",
        vec![
            queried(
                inventory(
                    "ListDataGrants",
                    "DataGrantSummaries",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "Arn",
                    "Name",
                    "AcceptanceState",
                    "DataSetId",
                    "SourceDataSetId",
                    "Id",
                ],
            )?,
            fixed_value(
                queried(
                    inventory(
                        "ListDataSets",
                        "DataSets",
                        arn("Arn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &["Arn", "AssetType", "Id", "Name", "SourceId"],
                )?,
                "Origin",
                FixedValue::String("OWNED".to_owned()),
            )?,
            queried(
                inventory(
                    "ListReceivedDataGrants",
                    "DataGrantSummaries",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name", "AcceptanceState", "DataSetId", "Id"],
            )?,
            queried(
                inventory(
                    "ListEventActions",
                    "EventActions",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Id"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListDataSetRevisions",
                        "Revisions",
                        arn("Arn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &[("DataSetId", "ListDataSets", "Id")],
                )?,
                &["Arn", "DataSetId", "Id"],
            )?,
        ],
        &[super::s_dataexchange_decisions_0::ENTRIES],
        &[
            ("ListJobs", DenyReason::Secret),
            ("ListRevisionAssets", DenyReason::Secret),
        ],
    )
}
