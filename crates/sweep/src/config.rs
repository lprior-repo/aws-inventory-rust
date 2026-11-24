use std::net::{Ipv4Addr, Ipv6Addr};

use super::{SweepConfig, SweepError};

const MAX_PAGES: u32 = 1_000_000;
const MAX_CONCURRENCY: usize = inventory_engine::limiter::MAX_CONCURRENCY;

impl SweepConfig {
    pub(super) fn validate(&self) -> Result<(), SweepError> {
        if !(1..=MAX_PAGES).contains(&self.max_pages) {
            return Err(SweepError::InvalidConfig(
                "max-pages must be in 1..=1000000",
            ));
        }
        if !(1..=MAX_CONCURRENCY).contains(&self.concurrency) {
            return Err(SweepError::InvalidConfig("concurrency must be in 1..=4096"));
        }
        override_uri(self.endpoint.as_deref(), self.ca_bundle.as_deref())?;
        Ok(())
    }

    pub(super) fn endpoint_override(&self) -> Result<Option<&str>, SweepError> {
        override_uri(self.endpoint.as_deref(), self.ca_bundle.as_deref())
    }
}

fn override_uri<'a>(
    endpoint: Option<&'a str>,
    ca_bundle: Option<&std::path::Path>,
) -> Result<Option<&'a str>, SweepError> {
    match (endpoint, ca_bundle) {
        (None, None) => Ok(None),
        (Some(value), Some(path)) => {
            validate_override(value, path)?;
            Ok(Some(value))
        }
        _ => Err(SweepError::InvalidConfig(
            "endpoint overrides require both --endpoint and --ca-bundle",
        )),
    }
}

fn validate_override(value: &str, ca_bundle: &std::path::Path) -> Result<(), SweepError> {
    let uri = value
        .parse::<http::Uri>()
        .map_err(|_| SweepError::InvalidConfig("endpoint is not a valid URI"))?;
    let authority = uri
        .authority()
        .ok_or(SweepError::InvalidConfig("endpoint authority is required"))?;
    if uri.scheme_str() != Some("https")
        || authority.as_str().contains('@')
        || !loopback(authority.host())
    {
        return Err(SweepError::InvalidConfig(
            "endpoint override must be HTTPS on a loopback host",
        ));
    }
    validate_override_bundle(&uri, ca_bundle)?;
    Ok(())
}

fn validate_override_bundle(
    uri: &http::Uri,
    ca_bundle: &std::path::Path,
) -> Result<(), SweepError> {
    if uri
        .path_and_query()
        .is_some_and(|path| path.as_str().contains('?'))
        || !ca_bundle.is_file()
    {
        return Err(SweepError::InvalidConfig(
            "endpoint override requires a CA bundle file and no query",
        ));
    }
    Ok(())
}

fn loopback(host: &str) -> bool {
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<Ipv4Addr>()
            .is_ok_and(|address| address.is_loopback())
        || host
            .parse::<Ipv6Addr>()
            .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracketed_ipv6_loopback_is_allowed() {
        assert!(loopback("[::1]"));
        assert!(!loopback("[2001:db8::1]"));
    }
}
