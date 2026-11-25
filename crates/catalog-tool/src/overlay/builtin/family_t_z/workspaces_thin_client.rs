use super::super::service;
use super::{defer, deny, workspaces_thin_client_plans_1};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workspaces-thin-client",
        vec![
            ("CreateEnvironment", deny(DenyReason::Mutating)),
            ("DeleteDevice", deny(DenyReason::Mutating)),
            ("DeleteEnvironment", deny(DenyReason::Mutating)),
            ("DeregisterDevice", deny(DenyReason::Mutating)),
            ("GetDevice", workspaces_thin_client_plans_1::get_device()?),
            ("GetEnvironment", deny(DenyReason::Secret)),
            ("GetSoftwareSet", deny(DenyReason::ReferenceData)),
            (
                "ListDevices",
                workspaces_thin_client_plans_1::list_devices()?,
            ),
            ("ListEnvironments", deny(DenyReason::Secret)),
            ("ListSoftwareSets", deny(DenyReason::ReferenceData)),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateDevice", deny(DenyReason::Mutating)),
            ("UpdateEnvironment", deny(DenyReason::Mutating)),
            ("UpdateSoftwareSet", deny(DenyReason::Mutating)),
        ],
    )
}
