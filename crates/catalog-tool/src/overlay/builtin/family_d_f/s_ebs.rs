// Pinned model: .local/aws-models/models/ebs/service/2019-11-02/ebs-2019-11-02.json
use super::support::{reviewed};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ebs",
        vec![

        ],
        &[],
        &[
            // Returns the data in a block in an Amazon Elastic Block Store snapshot.
            ("GetSnapshotBlock", DenyReason::Content),
            // Returns information about the blocks in an Amazon Elastic Block Store snapshot.
            ("ListSnapshotBlocks", DenyReason::Content),
            // Returns information about the blocks that are different between two Amazon Elastic Block Store snapshots of the same volume/snapshot lineage.
            ("ListChangedBlocks", DenyReason::Content),
            // Creates a new Amazon EBS snapshot.
            ("StartSnapshot", DenyReason::Mutating),
            // Writes a block of data to a snapshot.
            ("PutSnapshotBlock", DenyReason::Mutating),
            // Seals and completes the snapshot after all of the required blocks of data have been written to it.
            ("CompleteSnapshot", DenyReason::Mutating),
        ],
    )
}
