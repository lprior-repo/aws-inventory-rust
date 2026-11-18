// Pinned operation purposes: .local/aws-models/models/serverlessapplicationrepository/service/2017-09-08/serverlessapplicationrepository-2017-09-08.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates an application, optionally including an AWS SAM file to create the first application version in the same call.
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an application version.
    (
        "CreateApplicationVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an AWS CloudFormation change set for the given application.
    (
        "CreateCloudFormationChangeSet",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an AWS CloudFormation template.
    (
        "CreateCloudFormationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified application.
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
    // Changes service state rather than reading inventory. Pinned purpose: Sets the permission policy for an application. For the list of actions supported for this operation, see Application Permissions .
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
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified application.
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
];
