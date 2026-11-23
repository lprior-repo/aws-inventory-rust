//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iotfleetwise/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListFleets",
            metadata("fleetSummaries")?,
        ),
        (
            "ListSignalCatalogs",
            policy::query(
                policy::enumerate(
                    "summaries",
                    policy::arn("arn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListModelManifests",
            metadata("summaries")?,
        ),
        (
            "ListDecoderManifests",
            metadata("summaries")?,
        ),
        (
            "ListVehicles",
            metadata("vehicleSummaries")?,
        ),
        (
            "ListCampaigns",
            metadata("campaignSummaries")?,
        ),
        (
            "ListStateTemplates",
            metadata("summaries")?,
        ),
    ];
    policy::finish(
        "iotfleetwise",
        approved,
        DEFERRED,
        "Approved lists expose fleet, manifest, vehicle, campaign, signal-catalog and state-template metadata. Feature/region gates can still reject calls; signal values and detailed decoder/model content are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

fn metadata(items: &str) -> Result<crate::overlay::OperationOverlay, NameError> {
    policy::fixed(
        policy::enumerate(items, policy::arn("arn")?, Some(("nextToken", "nextToken")))?,
        "listResponseScope",
        "METADATA_ONLY",
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetCampaign",
    "GetDecoderManifest",
    "GetEncryptionConfiguration",
    "GetFleet",
    "GetLoggingOptions",
    "GetModelManifest",
    "GetRegisterAccountStatus",
    "GetSignalCatalog",
    "GetStateTemplate",
    "GetVehicle",
    "GetVehicleStatus",
    "ListDecoderManifestNetworkInterfaces",
    "ListDecoderManifestSignals",
    "ListFleetsForVehicle",
    "ListModelManifestNodes",
    "ListSignalCatalogNodes",
    "ListTagsForResource",
    "ListVehiclesInFleet",
];

const MUTATING: &[&str] = &[
    "AssociateVehicleFleet",
    "BatchCreateVehicle",
    "BatchUpdateVehicle",
    "CreateCampaign",
    "CreateDecoderManifest",
    "CreateFleet",
    "CreateModelManifest",
    "CreateSignalCatalog",
    "CreateStateTemplate",
    "CreateVehicle",
    "DeleteCampaign",
    "DeleteDecoderManifest",
    "DeleteFleet",
    "DeleteModelManifest",
    "DeleteSignalCatalog",
    "DeleteStateTemplate",
    "DeleteVehicle",
    "DisassociateVehicleFleet",
    "ImportDecoderManifest",
    "ImportSignalCatalog",
    "PutEncryptionConfiguration",
    "PutLoggingOptions",
    "RegisterAccount",
    "TagResource",
    "UntagResource",
    "UpdateCampaign",
    "UpdateDecoderManifest",
    "UpdateFleet",
    "UpdateModelManifest",
    "UpdateSignalCatalog",
    "UpdateStateTemplate",
    "UpdateVehicle",
];
