use crate::kind_map::{
    CfnMapping, OperationMapping, RelationMapping, RelationPath, RelationSource, RelationTarget,
};

pub(super) const CFN: &[CfnMapping<'static>] =
    &[CfnMapping::canonical("AWS::IAM::InstanceProfile")];
pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "iam",
        operation: "ListInstanceProfiles",
        kind: "AWS::IAM::InstanceProfile",
    },
    OperationMapping {
        service: "iam",
        operation: "GetInstanceProfile",
        kind: "AWS::IAM::InstanceProfile",
    },
    OperationMapping {
        service: "iam",
        operation: "ListInstanceProfilesForRole",
        kind: "AWS::IAM::InstanceProfile",
    },
];
pub(super) const RELATIONS: &[RelationMapping<'static>] = &[RelationMapping {
    source_kind: "AWS::IAM::InstanceProfile",
    source: RelationSource::Request(RelationPath::Fields("RoleName")),
    target_kind: "AWS::IAM::Role",
    target: RelationTarget::SelectedOutput(RelationPath::Fields("RoleName")),
    relation: "HAS_ROLE",
    scope: "account",
    guard: None,
}];
