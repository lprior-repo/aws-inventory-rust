use super::*;
use crate::overlay::FixedValue;

pub(super) fn apply(service: &mut ServiceOverlay) -> Result<(), NameError> {
    addins_plan(service)?;
    sessions_plan(service)?;
    association_plans(service)
}

fn addins_plan(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut addins = child(
        "AddIns",
        correlated(&["ResourceId"], &["AddInId"])?,
        "DescribeWorkspaceDirectories",
        "ResourceId",
        "DirectoryId",
        true,
    )?;
    addins
        .fixed
        .insert(path("MaxResults")?, FixedValue::Integer(25));
    insert(service, "DescribeConnectClientAddIns", addins)
}

fn sessions_plan(service: &mut ServiceOverlay) -> Result<(), NameError> {
    let mut sessions = child(
        "Sessions",
        correlated(&["PoolId"], &["SessionId"])?,
        "DescribeWorkspacesPools",
        "PoolId",
        "PoolId",
        true,
    )?;
    // Neither operation has a pagination trait. Both document NextToken explicitly.
    sessions
        .fixed
        .insert(path("Limit")?, FixedValue::Integer(50));
    insert(service, "DescribeWorkspacesPoolSessions", sessions)
}

fn association_plans(service: &mut ServiceOverlay) -> Result<(), NameError> {
    [
        (
            "DescribeApplicationAssociations",
            "DescribeApplications",
            "ApplicationId",
            &["WORKSPACE", "BUNDLE", "IMAGE"][..],
        ),
        (
            "DescribeBundleAssociations",
            "DescribeWorkspaceBundles",
            "BundleId",
            &["APPLICATION"][..],
        ),
        (
            "DescribeImageAssociations",
            "DescribeWorkspaceImages",
            "ImageId",
            &["APPLICATION"][..],
        ),
        (
            "DescribeWorkspaceAssociations",
            "DescribeWorkspaces",
            "WorkspaceId",
            &["APPLICATION"][..],
        ),
    ]
    .into_iter()
    .try_for_each(|(operation, parent, id, selectors)| {
        let mut plan = child(
            "Associations",
            correlated(&[id], &["AssociatedResourceId", "AssociatedResourceType"])?,
            parent,
            id,
            id,
            operation == "DescribeApplicationAssociations",
        )?;
        plan.fixed.insert(
            path("AssociatedResourceTypes")?,
            FixedValue::Strings(selectors.iter().map(|value| (*value).to_owned()).collect()),
        );
        insert(service, operation, plan)
    })
}
