// Pinned source: .local/aws-models/models/appintegrations/service/2020-07-29/appintegrations-2020-07-29.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateApplication", deferred("Model requires Name, Namespace, ApplicationSourceConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and persists an Application resource.")),
        ("CreateDataIntegration", deferred("Model requires Name, KmsKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and persists a DataIntegration resource. You cannot create a DataIntegration association for a DataIntegration that has been previously associated. Use a different DataIntegration, or recreate the DataIntegration using…")),
        ("CreateDataIntegrationAssociation", deferred("Model requires DataIntegrationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and persists a DataIntegrationAssociation resource.")),
        ("CreateEventIntegration", deferred("Model requires Name, EventFilter, EventBridgeBus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an EventIntegration, given a specified name, description, and a reference to an Amazon EventBridge bus in your account and a partner event source that pushes events to that bus. No objects are created in the your account,…")),
        ("DeleteApplication", deferred("Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an application. If the application has associations, you must delete them first. Alternatively, use the force option to delete the application and remove its associations.")),
        ("DeleteDataIntegration", deferred("Model requires DataIntegrationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the DataIntegration. Only DataIntegrations that don't have any DataIntegrationAssociations can be deleted. Deleting a DataIntegration also deletes the underlying Amazon AppFlow flow and service linked role. You cannot…")),
        ("DeleteEventIntegration", deferred("Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified existing event integration. If the event integration is associated with clients, the request is rejected.")),
        // Model: Get an Application resource.
        ("GetApplication", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("Arn")?, Binding::From { operation: name("ListApplications")?, path: path("Arn")? })].into_iter().collect(),
            tags: Some(path("Tags")?),
            queryable: vec![path("Arn")?],
            ..read("$", Identity::Arn(path("Arn")?), Scope::Region, Paginator::None)?
        }),
        ("GetDataIntegration", deferred("Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the DataIntegration. You cannot create a DataIntegration association for a DataIntegration that has been previously associated. Use a different DataIntegration, or recreate the DataIntegration using the…")),
        // Model: Returns information about the event integration.
        ("GetEventIntegration", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("Name")?, Binding::From { operation: name("ListEventIntegrations")?, path: path("Name")? })].into_iter().collect(),
            tags: Some(path("Tags")?),
            queryable: vec![path("EventIntegrationArn")?],
            ..read("$", Identity::Arn(path("EventIntegrationArn")?), Scope::Region, Paginator::None)?
        }),
        // Model: Returns a paginated list of application associations for an application.
        ("ListApplicationAssociations", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("ApplicationId")?, Binding::From { operation: name("ListApplications")?, path: path("Id")? })].into_iter().collect(),
            queryable: vec![path("ApplicationAssociationArn")?],
            ..read("ApplicationAssociations", Identity::Arn(path("ApplicationAssociationArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Lists applications in the account.
        ("ListApplications", OperationOverlay {
            queryable: vec![path("Arn")?],
            ..read("Applications", Identity::Arn(path("Arn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns a paginated list of DataIntegration associations in the account. You cannot create a DataIntegration association for a DataIntegration that has been previously associated. Use a different…
        ("ListDataIntegrationAssociations", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("DataIntegrationIdentifier")?, Binding::From { operation: name("ListDataIntegrations")?, path: path("Arn")? })].into_iter().collect(),
            queryable: vec![path("DataIntegrationAssociationArn")?],
            ..read("DataIntegrationAssociations", Identity::Arn(path("DataIntegrationAssociationArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Returns a paginated list of DataIntegrations in the account. You cannot create a DataIntegration association for a DataIntegration that has been previously associated. Use a different…
        ("ListDataIntegrations", OperationOverlay {
            queryable: vec![path("Arn")?],
            ..read("DataIntegrations", Identity::Arn(path("Arn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Returns a paginated list of event integration associations in the account.
        ("ListEventIntegrationAssociations", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("EventIntegrationName")?, Binding::From { operation: name("ListEventIntegrations")?, path: path("Name")? })].into_iter().collect(),
            queryable: vec![path("EventIntegrationAssociationArn")?],
            ..read("EventIntegrationAssociations", Identity::Arn(path("EventIntegrationAssociationArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Returns a paginated list of event integrations in the account.
        ("ListEventIntegrations", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("EventIntegrationArn")?],
            ..read("EventIntegrations", Identity::Arn(path("EventIntegrationArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags for the specified resource.")),
        ("TagResource", deferred("Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds the specified tags to the specified resource.")),
        ("UntagResource", deferred("Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified resource.")),
        ("UpdateApplication", deferred("Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates and persists an Application resource.")),
        ("UpdateDataIntegration", deferred("Model requires Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the description of a DataIntegration. You cannot create a DataIntegration association for a DataIntegration that has been previously associated. Use a different DataIntegration, or recreate the DataIntegration using the…")),
        ("UpdateDataIntegrationAssociation", deferred("Model requires DataIntegrationIdentifier, DataIntegrationAssociationIdentifier, ExecutionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates and persists a DataIntegrationAssociation resource. Updating a DataIntegrationAssociation with ExecutionConfiguration will rerun the on-demand job.")),
        ("UpdateEventIntegration", deferred("Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the description of an event integration.")),
    ])
}
