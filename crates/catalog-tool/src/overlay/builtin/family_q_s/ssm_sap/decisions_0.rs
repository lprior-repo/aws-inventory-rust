use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DeleteResourcePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeregisterApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetApplication",
        Decision::Deferred(
            "Requires one of ApplicationId, ApplicationArn or AppRegistryArn although no field is Smithy-required. ListApplications is the complete application root.",
        ),
    ),
    (
        "GetComponent",
        Decision::Deferred(
            "Required selectors ApplicationId, ComponentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Component, Tags. Pinned purpose: Gets the component of an application registered with AWS Systems Manager for SAP.",
        ),
    ),
    (
        "GetConfigurationCheckOperation",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationCheckOperation. Pinned purpose: Gets the details of a configuration check operation by specifying the operation ID.",
        ),
    ),
    (
        "GetDatabase",
        Decision::Deferred(
            "Requires one of DatabaseArn or ApplicationId+ComponentId+DatabaseId semantic alternatives. ListDatabases supplies summary inventory; no guessed empty request is executable.",
        ),
    ),
    (
        "GetOperation",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Operation. Pinned purpose: Gets the details of an operation by specifying the operation ID.",
        ),
    ),
    (
        "GetResourcePermission",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: Gets permissions associated with the target database.",
        ),
    ),
    (
        "ListConfigurationCheckDefinitions",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListConfigurationCheckOperations",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationCheckOperations, NextToken. Pinned purpose: Lists the configuration check operations performed by AWS Systems Manager for SAP.",
        ),
    ),
    (
        "ListOperationEvents",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationEvents, NextToken. Pinned purpose: Returns a list of operations events. Available parameters include OperationID , as well as optional parameters MaxResults , NextToken , and Filters .",
        ),
    ),
    (
        "ListOperations",
        Decision::Deferred(
            "Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Operations, NextToken. Pinned purpose: Lists the operations performed by AWS Systems Manager for SAP.",
        ),
    ),
    (
        "ListSubCheckResults",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SubCheckResults, NextToken. Pinned purpose: Lists the sub-check results of a specified configuration check operation.",
        ),
    ),
    (
        "ListSubCheckRuleResults",
        Decision::Deferred(
            "Required selectors SubCheckResultId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RuleResults, NextToken. Pinned purpose: Lists the rules of a specified sub-check belonging to a configuration check operation.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists all tags on an SAP HANA application and/or database registered with AWS Systems Manager for SAP.",
        ),
    ),
    (
        "PutResourcePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RegisterApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StartApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "StartApplicationRefresh",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StartConfigurationChecks",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("StopApplication", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateApplicationSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
];
