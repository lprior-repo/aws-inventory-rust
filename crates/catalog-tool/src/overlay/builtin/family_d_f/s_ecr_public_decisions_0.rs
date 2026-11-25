pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "BatchCheckLayerAvailability",
        "Requires repositoryName, layerDigests; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Checks the availability of one or more image layers that are within a repository in a public registry.",
    ),
    (
        "BatchDeleteImage",
        "Requires repositoryName, imageIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a list of specified images that are within a repository in a public registry.",
    ),
    (
        "CompleteLayerUpload",
        "Requires repositoryName, uploadId, layerDigests; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Informs Amazon ECR that the image layer upload is complete for a specified public registry, repository name, and upload ID.",
    ),
    (
        "CreateRepository",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a repository in a public registry.",
    ),
    (
        "DeleteRepository",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a repository in a public registry.",
    ),
    (
        "DeleteRepositoryPolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the repository policy that's associated with the specified repository.",
    ),
    (
        "DescribeImageTags",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the image tag details for a repository in a public registry.",
    ),
    (
        "GetRegistryCatalogData",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves catalog metadata for a public registry.",
    ),
    (
        "GetRepositoryPolicy",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the repository policy for the specified repository.",
    ),
    (
        "InitiateLayerUpload",
        "Requires repositoryName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Notifies Amazon ECR that you intend to upload an image layer.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List the tags for an Amazon ECR Public resource.",
    ),
    (
        "PutImage",
        "Requires repositoryName, imageManifest; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates the image manifest and tags that are associated with an image.",
    ),
    (
        "PutRegistryCatalogData",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Create or update the catalog data for a public registry.",
    ),
    (
        "PutRepositoryCatalogData",
        "Requires repositoryName, catalogData; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates the catalog data for a repository in a public registry.",
    ),
    (
        "SetRepositoryPolicy",
        "Requires repositoryName, policyText; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Applies a repository policy to the specified public repository to control access permissions.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates the specified tags to a resource with the specified resourceArn .",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes specified tags from a resource.",
    ),
    (
        "UploadLayerPart",
        "Requires repositoryName, uploadId, partFirstByte, partLastByte, layerPartBlob; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Uploads an image layer part to Amazon ECR.",
    ),
];
