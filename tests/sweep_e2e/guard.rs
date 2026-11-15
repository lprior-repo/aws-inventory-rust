use anyhow::{Context, Result, ensure};
use std::path::PathBuf;
#[path = "guard/profile.rs"]
mod profile;

#[derive(Clone, Debug)]
pub(super) struct Environment {
    pub(super) endpoint: String,
    pub(super) ca_bundle: PathBuf,
    pub(super) graph_url: String,
    pub(super) graph_name: String,
    pub(super) region: String,
    pub(super) profile: String,
    pub(super) catalog: PathBuf,
    pub(super) aws_config: PathBuf,
    pub(super) aws_credentials: PathBuf,
}

impl Environment {
    pub(super) fn load() -> Result<Self> {
        let environment = Self {
            endpoint: required("AWS_INVENTORY_E2E_ENDPOINT")?,
            ca_bundle: required_path("AWS_INVENTORY_E2E_CA_BUNDLE")?,
            graph_url: required("AWS_INVENTORY_E2E_GRAPH_URL")?,
            graph_name: required("AWS_INVENTORY_E2E_GRAPH_NAME")?,
            region: required("AWS_INVENTORY_E2E_REGION")?,
            profile: required("AWS_INVENTORY_E2E_PROFILE")?,
            catalog: required_path("AWS_INVENTORY_E2E_CATALOG")?,
            aws_config: required_path("AWS_INVENTORY_E2E_AWS_CONFIG")?,
            aws_credentials: required_path("AWS_INVENTORY_E2E_AWS_CREDENTIALS")?,
        };
        environment.validate()?;
        Ok(environment)
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            loopback_https(&self.endpoint),
            "emulator endpoint must be HTTPS on literal loopback"
        );
        ensure!(
            loopback_redis(&self.graph_url),
            "Falkor URL must use loopback Redis transport"
        );
        ensure!(
            self.ca_bundle.is_file(),
            "trusted CA bundle must be an existing file"
        );
        ensure!(
            self.catalog.is_file(),
            "compiled catalog must be an existing file"
        );
        ensure!(
            self.aws_config.is_file(),
            "isolated AWS config file must exist"
        );
        ensure!(
            self.aws_credentials.is_file(),
            "isolated AWS credentials file must exist"
        );
        ensure!(
            !self.graph_name.is_empty()
                && self.graph_name.len() <= 90
                && self
                    .graph_name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'),
            "graph name must be a short ASCII identifier safe for a unique temporary suffix"
        );
        ensure!(!self.region.trim().is_empty(), "region must not be empty");
        ensure!(!self.profile.trim().is_empty(), "profile must not be empty");
        ensure!(
            configured_file("AWS_CONFIG_FILE", &self.aws_config),
            "AWS_CONFIG_FILE must select the isolated config"
        );
        ensure!(
            configured_file("AWS_SHARED_CREDENTIALS_FILE", &self.aws_credentials),
            "AWS_SHARED_CREDENTIALS_FILE must select isolated credentials"
        );
        ensure!(
            static_profile_credentials(self)?,
            "E2E profile must use isolated static credentials, not a role or credential process"
        );
        ensure!(
            no_ambient_credentials(),
            "ambient AWS credential variables are forbidden for emulator E2E"
        );
        Ok(())
    }
}

fn configured_file(name: &str, path: &std::path::Path) -> bool {
    std::env::var_os(name).as_deref() == Some(path.as_os_str())
}

fn no_ambient_credentials() -> bool {
    [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_PROFILE",
        "AWS_DEFAULT_PROFILE",
        "AWS_WEB_IDENTITY_TOKEN_FILE",
        "AWS_ROLE_ARN",
        "AWS_CONTAINER_CREDENTIALS_RELATIVE_URI",
        "AWS_CONTAINER_CREDENTIALS_FULL_URI",
    ]
    .iter()
    .all(|name| std::env::var_os(name).is_none())
}
fn static_profile_credentials(environment: &Environment) -> Result<bool> {
    let credentials = std::fs::read_to_string(&environment.aws_credentials)?;
    let config = std::fs::read_to_string(&environment.aws_config)?;
    profile::validate(
        &credentials,
        &config,
        &environment.profile,
        &environment.endpoint,
    )
}

fn required(name: &str) -> Result<String> {
    std::env::var(name)
        .with_context(|| format!("required safeguard environment variable {name} is missing"))
}

fn required_path(name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(required(name)?))
}

fn loopback_https(endpoint: &str) -> bool {
    ["https://127.0.0.1:", "https://localhost:"]
        .iter()
        .any(|prefix| endpoint.strip_prefix(prefix).is_some_and(valid_port))
}

fn loopback_redis(endpoint: &str) -> bool {
    [
        "redis://127.0.0.1:",
        "redis://localhost:",
        "rediss://127.0.0.1:",
        "rediss://localhost:",
    ]
    .iter()
    .any(|prefix| {
        endpoint
            .strip_prefix(prefix)
            .is_some_and(valid_port_and_suffix)
    })
}

fn valid_port(value: &str) -> bool {
    value.parse::<u16>().is_ok_and(|port| port > 0)
}

fn valid_port_and_suffix(value: &str) -> bool {
    let (port, suffix) = value
        .split_once('/')
        .map_or((value, ""), |(port, suffix)| (port, suffix));
    valid_port(port) && !suffix.contains('@') && !suffix.contains('#') && !suffix.contains('?')
}
#[cfg(test)]
mod tests {
    use super::{loopback_https, loopback_redis};

    #[test]
    fn endpoints_must_be_loopback_and_transport_secure() {
        assert!(loopback_https("https://127.0.0.1:4567"));
        assert!(loopback_https("https://localhost:4567"));
        assert!(!loopback_https("http://127.0.0.1:4567"));
        assert!(!loopback_https("https://example.com:4567"));
        assert!(!loopback_https("https://127.0.0.1:0"));
        assert!(loopback_redis("redis://127.0.0.1:6379"));
        assert!(loopback_redis("rediss://localhost:6379/"));
        assert!(!loopback_redis("redis://example.com:6379"));
        assert!(!loopback_redis("redis://127.0.0.1:6379?password=x"));
    }
}
