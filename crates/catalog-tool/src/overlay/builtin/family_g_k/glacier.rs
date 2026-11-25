use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListVaults",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "VaultList",
                        policy::arn("VaultARN")?,
                        Some(("marker", "Marker")),
                    )?,
                    "accountId",
                    "-",
                )?,
                &["VaultARN"],
            )?,
        ),
        (
            "ListProvisionedCapacity",
            policy::query(
                policy::fixed(
                    policy::enumerate("ProvisionedCapacityList", policy::key("CapacityId")?, None)?,
                    "accountId",
                    "-",
                )?,
                &["CapacityId", "StartDate", "ExpirationDate"],
            )?,
        ),
    ];
    policy::finish(
        "glacier",
        approved,
        DEFERRED,
        "Vaults and existing provisioned-capacity units use the documented own-account dash. Capacity metadata has only its ID and dates, with no pagination. Purchases, archive retrieval and mutations remain denied; jobs and per-vault configuration need separate review.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "DescribeJob",
    "DescribeVault",
    "GetDataRetrievalPolicy",
    "GetVaultAccessPolicy",
    "GetVaultLock",
    "GetVaultNotifications",
    "ListJobs",
    "ListMultipartUploads",
    "ListParts",
    "ListTagsForVault",
];

const MUTATING: &[&str] = &[
    "AbortMultipartUpload",
    "AbortVaultLock",
    "AddTagsToVault",
    "CompleteMultipartUpload",
    "CompleteVaultLock",
    "CreateVault",
    "DeleteArchive",
    "DeleteVault",
    "DeleteVaultAccessPolicy",
    "DeleteVaultNotifications",
    "InitiateJob",
    "InitiateMultipartUpload",
    "InitiateVaultLock",
    "PurchaseProvisionedCapacity",
    "RemoveTagsFromVault",
    "SetDataRetrievalPolicy",
    "SetVaultAccessPolicy",
    "SetVaultNotifications",
    "UploadArchive",
    "UploadMultipartPart",
];

const CONTENT: &[&str] = &["GetJobOutput"];
