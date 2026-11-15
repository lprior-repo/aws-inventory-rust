// Pinned source: .local/aws-models/models/agent-registry/service/2025-12-01/agent-registry-2025-12-01.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("BatchGetDiscoverableRegistryRecord", deferred("Model requires entries. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves multiple discoverable registry records by ID from a single registry. Records that cannot be retrieved are reported individually in the errors list rather than failing the entire request.")),
        ("ListDiscoverableRegistryRecords", deferred("Model requires registryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the discoverable registry records in a registry. You can optionally filter and paginate the results.")),
        ("SearchDiscoverableRegistryRecords", deferred("Model requires searchQuery, registryIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches the discoverable registry records in a registry using a natural language query. Returns metadata for the matching records ordered by relevance.")),
    ])
}
