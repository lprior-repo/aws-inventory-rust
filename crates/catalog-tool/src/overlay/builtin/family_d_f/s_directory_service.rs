use super::support::{inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "directory-service",
        vec![
            queried(
                inventory(
                    "DescribeSnapshots",
                    "Snapshots",
                    key(&["SnapshotId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["SnapshotId", "DirectoryId", "Type", "Name", "Status"],
            )?,
            queried(
                inventory(
                    "DescribeTrusts",
                    "Trusts",
                    key(&["TrustId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "TrustId",
                    "DirectoryId",
                    "RemoteDomainName",
                    "TrustType",
                    "TrustState",
                    "StateLastUpdatedDateTime",
                    "TrustStateReason",
                ],
            )?,
            queried(
                inventory(
                    "ListADAssessments",
                    "Assessments",
                    key(&["AssessmentId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "AssessmentId",
                    "DirectoryId",
                    "DnsName",
                    "Status",
                    "ReportType",
                ],
            )?,
            queried(
                inventory(
                    "ListLogSubscriptions",
                    "LogSubscriptions",
                    key(&["DirectoryId", "LogGroupName"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["DirectoryId", "LogGroupName"],
            )?,
            queried(
                inventory(
                    "DescribeEventTopics",
                    "EventTopics",
                    key(&["DirectoryId", "TopicName"])?,
                    Paginator::None,
                )?,
                &["DirectoryId", "TopicName", "TopicArn", "Status"],
            )?,
        ],
        &[
            super::s_directory_service_decisions_0::ENTRIES,
            super::s_directory_service_decisions_1::ENTRIES,
            super::s_directory_service_decisions_2::ENTRIES,
        ],
        &[("DescribeDirectories", DenyReason::Secret)],
    )
}
