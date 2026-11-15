// Pinned source: .local/aws-models/models/account-access-runtime/service/2018-05-10/account-access-runtime-2018-05-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("GetRoleCredentials", deferred("Model requires tenantId, roleArn, accessType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns temporary Amazon Web Services credentials for a role that you are entitled to assume within a tenant. You can use these credentials for programmatic access with the CLI or an Amazon Web Services SDK. If you aren't…")),
        ("ListRoles", deferred("Model requires tenantId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the roles that you are entitled to access within a tenant. The results are paginated. Use a returned role with GetRoleCredentials to obtain temporary credentials.")),
    ])
}
