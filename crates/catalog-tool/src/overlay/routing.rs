use super::{FixedValue, MemberPath, Scope};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RequestRouting {
    #[default]
    Regional,
    FixedRegion { region: String },
    Discovered { operation: String, endpoints_path: MemberPath, address_path: MemberPath, cache_minutes_path: MemberPath, trusted_dns_suffix: String },
    S3Outposts,
    S3ExpressControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionBranch {
    pub discriminator: String,
    pub fixed: BTreeMap<MemberPath, FixedValue>,
    pub bind: BTreeMap<MemberPath, super::Binding>,
    pub emission: Option<super::Emission>,
    pub row_policy: Option<super::RowPolicy>,
    pub scope: Scope,
    pub request_region: Option<String>,
}
