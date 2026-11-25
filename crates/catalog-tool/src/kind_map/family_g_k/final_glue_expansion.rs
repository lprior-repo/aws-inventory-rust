use crate::kind_map::{CfnMapping, OperationMapping};

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
