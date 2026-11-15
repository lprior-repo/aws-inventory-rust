use crate::kind_map::{ApiOnlyMapping, CfnMapping, OperationMapping};

pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::IVS::AdConfiguration"),
];

pub(super) const API_ONLY: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::IVSFailoverSlateConfiguration",
        reason: "The pinned 1,881-type CFN registry has no AWS::IVS::FailoverSlateConfiguration; the IVS account-owned regional summary has its own ARN and lifecycle/threshold/duration metadata, with no slate media or content URL.",
    },
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping { service: "ivs", operation: "ListAdConfigurations", kind: "AWS::IVS::AdConfiguration" },
    OperationMapping { service: "ivs", operation: "ListFailoverSlateConfigurations", kind: "AWS::ApiOnly::IVSFailoverSlateConfiguration" },
];
