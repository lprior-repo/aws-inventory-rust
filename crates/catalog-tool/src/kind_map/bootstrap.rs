use super::BootstrapPurpose;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BootstrapEntry {
    pub(super) service: String,
    pub(super) operation: String,
    pub(super) purpose: BootstrapPurpose,
}

