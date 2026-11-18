// Pinned model: .local/aws-models/models/elementalinference/service/2018-11-14/elementalinference-2018-11-14.json
use super::support::{arn, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elementalinference",
        vec![
            // Lists the dictionaries in your account.
            queried(
                inventory(
                    "ListDictionaries",
                    "dictionaries",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "id", "name", "status"],
            )?,
            // Displays a list of feeds that belong to this AWS account.
            queried(
                inventory(
                    "ListFeeds",
                    "feeds",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "id", "name", "status"],
            )?,
        ],
        &[super::s_elementalinference_decisions_0::ENTRIES],
        &[
            // Retrieves information about the specified dictionary.
            ("GetDictionary", DenyReason::Content),
        ],
    )
}
