use super::{defer, deny, extra_plans, voice_id_plans_1};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("voice-id", vec![
        ("AssociateFraudster", deny(DenyReason::Mutating)),
        ("CreateDomain", deny(DenyReason::Mutating)),
        ("CreateWatchlist", deny(DenyReason::Mutating)),
        ("DeleteDomain", deny(DenyReason::Mutating)),
        ("DeleteFraudster", deny(DenyReason::Mutating)),
        ("DeleteSpeaker", deny(DenyReason::Mutating)),
        ("DeleteWatchlist", deny(DenyReason::Mutating)),
        ("DescribeDomain", extra_plans::describe_domain()?),
        ("DescribeFraudster", deny(DenyReason::Content)),
        ("DescribeFraudsterRegistrationJob", deny(DenyReason::Content)),
        ("DescribeSpeaker", deny(DenyReason::Content)),
        ("DescribeSpeakerEnrollmentJob", deny(DenyReason::Content)),
        ("DescribeWatchlist", deny(DenyReason::Content)),
        ("DisassociateFraudster", deny(DenyReason::Mutating)),
        ("EvaluateSession", deny(DenyReason::Content)),
        ("ListDomains", voice_id_plans_1::list_domains()?),
        ("ListFraudsterRegistrationJobs", deny(DenyReason::Content)),
        ("ListFraudsters", deny(DenyReason::Content)),
        ("ListSpeakerEnrollmentJobs", deny(DenyReason::Content)),
        ("ListSpeakers", deny(DenyReason::Content)),
        ("ListTagsForResource", defer("Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.")),
        ("ListWatchlists", deny(DenyReason::Content)),
        ("OptOutSpeaker", defer("Missing executable inventory binding for DomainId, SpeakerId: required parent identifiers or request predicates are not present together in a reviewed enumeration item.")),
        ("StartFraudsterRegistrationJob", deny(DenyReason::Mutating)),
        ("StartSpeakerEnrollmentJob", deny(DenyReason::Mutating)),
        ("TagResource", deny(DenyReason::Mutating)),
        ("UntagResource", deny(DenyReason::Mutating)),
        ("UpdateDomain", deny(DenyReason::Mutating)),
        ("UpdateWatchlist", deny(DenyReason::Mutating)),
    ])
}
