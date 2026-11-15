// Pinned model: .local/aws-models/models/directory-service/service/2015-04-16/directory-service-2015-04-16.json
use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, Paginator, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "directory-service",
        vec![
        // Obtains information about the directory snapshots that belong to this account.
        queried(inventory("DescribeSnapshots", "Snapshots", key(&["SnapshotId"])?, token("NextToken", "NextToken")?)?, &["SnapshotId", "DirectoryId", "Type", "Name", "Status"])?,
        // Obtains information about the trust relationships for this account.
        queried(inventory("DescribeTrusts", "Trusts", key(&["TrustId"])?, token("NextToken", "NextToken")?)?, &["TrustId", "DirectoryId", "RemoteDomainName", "TrustType", "TrustState", "StateLastUpdatedDateTime", "TrustStateReason"])?,
        // Retrieves a list of directory assessments for the specified directory or all assessments in your account.
        queried(inventory("ListADAssessments", "Assessments", key(&["AssessmentId"])?, token("NextToken", "NextToken")?)?, &["AssessmentId", "DirectoryId", "DnsName", "Status", "ReportType"])?,
        // Lists the active log subscriptions for the Amazon Web Services account.
        queried(inventory("ListLogSubscriptions", "LogSubscriptions", key(&["DirectoryId", "LogGroupName"])?, token("NextToken", "NextToken")?)?, &["DirectoryId", "LogGroupName"])?,
        // With no DirectoryId or TopicName, the API documents all account associations; no pagination fields exist.
        queried(inventory("DescribeEventTopics", "EventTopics", key(&["DirectoryId", "TopicName"])?, Paginator::None)?, &["DirectoryId", "TopicName", "TopicArn", "Status"])?,
        ],
        &[super::s_directory_service_decisions_0::ENTRIES, super::s_directory_service_decisions_1::ENTRIES, super::s_directory_service_decisions_2::ENTRIES],
        &[
            // DirectoryDescriptions includes RadiusSettings.SharedSecret and OwnerDirectoryDescription.RadiusSettings.SharedSecret; pinned output docs do not establish omission.
            ("DescribeDirectories", DenyReason::Secret),
        ],
    )
}
