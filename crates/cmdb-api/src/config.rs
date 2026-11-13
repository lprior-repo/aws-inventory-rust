//! Command-line configuration and the local-only bind policy.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use thiserror::Error;

/// Loopback default: the API has no authentication.
pub const DEFAULT_BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8787);

/// Usage line printed for `--help` and refused arguments.
pub const USAGE: &str = "usage: cmdb-api --export <inventory.json> [--bind 127.0.0.1:8787]\n";

/// Resolved startup configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Export document loaded once at startup.
    pub export: PathBuf,
    /// Loopback socket the server listens on.
    pub bind: SocketAddr,
}

impl Config {
    /// Parse `--export` and `--bind` (both value-taking, order-independent).
    pub fn parse<I>(args: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut export = None;
        let mut bind = DEFAULT_BIND;
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--export" => export = Some(PathBuf::from(next_value(&mut args, "--export")?)),
                "--bind" => bind = parse_bind(&next_value(&mut args, "--bind")?)?,
                "-h" | "--help" => return Err(ConfigError::Help),
                other => return Err(ConfigError::UnknownArgument(other.to_owned())),
            }
        }
        let export = export.ok_or(ConfigError::MissingExport)?;
        Ok(Self { export, bind })
    }
}

/// Startup configuration failures, including the help request.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("--export <inventory.json> is required")]
    MissingExport,
    #[error("{0} requires a value")]
    MissingValue(&'static str),
    #[error("unknown argument {0:?}; expected --export and --bind")]
    UnknownArgument(String),
    #[error("{value:?} is not a valid socket address")]
    InvalidBind { value: String },
    #[error("non-local bind {0} is unsupported without authentication; use 127.0.0.1 or ::1")]
    NonLocalBind(SocketAddr),
    #[error("help requested")]
    Help,
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    args.next().ok_or(ConfigError::MissingValue(flag))
}

fn parse_bind(value: &str) -> Result<SocketAddr, ConfigError> {
    let address = value
        .parse::<SocketAddr>()
        .map_err(|_| ConfigError::InvalidBind {
            value: value.to_owned(),
        })?;
    if !address.ip().is_loopback() {
        return Err(ConfigError::NonLocalBind(address));
    }
    Ok(address)
}
