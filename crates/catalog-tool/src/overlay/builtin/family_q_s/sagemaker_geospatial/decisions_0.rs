use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DeleteEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteVectorEnrichmentJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExportEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ExportVectorEnrichmentJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetEarthObservationJob",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Name, CreationTime, DurationInSeconds, Status, KmsKeyId, InputConfig, JobConfig, OutputBands, ExecutionRoleArn, ErrorDetails, ExportStatus, ExportErrorDetails, Tags. Pinned purpose: Get the details for a previously initiated Earth Observation job.",
        ),
    ),
    (
        "GetRasterDataCollection",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, Arn, Type, Description, DescriptionPageUrl, SupportedFilters, ImageSourceBands, Tags. Pinned purpose: Use this operation to get details of a specific raster data collection.",
        ),
    ),
    (
        "GetTile",
        Decision::Deferred(
            "Required selectors x, y, z, ImageAssets, Target, Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members BinaryFile. Pinned purpose: Gets a web mercator tile for the given Earth Observation job.",
        ),
    ),
    (
        "GetVectorEnrichmentJob",
        Decision::Deferred(
            "Required selectors Arn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Type, Name, CreationTime, DurationInSeconds, Status, KmsKeyId, InputConfig, JobConfig, ExecutionRoleArn, ErrorDetails, ExportStatus, ExportErrorDetails, Tags. Pinned purpose: Retrieves details of a Vector Enrichment Job for a given job Amazon Resource Name (ARN).",
        ),
    ),
    (
        "ListRasterDataCollections",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags attached to the resource.",
        ),
    ),
    (
        "SearchRasterDataCollection",
        Decision::Deferred(
            "Required selectors Arn, RasterDataCollectionQuery are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ApproximateResultCount, NextToken, Items. Pinned purpose: Allows you run image query on a specific raster data collection to get a list of the satellite imagery matching the selected filters.",
        ),
    ),
    (
        "StartEarthObservationJob",
        Decision::Denied(DenyReason::Paid),
    ),
    (
        "StartVectorEnrichmentJob",
        Decision::Denied(DenyReason::Paid),
    ),
    (
        "StopEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "StopVectorEnrichmentJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
