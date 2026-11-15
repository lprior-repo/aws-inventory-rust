use super::MemberPath;
use std::collections::BTreeMap;

/// Reviewed transformation of otherwise safe resource metadata before persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceProjection {
    /// Publish bounded IAM trust summaries, never the document or condition values.
    IamTrustSummary(MemberPath),
    /// Stable digest of one reviewed sensitive metadata text field.
    SensitiveDigest(MemberPath),
    /// Stable digest of one explicitly reviewed entry in a modeled string map.
    SensitiveMapDigest { path: MemberPath, key: String },
}

/// Exact, fail-closed discrimination of one modeled response into canonical kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindClassification {
    pub field: MemberPath,
    pub variants: BTreeMap<String, String>,
}
