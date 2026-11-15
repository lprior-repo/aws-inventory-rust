// Pinned operation purposes: .local/aws-models/models/ssm-sap/service/2018-05-10/ssm-sap-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Removes permissions associated with the target database.
    ("DeleteResourcePermission", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deregister an SAP application with AWS Systems Manager for SAP. This action does not aﬀect the existing setup of your SAP workloads on Amazon EC2.
    ("DeregisterApplication", Decision::Denied(DenyReason::Mutating)),
    ("GetApplication", Decision::Deferred("Requires one of ApplicationId, ApplicationArn or AppRegistryArn although no field is Smithy-required. ListApplications is the complete application root.")),
    ("GetComponent", Decision::Deferred("Required selectors ApplicationId, ComponentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Component, Tags. Pinned purpose: Gets the component of an application registered with AWS Systems Manager for SAP.")),
    ("GetConfigurationCheckOperation", Decision::Deferred("Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationCheckOperation. Pinned purpose: Gets the details of a configuration check operation by specifying the operation ID.")),
    ("GetDatabase", Decision::Deferred("Requires one of DatabaseArn or ApplicationId+ComponentId+DatabaseId semantic alternatives. ListDatabases supplies summary inventory; no guessed empty request is executable.")),
    ("GetOperation", Decision::Deferred("Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Operation. Pinned purpose: Gets the details of an operation by specifying the operation ID.")),
    ("GetResourcePermission", Decision::Deferred("Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Policy. Pinned purpose: Gets permissions associated with the target database.")),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists all configuration check types supported by AWS Systems Manager for SAP.
    ("ListConfigurationCheckDefinitions", Decision::Denied(DenyReason::ReferenceData)),
    ("ListConfigurationCheckOperations", Decision::Deferred("Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ConfigurationCheckOperations, NextToken. Pinned purpose: Lists the configuration check operations performed by AWS Systems Manager for SAP.")),
    ("ListOperationEvents", Decision::Deferred("Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members OperationEvents, NextToken. Pinned purpose: Returns a list of operations events. Available parameters include OperationID , as well as optional parameters MaxResults , NextToken , and Filters .")),
    ("ListOperations", Decision::Deferred("Required selectors ApplicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Operations, NextToken. Pinned purpose: Lists the operations performed by AWS Systems Manager for SAP.")),
    ("ListSubCheckResults", Decision::Deferred("Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SubCheckResults, NextToken. Pinned purpose: Lists the sub-check results of a specified configuration check operation.")),
    ("ListSubCheckRuleResults", Decision::Deferred("Required selectors SubCheckResultId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members RuleResults, NextToken. Pinned purpose: Lists the rules of a specified sub-check belonging to a configuration check operation.")),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists all tags on an SAP HANA application and/or database registered with AWS Systems Manager for SAP.")),
    // Changes service state rather than reading inventory. Pinned purpose: Adds permissions to the target database.
    ("PutResourcePermission", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Register an SAP application with AWS Systems Manager for SAP. You must meet the following requirements before registering. The SAP application you want to register with AWS Systems Manager for SAP is running on Amazon EC2. AWS Systems Manager Agent must be setup on an Amazon EC2 instance along with the required IAM permissions. Amazon EC2 instance(s) must have access to the secrets created in AWS 
    ("RegisterApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Request is an operation which starts an application. Parameter ApplicationId is required.
    ("StartApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Refreshes a registered application.
    ("StartApplicationRefresh", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates configuration check operations against a specified application.
    ("StartConfigurationChecks", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Request is an operation to stop an application. Parameter ApplicationId is required. Parameters StopConnectedEntity and IncludeEc2InstanceShutdown are optional.
    ("StopApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates tag for a resource by specifying the ARN.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Delete the tags for a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the settings of an application registered with AWS Systems Manager for SAP.
    ("UpdateApplicationSettings", Decision::Denied(DenyReason::Mutating)),
];
