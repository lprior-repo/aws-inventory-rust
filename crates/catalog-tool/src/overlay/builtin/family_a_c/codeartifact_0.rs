use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateExternalConnection",
            deferred(
                "Model requires domain, repository, externalConnection. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an existing external connection to a repository. One external connection is allowed per repository. A repository can have one or more upstream repositories, or an external connection.",
            ),
        ),
        (
            "CopyPackageVersions",
            deferred(
                "Model requires domain, sourceRepository, destinationRepository, format, package. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Copies package versions from one repository to another repository in the same domain. You must specify versions or versionRevisions . You cannot specify both.",
            ),
        ),
        (
            "CreateDomain",
            deferred(
                "Model requires domain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain. CodeArtifact domains make it easier to manage multiple repositories across an organization. You can use a domain to apply permissions across many repositories owned by different Amazon Web Services accounts. An…",
            ),
        ),
        (
            "CreatePackageGroup",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a package group. For more information about creating package groups, including example CLI commands, see Create a package group in the CodeArtifact User Guide .",
            ),
        ),
        (
            "CreateRepository",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a repository.",
            ),
        ),
        (
            "DeleteDomain",
            deferred(
                "Model requires domain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a domain. You cannot delete a domain that contains repositories. If you want to delete a domain with repositories, first delete its repositories.",
            ),
        ),
        (
            "DeleteDomainPermissionsPolicy",
            deferred(
                "Model requires domain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the resource policy set on a domain.",
            ),
        ),
        (
            "DeletePackage",
            deferred(
                "Model requires domain, repository, format, package. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a package and all associated package versions. A deleted package cannot be restored. To delete one or more package versions, use the DeletePackageVersions API.",
            ),
        ),
        (
            "DeletePackageGroup",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a package group. Deleting a package group does not delete packages or package versions associated with the package group. When a package group is deleted, the direct child package groups will become children of the…",
            ),
        ),
        (
            "DeletePackageVersions",
            deferred(
                "Model requires domain, repository, format, package, versions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes one or more versions of a package. A deleted package version cannot be restored in your repository. If you want to remove a package version from your repository and be able to restore it later, set its status to Archived…",
            ),
        ),
        (
            "DeleteRepository",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a repository.",
            ),
        ),
        (
            "DeleteRepositoryPermissionsPolicy",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the resource policy that is set on a repository. After a resource policy is deleted, the permissions allowed and denied by the deleted policy are removed. The effect of deleting a resource policy might not be immediate.…",
            ),
        ),
        (
            "DescribeDomain",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("domain")?,
                        Binding::From {
                            operation: name("ListDomains")?,
                            path: path("name")?,
                        },
                    ),
                    (
                        path("domainOwner")?,
                        Binding::From {
                            operation: name("ListDomains")?,
                            path: path("owner")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "domain",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribePackage",
            deferred(
                "Model requires domain, repository, format, package. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a PackageDescription object that contains information about the requested package.",
            ),
        ),
        (
            "DescribePackageGroup",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a PackageGroupDescription object that contains information about the requested package group.",
            ),
        ),
        (
            "DescribePackageVersion",
            deferred(
                "Model requires domain, repository, format, package, packageVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a PackageVersionDescription object that contains information about the requested package version.",
            ),
        ),
        (
            "DescribeRepository",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("domain")?,
                        Binding::From {
                            operation: name("ListRepositories")?,
                            path: path("domainName")?,
                        },
                    ),
                    (
                        path("repository")?,
                        Binding::From {
                            operation: name("ListRepositories")?,
                            path: path("name")?,
                        },
                    ),
                    (
                        path("domainOwner")?,
                        Binding::From {
                            operation: name("ListRepositories")?,
                            path: path("domainOwner")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "repository",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DisassociateExternalConnection",
            deferred(
                "Model requires domain, repository, externalConnection. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an existing external connection from a repository.",
            ),
        ),
        (
            "DisposePackageVersions",
            deferred(
                "Model requires domain, repository, format, package, versions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the assets in package versions and sets the package versions' status to Disposed . A disposed package version cannot be restored in your repository because its assets are deleted. To view all disposed package versions in…",
            ),
        ),
        (
            "GetAssociatedPackageGroup",
            deferred(
                "Model requires domain, format, package. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the most closely associated package group to the specified package. This API does not require that the package exist in any repository in the domain. As such, GetAssociatedPackageGroup can be used to see which package…",
            ),
        ),
        ("GetAuthorizationToken", denied(DenyReason::Secret)),
        (
            "GetDomainPermissionsPolicy",
            deferred(
                "Model requires domain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the resource policy attached to the specified domain. The policy is a resource-based policy, not an identity-based policy. For more information, see Identity-based policies and resource-based policies in the IAM User…",
            ),
        ),
        (
            "GetPackageVersionAsset",
            deferred(
                "Model requires domain, repository, format, package, packageVersion, asset. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an asset (or file) that is in a package. For example, for a Maven package version, use GetPackageVersionAsset to download a JAR file, a POM file, or any other assets in the package version.",
            ),
        ),
        (
            "GetPackageVersionReadme",
            deferred(
                "Model requires domain, repository, format, package, packageVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the readme file or descriptive text for a package version. The returned text might contain formatting. For example, it might contain formatting for Markdown or reStructuredText.",
            ),
        ),
        (
            "GetRepositoryEndpoint",
            deferred(
                "Model requires domain, repository, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the endpoint of a repository for a specific package format. A repository has one endpoint for each package format: cargo generic maven npm nuget pypi ruby swift",
            ),
        ),
        (
            "GetRepositoryPermissionsPolicy",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the resource policy that is set on a repository.",
            ),
        ),
        (
            "ListAllowedRepositoriesForGroup",
            deferred(
                "Model requires domain, packageGroup, originRestrictionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the repositories in the added repositories list of the specified restriction type for a package group. For more information about restriction types and added repository lists, see Package group origin controls in the…",
            ),
        ),
        (
            "ListAssociatedPackages",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of packages associated with the requested package group. For information package group association and matching, see Package group definition syntax and matching behavior in the CodeArtifact User Guide .",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListDomains",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "domains",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPackageGroups",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("domain")?,
                        Binding::From {
                            operation: name("ListDomains")?,
                            path: path("name")?,
                        },
                    ),
                    (
                        path("domainOwner")?,
                        Binding::From {
                            operation: name("ListDomains")?,
                            path: path("owner")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "packageGroups",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListPackageVersionAssets",
            deferred(
                "Model requires domain, repository, format, package, packageVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of AssetSummary objects for assets in a package version.",
            ),
        ),
        (
            "ListPackageVersionDependencies",
            deferred(
                "Model requires domain, repository, format, package, packageVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the direct dependencies for a package version. The dependencies are returned as PackageDependency objects. CodeArtifact extracts the dependencies for a package version from the metadata file for the package format (for…",
            ),
        ),
        (
            "ListPackageVersions",
            deferred(
                "Model requires domain, repository, format, package. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of PackageVersionSummary objects for package versions in a repository that match the request parameters. Package versions of all statuses will be returned by default when calling list-package-versions with no…",
            ),
        ),
        (
            "ListPackages",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of PackageSummary objects for packages in a repository that match the request parameters.",
            ),
        ),
        (
            "ListRepositories",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "repositories",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListRepositoriesInDomain",
            deferred(
                "Model requires domain. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of RepositorySummary objects. Each RepositorySummary contains information about a repository in the specified domain and that matches the input parameters.",
            ),
        ),
        (
            "ListSubPackageGroups",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of direct children of the specified package group. For information package group hierarchy, see Package group definition syntax and matching behavior in the CodeArtifact User Guide .",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about Amazon Web Services tags for a specified Amazon Resource Name (ARN) in CodeArtifact.",
            ),
        ),
        (
            "PublishPackageVersion",
            deferred(
                "Model requires domain, repository, format, package, packageVersion, assetContent, assetName, assetSHA256. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new package version containing one or more assets (or files). The unfinished flag can be used to keep the package version in the Unfinished state until all of its assets have been uploaded (see Package version status in…",
            ),
        ),
        (
            "PutDomainPermissionsPolicy",
            deferred(
                "Model requires domain, policyDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets a resource policy on a domain that specifies permissions to access it. When you call PutDomainPermissionsPolicy , the resource policy on the domain is ignored when evaluting permissions. This ensures that the owner of a…",
            ),
        ),
        (
            "PutPackageOriginConfiguration",
            deferred(
                "Model requires domain, repository, format, package, restrictions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the package origin configuration for a package. The package origin configuration determines how new versions of a package can be added to a repository. You can allow or block direct publishing of new package versions, or…",
            ),
        ),
        (
            "PutRepositoryPermissionsPolicy",
            deferred(
                "Model requires domain, repository, policyDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the resource policy on a repository that specifies permissions to access it. When you call PutRepositoryPermissionsPolicy , the resource policy on the repository is ignored when evaluting permissions. This ensures that the…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates tags for a resource in CodeArtifact.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from a resource in CodeArtifact.",
            ),
        ),
        (
            "UpdatePackageGroup",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a package group. This API cannot be used to update a package group's origin configuration or pattern. To update a package group's origin configuration, use UpdatePackageGroupOriginConfiguration .",
            ),
        ),
        (
            "UpdatePackageGroupOriginConfiguration",
            deferred(
                "Model requires domain, packageGroup. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the package origin configuration for a package group. The package origin configuration determines how new versions of a package can be added to a repository. You can allow or block direct publishing of new package…",
            ),
        ),
        (
            "UpdatePackageVersionsStatus",
            deferred(
                "Model requires domain, repository, format, package, versions, targetStatus. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the status of one or more versions of a package. Using UpdatePackageVersionsStatus , you can update the status of package versions to Archived , Published , or Unlisted . To set the status of a package version to Disposed…",
            ),
        ),
        (
            "UpdateRepository",
            deferred(
                "Model requires domain, repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update the properties of a repository.",
            ),
        ),
    ])
}
