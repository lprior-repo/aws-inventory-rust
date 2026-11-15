// Pinned source: .local/aws-models/models/controlcatalog/service/2018-05-10/controlcatalog-2018-05-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("GetControl", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns details about a specific control, most notably a list of Amazon Web Services Regions where this control is supported. Input a value for the ControlArn parameter, in ARN form. GetControl accepts controltower or…")),
        ("ListCommonControls", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns a paginated list of common controls from the Amazon Web Services Control Catalog. You can apply an optional filter to see common controls that have a specific objective. If you don’t provide a filter, the operation…")),
        ("ListControlMappings", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns a paginated list of control mappings from the Control Catalog. Control mappings show relationships between controls and other entities, such as common controls or compliance frameworks.")),
        ("ListControls", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns a paginated list of all available controls in the Control Catalog library. Allows you to discover available controls. The list of controls is given as structures of type controlSummary . The ARN is returned in the global…")),
        ("ListDomains", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns a paginated list of domains from the Control Catalog.")),
        ("ListObjectives", deferred("Public control-catalog reference definitions, not account-owned deployed resources. Model: Returns a paginated list of objectives from the Control Catalog. You can apply an optional filter to see the objectives that belong to a specific domain. If you don’t provide a filter, the operation returns all objectives.")),
    ])
}
