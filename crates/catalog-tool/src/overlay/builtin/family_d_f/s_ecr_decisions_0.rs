pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "BatchCheckLayerAvailability",
        "Requires repositoryName, layerDigests; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Checks the availability of one or more image layers in a repository.",
    ),
    (
        "BatchDeleteImage",
        "Requires repositoryName, imageIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a list of specified images within a repository.",
    ),
    (
        "BatchGetRepositoryScanningConfiguration",
        "Requires repositoryNames; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the scanning configuration for one or more repositories.",
    ),
    (
        "CompleteLayerUpload",
        "Requires repositoryName, uploadId, layerDigests; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Informs Amazon ECR that the image layer upload has completed for a specified registry, repository name, and upload ID.",
    ),
    (
        "CreatePullThroughCacheRule",
        "Requires ecrRepositoryPrefix, upstreamRegistryUrl; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a pull through cache rule.",
    ),
    (
        "CreateRepository",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a repository.",
    ),
    (
        "CreateRepositoryCreationTemplate",
        "Requires prefix, appliedFor; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a repository creation template.",
    ),
    (
        "DeleteLifecyclePolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the lifecycle policy associated with the specified repository.",
    ),
    (
        "DeletePullThroughCacheRule",
        "Requires ecrRepositoryPrefix; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a pull through cache rule.",
    ),
    (
        "DeleteRegistryPolicy",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes the registry permissions policy.",
    ),
    (
        "DeleteRepository",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a repository.",
    ),
    (
        "DeleteRepositoryCreationTemplate",
        "Requires prefix; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a repository creation template.",
    ),
    (
        "DeleteRepositoryPolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the repository policy associated with the specified repository.",
    ),
    (
        "DeleteSigningConfiguration",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes the registry's signing configuration.",
    ),
    (
        "DeregisterPullTimeUpdateExclusion",
        "Requires principalArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a principal from the pull time update exclusion list for a registry.",
    ),
    (
        "DescribeImageReplicationStatus",
        "Requires repositoryName, imageId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the replication status for a specified image.",
    ),
    (
        "DescribeImageScanFindings",
        "Requires repositoryName, imageId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the scan findings for the specified image.",
    ),
    (
        "DescribeImageSigningStatus",
        "Requires repositoryName, imageId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the signing status for a specified image.",
    ),
    (
        "DescribeRegistry",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes the settings for a registry.",
    ),
    (
        "DescribeReplicationBackfillResults",
        "Requires replicationBackfillId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the status, progress, and per-image results for a replication backfill.",
    ),
    (
        "GetAccountSetting",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the account setting value for the specified setting name.",
    ),
    (
        "GetLifecyclePolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the lifecycle policy for the specified repository.",
    ),
    (
        "GetLifecyclePolicyPreview",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the results of the lifecycle policy preview request for the specified repository.",
    ),
    (
        "GetRegistryPolicy",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves the permissions policy for a registry.",
    ),
    (
        "GetRegistryScanningConfiguration",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves the scanning configuration for a registry.",
    ),
    (
        "GetRepositoryPolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the repository policy for the specified repository.",
    ),
    (
        "GetSigningConfiguration",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves the registry's signing configuration, which defines rules for automatically signing images using Amazon Web Services Signer.",
    ),
    (
        "InitiateLayerUpload",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Notifies Amazon ECR that you intend to upload an image layer.",
    ),
    (
        "ListImageReferrers",
        "Requires repositoryName, subjectId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the artifacts associated with a specified subject image.",
    ),
    (
        "ListImages",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all the image IDs for the specified repository.",
    ),
    (
        "ListPullTimeUpdateExclusions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists the IAM principals that are excluded from having their image pull times recorded.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List the tags for an Amazon ECR resource.",
    ),
    (
        "PutAccountSetting",
        "Requires name, value; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Allows you to change the basic scan type version or registry policy scope.",
    ),
    (
        "PutImage",
        "Requires repositoryName, imageManifest; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates the image manifest and tags associated with an image.",
    ),
];
