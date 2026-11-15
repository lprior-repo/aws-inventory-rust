use super::{defer, deny};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("timestream-write", vec![
        ("CreateBatchLoadTask", deny(DenyReason::Mutating)),
        ("CreateDatabase", deny(DenyReason::Mutating)),
        ("CreateTable", deny(DenyReason::Mutating)),
        ("DeleteDatabase", deny(DenyReason::Mutating)),
        ("DeleteTable", deny(DenyReason::Mutating)),
        ("DescribeBatchLoadTask", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("DescribeDatabase", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("DescribeEndpoints", deny(DenyReason::ReferenceData)),
        ("DescribeTable", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("ListBatchLoadTasks", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("ListDatabases", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("ListTables", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("ListTagsForResource", defer("This service requires discovered cell endpoints; the current endpoint transport cannot execute required endpoint discovery.")),
        ("ResumeBatchLoadTask", deny(DenyReason::Mutating)),
        ("TagResource", deny(DenyReason::Mutating)),
        ("UntagResource", deny(DenyReason::Mutating)),
        ("UpdateDatabase", deny(DenyReason::Mutating)),
        ("UpdateTable", deny(DenyReason::Mutating)),
        ("WriteRecords", deny(DenyReason::Mutating)),
    ])
}
