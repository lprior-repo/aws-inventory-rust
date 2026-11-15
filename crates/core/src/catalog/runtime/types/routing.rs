use super::{BindingPlan, PlanScope};
use crate::record::FieldId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RequestRouting {
    #[default]
    Regional,
    FixedRegion { region: String },
    Discovered { operation: String, endpoints_path: Box<[FieldId]>, address_path: Box<[FieldId]>, cache_minutes_path: Box<[FieldId]>, trusted_dns_suffix: String },
    S3Outposts,
    S3ExpressControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionBranch {
    pub discriminator: String,
    pub bindings: Box<[BindingPlan]>,
    pub emission: Option<super::Emission>,
    pub row_policy: Option<super::RowPolicyPlan>,
    pub scope: PlanScope,
    pub request_region: Option<String>,
}
