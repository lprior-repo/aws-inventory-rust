use crate::kind_map::{CfnMapping, MappingSource, OperationMapping};

pub(super) const SOURCE: MappingSource<'static> = MappingSource {
    cfn: &[CfnMapping::canonical("AWS::Lambda::Alias")],
    operations: &[
        OperationMapping {
            service: "lambda",
            operation: "ListFunctions",
            kind: "AWS::Lambda::Function",
        },
        OperationMapping {
            service: "lambda",
            operation: "GetFunctionConfiguration",
            kind: "AWS::Lambda::Function",
        },
        OperationMapping {
            service: "lambda",
            operation: "ListAliases",
            kind: "AWS::Lambda::Alias",
        },
    ],
    ..super::super::EMPTY
};
