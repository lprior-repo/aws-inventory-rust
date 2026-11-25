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
