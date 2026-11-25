// Pinned model: .local/aws-models/models/dataexchange/service/2017-07-25/dataexchange-2017-07-25.json
use super::support::{arn, bound, fixed_value, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{FixedValue, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dataexchange",
        vec![
            // This operation returns information about all data grants.
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
            // This operation lists your data sets.
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
            // This operation returns information about all received data grants.
            queried(
                inventory(
                    "ListReceivedDataGrants",
                    "DataGrantSummaries",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Name", "AcceptanceState", "DataSetId", "Id"],
            )?,
            // Event actions contain metadata for revision publication automation, not exported asset content.
            queried(
                inventory(
                    "ListEventActions",
                    "EventActions",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn", "Id"],
            )?,
            // Revision summaries contain metadata only; asset enumeration separately returns download capability URLs.
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
            // Jobs[].Details includes ExportAssetToSignedUrl.SignedUrl and ImportAssetFromSignedUrl.SignedUrl.
            ("ListJobs", DenyReason::Secret),
            // Assets[].AssetDetails.ApiGatewayApiAsset.ApiSpecificationDownloadUrl grants access to API specification content.
            ("ListRevisionAssets", DenyReason::Secret),
        ],
    )
}
