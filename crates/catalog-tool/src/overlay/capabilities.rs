use super::MemberPath;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceProjection {
    IamTrustSummary(MemberPath),
    SensitiveDigest(MemberPath),
    SensitiveMapDigest { path: MemberPath, key: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindClassification {
    pub field: MemberPath,
    pub variants: BTreeMap<String, String>,
}
