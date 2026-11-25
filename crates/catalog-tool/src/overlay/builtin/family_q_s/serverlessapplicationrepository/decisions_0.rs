use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateApplicationVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCloudFormationChangeSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCloudFormationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "GetApplication",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationId, Author, CreationTime, Description, HomePageUrl, IsVerifiedAuthor, Labels, LicenseUrl, Name, ReadmeUrl, SpdxLicenseId, VerifiedAuthorUrl, Version. Pinned purpose: Gets the specified application.",
        ),
    ),
    (
        "GetApplicationPolicy",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Statements. Pinned purpose: Retrieves the policy for the application.",
        ),
    ),
    (
        "GetCloudFormationTemplate",
        Decision::Deferred(
            "Required selectors ApplicationId, TemplateId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApplicationId, CreationTime, ExpirationTime, SemanticVersion, Status, TemplateId, TemplateUrl. Pinned purpose: Gets the specified AWS CloudFormation template.",
        ),
    ),
    (
        "ListApplicationDependencies",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Dependencies, NextToken. Pinned purpose: Retrieves the list of applications nested in the containing application.",
        ),
    ),
    (
        "ListApplicationVersions",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Versions. Pinned purpose: Lists versions for the specified application.",
        ),
    ),
    (
        "PutApplicationPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UnshareApplication",
        Decision::Deferred(
            "Required selectors ApplicationId, OrganizationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Unshares an application from an AWS Organization. This operation can be called only from the organization's master account.",
        ),
    ),
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
];
