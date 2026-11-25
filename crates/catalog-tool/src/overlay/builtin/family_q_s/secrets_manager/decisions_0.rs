use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateCrossAccountReplica",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("BatchGetSecretValue", Decision::Denied(DenyReason::Secret)),
    ("CancelRotateSecret", Decision::Denied(DenyReason::Mutating)),
    ("CreateSecret", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSecret", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeSecret",
        Decision::Deferred(
            "Required selectors SecretId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ARN, Name, Type, Description, KmsKeyId, RotationEnabled, RotationLambdaARN, RotationRules, ExternalSecretRotationMetadata, ExternalSecretRotationRoleArn, LastRotatedDate, LastChangedDate, LastAccessedDate, DeletedDate, NextRotationDate, Tags, VersionIdsToStages, OwningService, CreatedDate, PrimaryRegion, ReplicationStatus, CrossAccountPrimary, CrossAccountReplicas. Pinned purpose: Retrieves the details of a secret. It does not include the encrypted secret value. Secrets Manager only returns fields that have a value in the response. The VersionIdsToStages field in the response includes every version ID that has a staging label ",
        ),
    ),
    (
        "DisassociateCrossAccountReplica",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("GetRandomPassword", Decision::Denied(DenyReason::Content)),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "Required selectors SecretId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ARN, Name, ResourcePolicy, PolicyName. Pinned purpose: Retrieves the JSON text of the resource-based policy document attached to the secret. For more information about permissions policies attached to a secret, see Permissions policies attached to a secret . Secrets Manager generates a CloudTrail log ent",
        ),
    ),
    ("GetSecretValue", Decision::Denied(DenyReason::Secret)),
    (
        "ListResourcePolicies",
        Decision::Deferred(
            "Required selectors SecretId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourcePolicies, NextToken. Pinned purpose: Lists the resource-based policies attached to a secret. The response contains only the names of the policies, not their contents. Secrets Manager lists default first, followed by any other policies in alphabetical order. To get the contents of a poli",
        ),
    ),
    (
        "ListSecretVersionIds",
        Decision::Deferred(
            "Required selectors SecretId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Versions, NextToken, ARN, Name. Pinned purpose: Lists the versions of a secret. Secrets Manager uses staging labels to indicate the different versions of a secret. For more information, see Secrets Manager concepts: Versions . To list the secrets in the account, use ListSecrets . ListSecretVersion",
        ),
    ),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    ("PutSecretValue", Decision::Denied(DenyReason::Mutating)),
    (
        "RemoveRegionsFromReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ReplicateSecretToRegions",
        Decision::Deferred(
            "Required selectors SecretId, AddReplicaRegions are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ARN, ReplicationStatus. Pinned purpose: Replicates the secret to a new Regions. See Multi-Region secrets . Secrets Manager generates a CloudTrail log entry when you call this action. Do not include sensitive information in request parameters because it might be logged. For more information",
        ),
    ),
    ("RestoreSecret", Decision::Denied(DenyReason::Mutating)),
    ("RotateSecret", Decision::Denied(DenyReason::Mutating)),
    (
        "StopReplicationToReplica",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSecret", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateSecretVersionStage",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ValidateResourcePolicy",
        Decision::Deferred(
            "Required selectors ResourcePolicy are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PolicyValidationPassed, ValidationErrors. Pinned purpose: Validates that a resource policy does not grant a wide range of principals access to your secret. A resource-based policy is optional for secrets. The API performs three checks when validating the policy: Sends a call to Zelkova , an automated reason",
        ),
    ),
];
