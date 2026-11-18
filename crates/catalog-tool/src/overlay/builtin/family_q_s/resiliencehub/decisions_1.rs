// Pinned operation purposes: .local/aws-models/models/resiliencehub/service/2020-04-30/resiliencehub-2020-04-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Updates the Resilience Hub application version. This API updates the Resilience Hub application draft version. To use this information for running resiliency assessments, you must publish the Resilience Hub application using the PublishAppVersion API.
    ("UpdateAppVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing Application Component in the Resilience Hub application. This API updates the Resilience Hub application draft version. To use this Application Component for running assessments, you must publish the Resilience Hub application using the PublishAppVersion API.
    (
        "UpdateAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource details in the Resilience Hub application. This action has no effect outside Resilience Hub. This API updates the Resilience Hub application draft version. To use this resource for running resiliency assessments, you must publish the Resilience Hub application using the PublishAppVersion API. To update application version with new physicalResourceID , you must call ResolveAppV
    (
        "UpdateAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a resiliency policy. Resilience Hub allows you to provide a value of zero for rtoInSecs and rpoInSecs of your resiliency policy. But, while assessing your application, the lowest possible assessment result is near zero. Hence, if you provide value zero for rtoInSecs and rpoInSecs , the estimated workload RTO and estimated workload RPO result will be near zero and the Compliance status for
    (
        "UpdateResiliencyPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
];
