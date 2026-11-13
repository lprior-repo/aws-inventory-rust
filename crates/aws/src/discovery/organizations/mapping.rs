//! mapping operations.
use super::*;

pub(super) fn map_account(account: Account) -> Result<OrganizationAccount, AwsScanError> {
    let id = required(account.id(), "organization account ID")?;
    validate_account_id(&id).map_err(|_| data_error("invalid organization account ID"))?;
    let state = optional_text(
        account.state().map(|state| state.as_str()),
        "account state",
        64,
    )?;
    let joined_at = account
        .joined_timestamp()
        .map(|time| {
            time.fmt(Format::DateTime)
                .map_err(|_| data_error("invalid organization join timestamp"))
        })
        .transpose()?;
    Ok(OrganizationAccount {
        eligible_for_scan: state.as_deref() == Some("ACTIVE"),
        id,
        arn: optional_text(account.arn(), "organization account ARN", 2_048)?,
        name: optional_text(account.name(), "organization account name", 512)?,
        state,
        joined_method: optional_text(
            account.joined_method().map(|method| method.as_str()),
            "join method",
            64,
        )?,
        joined_at,
    })
}

pub(super) fn map_root(root: Root) -> Result<OrganizationNode, AwsScanError> {
    let id = required(root.id(), "organization root ID")?;
    validate_node_id(&id, "r-")?;
    Ok(OrganizationNode {
        id,
        arn: optional_text(root.arn(), "root ARN", 2_048)?,
        name: optional_text(root.name(), "root name", 512)?,
        parent_id: None,
        depth: 0,
    })
}

pub(super) fn map_unit(
    unit: OrganizationalUnit,
    parent: &str,
    depth: u32,
) -> Result<OrganizationNode, AwsScanError> {
    let id = required(unit.id(), "organizational unit ID")?;
    validate_node_id(&id, "ou-")?;
    Ok(OrganizationNode {
        id,
        arn: optional_text(unit.arn(), "OU ARN", 2_048)?,
        name: optional_text(unit.name(), "OU name", 512)?,
        parent_id: Some(parent.to_owned()),
        depth,
    })
}

pub(super) fn map_member(child: Child) -> Result<String, AwsScanError> {
    if child.r#type().is_none_or(|kind| kind.as_str() != "ACCOUNT") {
        return Err(data_error("ListChildren returned a non-account child"));
    }
    let id = required(child.id(), "organization member ID")?;
    validate_account_id(&id).map_err(|_| data_error("invalid organization member ID"))?;
    Ok(id)
}

pub(super) fn validate_node_id(id: &str, prefix: &str) -> Result<(), AwsScanError> {
    bounded_text(id, "organization node ID", 100)?;
    let segment = |value: &str, minimum: usize| {
        (minimum..=32).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    };
    let valid = match (prefix, id.strip_prefix(prefix)) {
        ("r-", Some(root)) => segment(root, 4),
        ("ou-", Some(unit)) => unit
            .split_once('-')
            .is_some_and(|(root, suffix)| segment(root, 4) && segment(suffix, 8)),
        _ => false,
    };
    if !valid {
        return Err(data_error("invalid organization node ID"));
    }
    Ok(())
}
