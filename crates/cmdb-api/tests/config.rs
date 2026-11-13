//! CLI parsing and the local-only bind policy.

use cmdb_api::{Config, ConfigError, DEFAULT_BIND};
use std::net::SocketAddr;
use std::path::PathBuf;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn defaults_to_loopback_and_requires_export() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse(args(&["--export", "inventory.json"]))?;
    assert_eq!(config.export, PathBuf::from("inventory.json"));
    assert_eq!(config.bind, DEFAULT_BIND);
    assert!(config.bind.ip().is_loopback());
    Ok(())
}

#[test]
fn accepts_explicit_ipv6_loopback() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse(args(&[
        "--bind",
        "[::1]:8787",
        "--export",
        "inventory.json",
    ]))?;
    let expected: SocketAddr = "[::1]:8787".parse()?;
    assert_eq!(config.bind, expected);
    Ok(())
}

#[test]
fn rejects_non_local_binds_without_auth() {
    for bind in ["0.0.0.0:8787", "192.168.1.10:8787", "[::]:8787"] {
        let result = Config::parse(args(&["--export", "inventory.json", "--bind", bind]));
        assert!(
            matches!(result, Err(ConfigError::NonLocalBind(_))),
            "{bind} must be refused"
        );
    }
}

#[test]
fn rejects_missing_export_and_malformed_arguments() {
    assert!(matches!(
        Config::parse(args(&[])),
        Err(ConfigError::MissingExport)
    ));
    assert!(matches!(
        Config::parse(args(&["--export"])),
        Err(ConfigError::MissingValue("--export"))
    ));
    assert!(matches!(
        Config::parse(args(&["--export", "x", "--wat"])),
        Err(ConfigError::UnknownArgument(_))
    ));
    assert!(matches!(
        Config::parse(args(&["--export", "x", "--bind", "localhost:8787"])),
        Err(ConfigError::InvalidBind { .. })
    ));
}
