//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iot-data-plane/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "iot-data-plane",
        approved,
        DEFERRED,
        "Shadow/retained-message payloads are content and retained-message enumeration documents messaging charges. Connection/subscription/shadow-name APIs need explicit device endpoint and identity review; none is approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
            (policy::DenyReason::Paid, PAID),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetConnection",
    "ListNamedShadowsForThing",
    "ListSubscriptions",
];

const MUTATING: &[&str] = &[
    "DeleteConnection",
    "DeleteThingShadow",
    "Publish",
    "SendDirectMessage",
    "UpdateThingShadow",
];

const CONTENT: &[&str] = &["GetRetainedMessage", "GetThingShadow"];

const PAID: &[&str] = &["ListRetainedMessages"];
