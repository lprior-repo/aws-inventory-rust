// Pinned source: .local/aws-models/models/cloudtrail-data/service/2021-08-11/cloudtrail-data-2021-08-11.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("PutAuditEvents", deferred("Model requires auditEvents, channelArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Ingests your application events into CloudTrail Lake. A required parameter, auditEvents , accepts the JSON records (also called payload ) of events that you want CloudTrail to ingest. You can add up to 100 of these events (or up…")),
    ])
}
