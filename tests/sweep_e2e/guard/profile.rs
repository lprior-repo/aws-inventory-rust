use anyhow::{Result, bail, ensure};
use std::collections::BTreeMap;

type Profile<'a> = BTreeMap<&'a str, &'a str>;

pub(super) fn validate(
    credentials: &str,
    config: &str,
    name: &str,
    endpoint: &str,
) -> Result<bool> {
    let Some(credentials) = parse(credentials, name)? else {
        return Ok(false);
    };
    let config = parse(config, name)?.unwrap_or_default();
    let credentials_safe = credentials.keys().all(|key| {
        matches!(
            *key,
            "aws_access_key_id" | "aws_secret_access_key" | "aws_session_token"
        )
    });
    let config_safe = config.iter().all(|(key, value)| match *key {
        "region" | "output" => true,
        "endpoint_url" => *value == endpoint,
        _ => false,
    });
    Ok(credentials_safe
        && config_safe
        && credentials
            .get("aws_access_key_id")
            .is_some_and(|value| !value.is_empty())
        && credentials
            .get("aws_secret_access_key")
            .is_some_and(|value| !value.is_empty()))
}

fn parse<'a>(contents: &'a str, selected: &str) -> Result<Option<Profile<'a>>> {
    let mut profile = None;
    let mut active = false;
    for line in contents.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if line.starts_with('[') {
            active = section(line)? == selected;
            if active {
                ensure!(profile.is_none(), "duplicate selected AWS profile section");
                profile = Some(BTreeMap::new());
            }
        } else if active {
            let fields = profile
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("missing profile section"))?;
            insert(fields, line)?;
        }
    }
    Ok(profile)
}

fn section(line: &str) -> Result<&str> {
    let name = line
        .strip_prefix('[')
        .and_then(|line| line.strip_suffix(']'))
        .ok_or_else(|| anyhow::anyhow!("malformed AWS profile section"))?
        .trim();
    let name = match name.strip_prefix("profile") {
        Some(rest) if rest.starts_with(char::is_whitespace) => rest.trim(),
        _ => name,
    };
    ensure!(!name.is_empty(), "empty AWS profile section");
    Ok(name)
}

fn insert<'a>(fields: &mut Profile<'a>, line: &'a str) -> Result<()> {
    let Some((key, value)) = line.split_once('=') else {
        bail!("malformed AWS profile property");
    };
    let key = key.trim();
    ensure!(
        !key.is_empty()
            && key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
        "unsupported AWS profile property syntax"
    );
    ensure!(
        fields.insert(key, value.trim()).is_none(),
        "duplicate AWS profile property"
    );
    Ok(())
}

#[cfg(test)]
#[path = "profile/tests.rs"]
mod tests;
