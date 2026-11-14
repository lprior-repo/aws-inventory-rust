use crate::cli::AccountArgs;
use anyhow::{Result, ensure};

pub const MAX_TARGET_ACCOUNTS: usize = 1_024;

pub fn validate(options: &AccountArgs, profiles: &[String]) -> Result<()> {
    ensure!(
        (1..=MAX_TARGET_ACCOUNTS).contains(&options.max_accounts),
        "max-accounts must be 1..1024"
    );
    ensure!(
        options.account_id.len() <= MAX_TARGET_ACCOUNTS,
        "too many account filters"
    );
    ensure!(
        !(options.organization_role.is_some() && options.role_arn.is_some()),
        "choose organization-role or role-arn"
    );
    ensure!(
        options.organization_role.is_some() || options.account_id.is_empty(),
        "account-id filters require organization-role"
    );
    ensure!(
        options.organization_role.is_none() || profiles.len() <= 1,
        "organization discovery accepts one management/delegated profile"
    );
    ensure!(
        options.role_arn.is_none() || profiles.len() <= 1,
        "an explicit role ARN accepts one source profile"
    );
    ensure!(
        options.external_id_env.is_none()
            || options.organization_role.is_some()
            || options.role_arn.is_some(),
        "external-id-env requires a role target"
    );
    for id in &options.account_id {
        validate_account(id)?;
    }
    if let Some(role) = &options.organization_role {
        validate_role_path(role)?;
    }
    if let Some(arn) = &options.role_arn {
        validate_role_arn(arn)?;
    }
    if let Some(name) = &options.external_id_env {
        validate_env(name)?;
    }
    ensure!(
        (900..=3600).contains(&options.role_duration_seconds),
        "role duration must be 900..3600 seconds for source sessions that may already assume a role"
    );
    ensure!(
        (2..=64).contains(&options.role_session_name.len())
            && options.role_session_name.bytes().all(role_char),
        "invalid role session name"
    );
    Ok(())
}

pub fn validate_account(id: &str) -> Result<()> {
    ensure!(
        id.len() == 12 && id.bytes().all(|b| b.is_ascii_digit()),
        "account IDs must have exactly 12 ASCII digits"
    );
    Ok(())
}

fn role_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"+=,.@_-".contains(&b)
}

fn validate_role_path(role: &str) -> Result<()> {
    ensure!(
        !role.is_empty()
            && role.len() <= 575
            && role.split('/').all(|part| !part.is_empty()
                && !matches!(part, "." | "..")
                && part.bytes().all(role_char))
            && role.rsplit('/').next().is_some_and(|name| name.len() <= 64)
            && role
                .rsplit_once('/')
                .is_none_or(|(prefix, _)| prefix.len().saturating_add(2) <= 512),
        "invalid role name/path"
    );
    Ok(())
}

fn validate_role_arn(arn: &str) -> Result<()> {
    ensure!(arn.len() <= 2048, "role ARN exceeds bound");
    let mut parts = arn.splitn(6, ':');
    ensure!(parts.next() == Some("arn"), "invalid role ARN prefix");
    let partition = parts.next().unwrap_or_default();
    ensure!(
        (3..=32).contains(&partition.len())
            && partition.split('-').enumerate().all(|(position, segment)| {
                if position == 0 {
                    segment == "aws"
                } else {
                    !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_lowercase())
                }
            }),
        "invalid ARN partition"
    );
    ensure!(
        parts.next() == Some("iam") && parts.next() == Some(""),
        "role ARN must identify IAM with an empty region"
    );
    validate_account(parts.next().unwrap_or_default())?;
    let resource = parts.next().unwrap_or_default();
    let role = resource
        .strip_prefix("role/")
        .ok_or_else(|| anyhow::anyhow!("IAM ARN must identify a role"))?;
    validate_role_path(role)
}

fn validate_env(name: &str) -> Result<()> {
    let mut bytes = name.bytes();
    ensure!(
        name.len() <= 256
            && bytes
                .next()
                .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_'),
        "invalid external-ID environment variable name"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn role_targets_validate_without_environment_or_credentials() -> Result<()> {
        let mut target = AccountArgs::default();
        validate(&target, &[])?;
        target.organization_role = Some("audit/InventoryReader".into());
        target.account_id = vec!["123456789012".into()];
        target.external_id_env = Some("INVENTORY_EXTERNAL_ID".into());
        validate(&target, &["management".into()])?;
        assert!(validate(&target, &["one".into(), "two".into()]).is_err());
        target.role_arn = Some("arn:aws:iam::123456789012:role/reader".into());
        assert!(validate(&target, &[]).is_err());
        target.organization_role = None;
        target.account_id.clear();
        validate(&target, &[])?;
        Ok(())
    }
    #[test]
    fn rejects_invalid_accounts_roles_and_secret_environment_names() {
        for arn in [
            "arn:aws:sts::123456789012:role/x",
            "arn:aws:iam:us-east-1:123456789012:role/x",
            "arn:aws:iam::123:role/x",
            "arn:aws:iam::123456789012:user/x",
            "arn:aws:iam::123456789012:role/a//b",
        ] {
            assert!(validate_role_arn(arn).is_err(), "{arn}");
        }
        assert!(validate_account("１２３４５６７８９０１２").is_err());
        assert!(validate_env("BAD;echo").is_err());
        assert!(validate_env("2BAD").is_err());
        let target = AccountArgs {
            account_id: vec!["123456789012".into()],
            ..Default::default()
        };
        assert!(validate(&target, &[]).is_err());
    }
}
