use super::Shape;

const TREE_0: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("profileName", Shape::Text),
    ("profileVersion", Shape::Text),
    ("profileVersionArn", Shape::Text),
    ("signingMaterial", Shape::Map),
    ("signingMaterial.certificateArn", Shape::Text),
    ("signatureValidityPeriod", Shape::Map),
    ("signatureValidityPeriod.value", Shape::Integer),
    ("signatureValidityPeriod.type", Shape::Text),
    ("platformId", Shape::Text),
    ("platformDisplayName", Shape::Text),
    ("signingParameters", Shape::Map),
    ("signingParameters.*", Shape::Text),
    ("status", Shape::Text),
    ("arn", Shape::Text),
    ("tags", Shape::Map),
    ("tags.*", Shape::Text),
];

const TREE_1: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("CodeSigningConfigId", Shape::Text),
    ("CodeSigningConfigArn", Shape::Text),
    ("Description", Shape::Text),
    ("AllowedPublishers", Shape::Map),
    ("AllowedPublishers.SigningProfileVersionArns", Shape::List),
    ("AllowedPublishers.SigningProfileVersionArns.*", Shape::Text),
    ("CodeSigningPolicies", Shape::Map),
    (
        "CodeSigningPolicies.UntrustedArtifactOnDeployment",
        Shape::Text,
    ),
    ("LastModified", Shape::Text),
];

const TREE_2: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("Path", Shape::Text),
    ("RoleName", Shape::Text),
    ("RoleId", Shape::Text),
    ("Arn", Shape::Text),
    ("CreateDate", Shape::Timestamp),
    ("AssumeRolePolicyDocument", Shape::Map),
    (
        "AssumeRolePolicyDocument.trust_policy_values_omitted",
        Shape::Bool,
    ),
    ("AssumeRolePolicyDocument.trust_statements", Shape::List),
    ("AssumeRolePolicyDocument.trust_statements.*", Shape::Map),
    (
        "AssumeRolePolicyDocument.trust_statements.*.effect",
        Shape::Text,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.actions",
        Shape::List,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.actions.*",
        Shape::Text,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.principals",
        Shape::Map,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.principals.Service",
        Shape::List,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.principals.Service.*",
        Shape::Text,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.has_not_principal",
        Shape::Bool,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.has_not_action",
        Shape::Bool,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.condition_operators",
        Shape::List,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.condition_operators.*",
        Shape::Text,
    ),
    (
        "AssumeRolePolicyDocument.trust_statements.*.condition_values_omitted",
        Shape::Bool,
    ),
    ("Description", Shape::Text),
    ("MaxSessionDuration", Shape::Integer),
    ("PermissionsBoundary", Shape::Map),
    ("PermissionsBoundary.PermissionsBoundaryType", Shape::Text),
    ("PermissionsBoundary.PermissionsBoundaryArn", Shape::Text),
    ("Tags", Shape::List),
    ("Tags.*", Shape::Map),
    ("Tags.*.Key", Shape::Text),
    ("Tags.*.Value", Shape::Text),
    ("RoleLastUsed", Shape::Map),
    ("RoleLastUsed.LastUsedDate", Shape::Timestamp),
    ("RoleLastUsed.Region", Shape::Text),
    ("SourceRoleTemplate", Shape::Map),
    ("SourceRoleTemplate.TemplateArn", Shape::Text),
    ("SourceRoleTemplate.TemplateMinorVersion", Shape::Integer),
];

const TREE_3: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("Path", Shape::Text),
    ("GroupName", Shape::Text),
    ("GroupId", Shape::Text),
    ("Arn", Shape::Text),
    ("CreateDate", Shape::Timestamp),
];

const TREE_4: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("activityArn", Shape::Text),
    ("name", Shape::Text),
    ("creationDate", Shape::Timestamp),
    ("encryptionConfiguration", Shape::Map),
    ("encryptionConfiguration.type", Shape::Text),
];

const TREE_5: &[(&str, Shape)] = &[
    ("", Shape::Map),
    ("stateMachineArn", Shape::Text),
    ("name", Shape::Text),
    ("type", Shape::Text),
    ("creationDate", Shape::Timestamp),
];

pub(super) fn tree(kind: &str) -> Option<&'static [(&'static str, Shape)]> {
    match kind {
        "AWS::Signer::SigningProfile" => Some(TREE_0),
        "AWS::Lambda::CodeSigningConfig" => Some(TREE_1),
        "AWS::IAM::Role" => Some(TREE_2),
        "AWS::IAM::Group" => Some(TREE_3),
        "AWS::StepFunctions::Activity" => Some(TREE_4),
        "AWS::StepFunctions::StateMachine" => Some(TREE_5),
        _ => None,
    }
}
