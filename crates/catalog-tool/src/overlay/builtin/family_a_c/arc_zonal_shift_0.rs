// Pinned source: .local/aws-models/models/arc-zonal-shift/service/2022-10-30/arc-zonal-shift-2022-10-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelPracticeRun",
            deferred(
                "Model requires zonalShiftId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancel an in-progress practice run zonal shift in Amazon Application Recovery Controller.",
            ),
        ),
        (
            "CancelZonalShift",
            deferred(
                "Model requires zonalShiftId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancel a zonal shift in Amazon Application Recovery Controller. To cancel the zonal shift, specify the zonal shift ID. A zonal shift can be one that you've started for a resource in your Amazon Web Services account in an Amazon…",
            ),
        ),
        (
            "CreatePracticeRunConfiguration",
            deferred(
                "Model requires resourceIdentifier, outcomeAlarms. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A practice run configuration for zonal autoshift is required when you enable zonal autoshift. A practice run configuration includes specifications for blocked dates and blocked time windows, and for Amazon CloudWatch alarms that…",
            ),
        ),
        (
            "DeletePracticeRunConfiguration",
            deferred(
                "Model requires resourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the practice run configuration for a resource. Before you can delete a practice run configuration for a resource., you must disable zonal autoshift for the resource. Practice runs must be configured for zonal autoshift to…",
            ),
        ),
        (
            "GetAutoshiftObserverNotificationStatus",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is status; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the status of the autoshift observer notification. Autoshift observer notifications notify you through Amazon EventBridge when there is an autoshift event for zonal autoshift. The status can be ENABLED or DISABLED . When…",
            ),
        ),
        (
            "GetManagedResource",
            deferred(
                "Model requires resourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get information about a resource that's been registered for zonal shifts with Amazon Application Recovery Controller in this Amazon Web Services Region. Resources that are registered for zonal shifts are managed resources in ARC.…",
            ),
        ),
        (
            "ListAutoshifts",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the autoshifts for an Amazon Web Services Region. By default, the call returns only ACTIVE autoshifts. Optionally, you can specify the status parameter to return COMPLETED autoshifts.",
            ),
        ),
        // Model: Lists all the resources in your Amazon Web Services account in this Amazon Web Services Region that are managed for zonal shifts in Amazon Application Recovery Controller, and information about them.…
        (
            "ListManagedResources",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "items",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists all active and completed zonal shifts in Amazon Application Recovery Controller in your Amazon Web Services account in this Amazon Web Services Region. ListZonalShifts returns…
        (
            "ListZonalShifts",
            OperationOverlay {
                queryable: vec![path("zonalShiftId")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("zonalShiftId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "StartPracticeRun",
            deferred(
                "Model requires resourceIdentifier, awayFrom, comment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Start an on-demand practice run zonal shift in Amazon Application Recovery Controller. With zonal autoshift enabled, you can start an on-demand practice run to verify preparedness at any time. Amazon Web Services also runs…",
            ),
        ),
        (
            "StartZonalShift",
            deferred(
                "Model requires resourceIdentifier, awayFrom, expiresIn, comment. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: You start a zonal shift to temporarily move load balancer traffic away from an Availability Zone in an Amazon Web Services Region, to help your application recover immediately, for example, from a developer's bad code deployment…",
            ),
        ),
        (
            "UpdateAutoshiftObserverNotificationStatus",
            deferred(
                "Model requires status. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update the status of autoshift observer notification. Autoshift observer notification enables you to be notified, through Amazon EventBridge, when there is an autoshift event for zonal autoshift. If the status is ENABLED , ARC…",
            ),
        ),
        (
            "UpdatePracticeRunConfiguration",
            deferred(
                "Model requires resourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update a practice run configuration to change one or more of the following: add, change, or remove the blocking alarm; change the outcome alarm; or add, change, or remove blocking dates or time windows.",
            ),
        ),
        (
            "UpdateZonalAutoshiftConfiguration",
            deferred(
                "Model requires resourceIdentifier, zonalAutoshiftStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The zonal autoshift configuration for a resource includes the practice run configuration and the status for running autoshifts, zonal autoshift status. When a resource has a practice run configuration, ARC starts weekly zonal…",
            ),
        ),
        (
            "UpdateZonalShift",
            deferred(
                "Model requires zonalShiftId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update an active zonal shift in Amazon Application Recovery Controller in your Amazon Web Services account. You can update a zonal shift to set a new expiration, or edit or replace the comment for the zonal shift.",
            ),
        ),
    ])
}
