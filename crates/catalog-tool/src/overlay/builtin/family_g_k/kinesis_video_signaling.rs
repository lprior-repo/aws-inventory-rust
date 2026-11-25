use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "kinesis-video-signaling",
        approved,
        DEFERRED,
        "ICE configuration discloses username/password credentials and Alexa offer sends signaling content; every operation is denied.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

const DEFERRED: &[&str] = &[];

const MUTATING: &[&str] = &["SendAlexaOfferToMaster"];

const SECRET: &[&str] = &["GetIceServerConfig"];
