use super::support::{arn, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elementalinference",
        vec![
            queried(
                inventory(
                    "ListDictionaries",
                    "dictionaries",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "id", "name", "status"],
            )?,
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
        &[("GetDictionary", DenyReason::Content)],
    )
}
