// Pinned operation purposes: .local/aws-models/models/rolesanywhere/service/2018-05-10/rolesanywhere-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a profile , a list of the roles that Roles Anywhere service is trusted to assume. You use profiles to intersect permissions with IAM managed policies. Required permissions: rolesanywhere:CreateProfile .
    ("CreateProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a trust anchor to establish trust between IAM Roles Anywhere and your certificate authority (CA). You can define a trust anchor as a reference to an Private Certificate Authority (Private CA) or by uploading a CA certificate. Your Amazon Web Services workloads can authenticate with the trust anchor using certificates issued by the CA in exchange for temporary Amazon Web Services credential
    ("CreateTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete an entry from the attribute mapping rules enforced by a given profile.
    (
        "DeleteAttributeMapping",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a certificate revocation list (CRL). Required permissions: rolesanywhere:DeleteCrl .
    ("DeleteCrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a profile. Required permissions: rolesanywhere:DeleteProfile .
    ("DeleteProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a trust anchor. Required permissions: rolesanywhere:DeleteTrustAnchor .
    ("DeleteTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disables a certificate revocation list (CRL). Required permissions: rolesanywhere:DisableCrl .
    ("DisableCrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disables a profile. When disabled, temporary credential requests with this profile fail. Required permissions: rolesanywhere:DisableProfile .
    ("DisableProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Disables a trust anchor. When disabled, temporary credential requests specifying this trust anchor are unauthorized. Required permissions: rolesanywhere:DisableTrustAnchor .
    ("DisableTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables a certificate revocation list (CRL). When enabled, certificates stored in the CRL are unauthorized to receive session credentials. Required permissions: rolesanywhere:EnableCrl .
    ("EnableCrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables temporary credential requests for a profile. Required permissions: rolesanywhere:EnableProfile .
    ("EnableProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables a trust anchor. When enabled, certificates in the trust anchor chain are authorized for trust validation. Required permissions: rolesanywhere:EnableTrustAnchor .
    ("EnableTrustAnchor", Decision::Denied(DenyReason::Mutating)),
    (
        "GetCrl",
        Decision::Deferred(
            "Required selectors crlId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members crl. Pinned purpose: Gets a certificate revocation list (CRL). Required permissions: rolesanywhere:GetCrl .",
        ),
    ),
    (
        "GetProfile",
        Decision::Deferred(
            "Required selectors profileId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members profile. Pinned purpose: Gets a profile. Required permissions: rolesanywhere:GetProfile .",
        ),
    ),
    (
        "GetSubject",
        Decision::Deferred(
            "Required selectors subjectId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members subject. Pinned purpose: Gets a subject , which associates a certificate identity with authentication attempts. The subject stores auditing information such as the status of the last authentication attempt, the certificate data used in the attempt, and the last time the asso",
        ),
    ),
    (
        "GetTrustAnchor",
        Decision::Deferred(
            "Required selectors trustAnchorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members trustAnchor. Pinned purpose: Gets a trust anchor. Required permissions: rolesanywhere:GetTrustAnchor .",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Imports the certificate revocation list (CRL). A CRL is a list of certificates that have been revoked by the issuing certificate Authority (CA).In order to be properly imported, a CRL must be in PEM format. IAM Roles Anywhere validates against the CRL before issuing credentials. Required permissions: rolesanywhere:ImportCrl .
    ("ImportCrl", Decision::Denied(DenyReason::Mutating)),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags attached to the resource. Required permissions: rolesanywhere:ListTagsForResource .",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Put an entry in the attribute mapping rules that will be enforced by a given profile. A mapping specifies a certificate field and one or more specifiers that have contextual meanings.
    (
        "PutAttributeMapping",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Attaches a list of notification settings to a trust anchor. A notification setting includes information such as event name, threshold, status of the notification setting, and the channel to notify. Required permissions: rolesanywhere:PutNotificationSettings .
    (
        "PutNotificationSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Resets the custom notification setting to IAM Roles Anywhere default setting. Required permissions: rolesanywhere:ResetNotificationSettings .
    (
        "ResetNotificationSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Attaches tags to a resource. Required permissions: rolesanywhere:TagResource .
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from the resource. Required permissions: rolesanywhere:UntagResource .
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the certificate revocation list (CRL). A CRL is a list of certificates that have been revoked by the issuing certificate authority (CA). IAM Roles Anywhere validates against the CRL before issuing credentials. Required permissions: rolesanywhere:UpdateCrl .
    ("UpdateCrl", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a profile , a list of the roles that IAM Roles Anywhere service is trusted to assume. You use profiles to intersect permissions with IAM managed policies. Required permissions: rolesanywhere:UpdateProfile .
    ("UpdateProfile", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a trust anchor. You establish trust between IAM Roles Anywhere and your certificate authority (CA) by configuring a trust anchor. You can define a trust anchor as a reference to an Private Certificate Authority (Private CA) or by uploading a CA certificate. Your Amazon Web Services workloads can authenticate with the trust anchor using certificates issued by the CA in exchange for temporar
    ("UpdateTrustAnchor", Decision::Denied(DenyReason::Mutating)),
];
