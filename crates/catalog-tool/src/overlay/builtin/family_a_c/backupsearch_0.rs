// Pinned source: .local/aws-models/models/backupsearch/service/2018-05-10/backupsearch-2018-05-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetSearchJob",
            deferred(
                "Model requires SearchJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation retrieves metadata of a search job, including its progress.",
            ),
        ),
        (
            "GetSearchResultExportJob",
            deferred(
                "Model requires ExportJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation retrieves the metadata of an export job. An export job is an operation that transmits the results of a search job to a specified S3 bucket in a .csv file. An export job allows you to retain results of a search…",
            ),
        ),
        (
            "ListSearchJobBackups",
            deferred(
                "Model requires SearchJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation returns a list of all backups (recovery points) in a paginated format that were included in the search job. If a search does not display an expected backup in the results, you can call this operation to display…",
            ),
        ),
        (
            "ListSearchJobResults",
            deferred(
                "Model requires SearchJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation returns a list of a specified search job.",
            ),
        ),
        // Model: This operation returns a list of search jobs belonging to an account.
        (
            "ListSearchJobs",
            OperationOverlay {
                queryable: vec![path("SearchJobArn")?],
                ..read(
                    "SearchJobs",
                    Identity::Arn(path("SearchJobArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListSearchResultExportJobs",
            deferred(
                "Operation documentation says this operation exports search results, although its input/output look like metadata listing. Fail closed on the conflicting side-effect documentation; do not approve by the List prefix.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation returns the tags for a resource type.",
            ),
        ),
        (
            "StartSearchJob",
            deferred(
                "Model requires SearchScope. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation creates a search job which returns recovery points filtered by SearchScope and items filtered by ItemFilters. You can optionally include ClientToken, EncryptionKeyArn, Name, and/or Tags.",
            ),
        ),
        (
            "StartSearchResultExportJob",
            deferred(
                "Model requires SearchJobIdentifier, ExportSpecification. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operations starts a job to export the results of search job to a designated S3 bucket.",
            ),
        ),
        (
            "StopSearchJob",
            deferred(
                "Model requires SearchJobIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operations ends a search job. Only a search job with a status of RUNNING can be stopped.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation puts tags on the resource you indicate.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation removes tags from the specified resource.",
            ),
        ),
    ])
}
