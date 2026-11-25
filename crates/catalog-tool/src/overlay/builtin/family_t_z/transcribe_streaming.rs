use super::super::service;
use super::deny;
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "transcribe-streaming",
        vec![
            ("GetMedicalScribeStream", deny(DenyReason::Content)),
            (
                "StartCallAnalyticsStreamTranscription",
                deny(DenyReason::Mutating),
            ),
            ("StartMedicalScribeStream", deny(DenyReason::Mutating)),
            (
                "StartMedicalStreamTranscription",
                deny(DenyReason::Mutating),
            ),
            ("StartStreamTranscription", deny(DenyReason::Mutating)),
        ],
    )
}
