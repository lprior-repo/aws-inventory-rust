// Pinned operation purposes: .local/aws-models/models/sagemaker-geospatial/service/2020-05-27/sagemaker-geospatial-2020-05-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to delete an Earth Observation job.
    (
        "DeleteEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to delete a Vector Enrichment job.
    (
        "DeleteVectorEnrichmentJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to export results of an Earth Observation job and optionally source images used as input to the EOJ to an Amazon S3 location.
    (
        "ExportEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to copy results of a Vector Enrichment job to an Amazon S3 location.
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
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Use this operation to get raster data collections.
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
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Use this operation to create an Earth observation job.
    (
        "StartEarthObservationJob",
        Decision::Denied(DenyReason::Paid),
    ),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Creates a Vector Enrichment job for the supplied job type. Currently, there are two supported job types: reverse geocoding and map matching.
    (
        "StartVectorEnrichmentJob",
        Decision::Denied(DenyReason::Paid),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Use this operation to stop an existing earth observation job.
    (
        "StopEarthObservationJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Stops the Vector Enrichment job for a given job ARN.
    (
        "StopVectorEnrichmentJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: The resource you want to tag.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: The resource you want to untag.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
