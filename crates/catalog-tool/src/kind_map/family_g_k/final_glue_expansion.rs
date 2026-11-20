use crate::kind_map::{CfnMapping, OperationMapping};

// Glue Database and Integration are list-capable in the pinned cloudcontrol registry. GetDatabases
// and DescribeIntegrations enumerate them directly; the other Glue kinds keep their list operation.
// Global Accelerator ByoipCidr and Greengrass v1 BulkDeployment have no pinned CFN type, so they
// map to reviewed API-only kinds.
pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::Glue::Database"),
    CfnMapping::canonical("AWS::Glue::Integration"),
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "glue",
        operation: "GetDatabases",
        kind: "AWS::Glue::Database",
    },
    OperationMapping {
        service: "glue",
        operation: "DescribeIntegrations",
        kind: "AWS::Glue::Integration",
    },
    OperationMapping {
        service: "global-accelerator",
        operation: "ListByoipCidrs",
        kind: "AWS::ApiOnly::GlobalAcceleratorByoipCidr",
    },
    OperationMapping {
        service: "greengrass",
        operation: "ListBulkDeployments",
        kind: "AWS::ApiOnly::GreengrassBulkDeployment",
    },
];
