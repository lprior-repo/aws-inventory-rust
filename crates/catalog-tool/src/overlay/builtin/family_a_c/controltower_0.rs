use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateLandingZone",
            deferred(
                "Model requires version. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new landing zone. This API call starts an asynchronous operation that creates and configures a landing zone, based on the parameters specified in the manifest JSON file.",
            ),
        ),
        (
            "DeleteLandingZone",
            deferred(
                "Model requires landingZoneIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Decommissions a landing zone. This API call starts an asynchronous operation that deletes Amazon Web Services Control Tower resources deployed in accounts managed by Amazon Web Services Control Tower. Decommissioning a landing…",
            ),
        ),
        (
            "DisableBaseline",
            deferred(
                "Model requires enabledBaselineIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disable an EnabledBaseline resource on the specified Target. This API starts an asynchronous operation to remove all resources deployed as part of the baseline enablement. The resource will vary depending on the enabled baseline.…",
            ),
        ),
        (
            "DisableControl",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is operationIdentifier; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: This API call turns off a control. It starts an asynchronous operation that deletes Amazon Web Services resources on the specified organizational unit and the accounts it contains. The resources will vary according to the control…",
            ),
        ),
        (
            "EnableBaseline",
            deferred(
                "Model requires baselineVersion, baselineIdentifier, targetIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enable (apply) a Baseline to a Target. This API starts an asynchronous operation to deploy resources specified by the Baseline to the specified Target. For usage examples, see the Amazon Web Services Control Tower User Guide .",
            ),
        ),
        (
            "EnableControl",
            deferred(
                "Model requires controlIdentifier, targetIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API call activates a control. It starts an asynchronous operation that creates Amazon Web Services resources on the specified organizational unit and the accounts it contains. The resources created will vary according to the…",
            ),
        ),
        (
            "GetBaseline",
            deferred(
                "Model requires baselineIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve details about an existing Baseline resource by specifying its identifier. For usage examples, see the Amazon Web Services Control Tower User Guide .",
            ),
        ),
        (
            "GetBaselineOperation",
            deferred(
                "Model requires operationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of an asynchronous baseline operation, as initiated by any of these APIs: EnableBaseline , DisableBaseline , UpdateEnabledBaseline , ResetEnabledBaseline . A status message is displayed in case of operation…",
            ),
        ),
        (
            "GetControlOperation",
            deferred(
                "Model requires operationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the status of a particular EnableControl or DisableControl operation. Displays a message in case of error. Details for an operation are available for 90 days. For usage examples, see the Controls Reference Guide .",
            ),
        ),
        (
            "GetEnabledBaseline",
            deferred(
                "Model requires enabledBaselineIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieve details of an EnabledBaseline resource by specifying its identifier.",
            ),
        ),
        (
            "GetEnabledControl",
            deferred(
                "Model requires enabledControlIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about an enabled control. For usage examples, see the Controls Reference Guide .",
            ),
        ),
        (
            "GetLandingZone",
            deferred(
                "Model requires landingZoneIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns details about the landing zone. Displays a message in case of error.",
            ),
        ),
        (
            "GetLandingZoneOperation",
            deferred(
                "Model requires operationIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the status of the specified landing zone operation. Details for an operation are available for 90 days.",
            ),
        ),
        (
            "ListBaselines",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is baselines; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a summary list of all available baselines. For usage examples, see the Amazon Web Services Control Tower User Guide .",
            ),
        ),
        (
            "ListControlOperations",
            OperationOverlay {
                queryable: vec![path("operationIdentifier")?],
                ..read(
                    "controlOperations",
                    Identity::Key(vec![path("operationIdentifier")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListEnabledBaselines",
            OperationOverlay {
                fixed: [(path("includeChildren")?, FixedValue::Boolean(true))]
                    .into_iter()
                    .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "enabledBaselines",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListEnabledControls",
            OperationOverlay {
                fixed: [(path("includeChildren")?, FixedValue::Boolean(true))]
                    .into_iter()
                    .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "enabledControls",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListLandingZoneOperations",
            OperationOverlay {
                queryable: vec![path("operationIdentifier")?],
                ..read(
                    "landingZoneOperations",
                    Identity::Key(vec![path("operationIdentifier")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        ("ListLandingZones", denied(DenyReason::Mutating)),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags associated with the resource. For usage examples, see the Controls Reference Guide .",
            ),
        ),
        (
            "ResetEnabledBaseline",
            deferred(
                "Model requires enabledBaselineIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Re-enables an EnabledBaseline resource. For example, this API can re-apply the existing Baseline after a new member account is moved to the target OU. For usage examples, see the Amazon Web Services Control Tower User Guide .",
            ),
        ),
        (
            "ResetEnabledControl",
            deferred(
                "Model requires enabledControlIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resets an enabled control. Does not work for controls implemented with SCPs.",
            ),
        ),
        (
            "ResetLandingZone",
            deferred(
                "Model requires landingZoneIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API call resets a landing zone. It starts an asynchronous operation that resets the landing zone to the parameters specified in the original configuration, which you specified in the manifest file. Nothing in the manifest…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Applies tags to a resource. For usage examples, see the Controls Reference Guide .",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from a resource. For usage examples, see the Controls Reference Guide .",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateEnabledBaseline",
            deferred(
                "Model requires baselineVersion, enabledBaselineIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an EnabledBaseline resource's applied parameters or version. For usage examples, see the Amazon Web Services Control Tower User Guide .",
            ),
        ),
        (
            "UpdateEnabledControl",
            deferred(
                "Model requires parameters, enabledControlIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration of an already enabled control. If the enabled control shows an EnablementStatus of SUCCEEDED, supply parameters that are different from the currently configured parameters. Otherwise, Amazon Web Services…",
            ),
        ),
        (
            "UpdateLandingZone",
            deferred(
                "Model requires version, landingZoneIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This API call updates the landing zone. It starts an asynchronous operation that updates the landing zone based on the new landing zone version, or on the changed parameters specified in the updated manifest file.",
            ),
        ),
    ])
}
