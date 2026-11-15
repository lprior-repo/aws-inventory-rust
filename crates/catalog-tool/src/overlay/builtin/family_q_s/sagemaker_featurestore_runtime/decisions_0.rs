// Pinned operation purposes: .local/aws-models/models/sagemaker-featurestore-runtime/service/2020-07-01/sagemaker-featurestore-runtime-2020-07-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Retrieves a batch of Records from a FeatureGroup .
    ("BatchGetRecord", Decision::Denied(DenyReason::Paid)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Writes a batch of Records to one or more FeatureGroup s. Use this API for bulk ingestion of records into the OnlineStore and OfflineStore . You can set the ingested records to expire at a given time to live (TTL) duration after the record's event time by specifying the TtlDuration parameter. A request level TtlDuration applies to all entries that do not specify their own TtlDuration .
    ("BatchWriteRecord", Decision::Denied(DenyReason::Content)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Record from a FeatureGroup in the OnlineStore . Feature Store supports both SoftDelete and HardDelete . For SoftDelete (default), feature columns are set to null and the record is no longer retrievable by GetRecord or BatchGetRecord . For HardDelete , the complete Record is removed from the OnlineStore . In both cases, Feature Store appends the deleted record marker to the OfflineStore .
    ("DeleteRecord", Decision::Denied(DenyReason::Mutating)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Use for OnlineStore serving from a FeatureStore . Only the latest records stored in the OnlineStore can be retrieved. If no Record with RecordIdentifierValue is found, then an empty result is returned.
    ("GetRecord", Decision::Denied(DenyReason::Paid)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Lists the RecordIdentifier values of all records stored in a FeatureGroup 's OnlineStore . This enables you to discover which records exist without retrieving the full record data.
    ("ListRecords", Decision::Denied(DenyReason::Content)),
    // Changes service state rather than reading inventory. Pinned purpose: The PutRecord API is used to ingest a list of Records into your feature group. If a new record’s EventTime is greater, the new record is written to both the OnlineStore and OfflineStore . Otherwise, the record is a historic record and it is written only to the OfflineStore . You can specify the ingestion to be applied to the OnlineStore , OfflineStore , or both by using the TargetStores request pa
    ("PutRecord", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates one or more feature values for an existing record in the specified feature group. Features that you do not include in the request remain unchanged. You can update up to 100 features per call. This operation is available only for feature groups that use the Standard_V2 or InMemory online store type. The record must already exist. If the record does not exist or has been soft-deleted, the op
    ("UpdateRecord", Decision::Denied(DenyReason::Mutating)),
];
