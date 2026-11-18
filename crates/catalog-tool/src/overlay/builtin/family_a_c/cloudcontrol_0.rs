// Pinned source: .local/aws-models/models/cloudcontrol/service/2021-09-30/cloudcontrol-2021-09-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelResourceRequest",
            deferred(
                "Model requires RequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels the specified resource operation request. For more information, see Canceling resource operation requests in the Amazon Web Services Cloud Control API User Guide . Only resource operations requests with a status of…",
            ),
        ),
        (
            "CreateResource",
            deferred(
                "Model requires TypeName, DesiredState. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates the specified resource. For more information, see Creating a resource in the Amazon Web Services Cloud Control API User Guide . After you have initiated a resource creation request, you can monitor the progress of your…",
            ),
        ),
        (
            "DeleteResource",
            deferred(
                "Model requires TypeName, Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified resource. For details, see Deleting a resource in the Amazon Web Services Cloud Control API User Guide . After you have initiated a resource deletion request, you can monitor the progress of your request by…",
            ),
        ),
        (
            "GetResource",
            deferred(
                "Model requires TypeName, Identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about the current state of the specified resource. For details, see Reading a resource's current state . You can use this action to return information about an existing resource in your account and Amazon Web…",
            ),
        ),
        (
            "GetResourceRequestStatus",
            deferred(
                "Model requires RequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the current status of a resource operation request. For more information, see Tracking the progress of resource operation requests in the Amazon Web Services Cloud Control API User Guide .",
            ),
        ),
        (
            "ListResourceRequests",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ResourceRequestStatusSummaries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns existing resource operation requests. This includes requests of all status types. For more information, see Listing active resource operation requests in the Amazon Web Services Cloud Control API User Guide . Resource…",
            ),
        ),
        (
            "ListResources",
            deferred(
                "Requires TypeName: complete collection needs a CloudFormation resource-type discovery stream and cross-service binding. No reviewed same-service parent supplies TypeName; a single fixed type would silently narrow inventory.",
            ),
        ),
        (
            "UpdateResource",
            deferred(
                "Model requires TypeName, Identifier, PatchDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified property values in the resource. You specify your resource property updates as a list of patch operations contained in a JSON patch document that adheres to the RFC 6902 - JavaScript Object Notation (JSON)…",
            ),
        ),
    ])
}
