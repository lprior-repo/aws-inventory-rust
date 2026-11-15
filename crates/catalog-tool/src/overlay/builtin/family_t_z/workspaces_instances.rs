use super::{defer, deny, workspaces_instances_plans_1};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("workspaces-instances", vec![
        ("AssociateVolume", deny(DenyReason::Mutating)),
        ("CreateVolume", deny(DenyReason::Mutating)),
        ("CreateWorkspaceInstance", deny(DenyReason::Mutating)),
        ("DeleteVolume", deny(DenyReason::Mutating)),
        ("DeleteWorkspaceInstance", deny(DenyReason::Mutating)),
        ("DisassociateVolume", deny(DenyReason::Mutating)),
        ("GetWorkspaceInstance", workspaces_instances_plans_1::get_workspace_instance()?),
        ("ListInstanceTypes", deny(DenyReason::ReferenceData)),
        ("ListRegions", deny(DenyReason::ReferenceData)),
        ("ListTagsForResource", defer("Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.")),
        ("ListWorkspaceInstances", workspaces_instances_plans_1::list_workspace_instances()?),
        ("ModifyVolume", deny(DenyReason::Mutating)),
        ("TagResource", deny(DenyReason::Mutating)),
        ("UntagResource", deny(DenyReason::Mutating)),
    ])
}
