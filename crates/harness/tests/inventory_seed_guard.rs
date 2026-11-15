use inventory_harness::inventory_seed::{SeedConfig, SeedError};
use std::path::Path;

fn ca_file() -> Result<(tempfile::TempDir, std::path::PathBuf), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("emulator-ca.pem");
    std::fs::write(
        &path,
        b"-----BEGIN CERTIFICATE-----\nZm9v\n-----END CERTIFICATE-----\n",
    )?;
    Ok((directory, path))
}

#[test]
fn guarded_config_accepts_numeric_loopback_and_explicit_ca()
-> Result<(), Box<dyn std::error::Error>> {
    let (_directory, ca) = ca_file()?;
    let config = SeedConfig::new("https://127.0.0.1:4566", &ca, "us-east-1")?;
    assert_eq!(config.region().as_str(), "us-east-1");
    Ok(())
}

#[test]
fn guarded_config_rejects_remote_and_plain_http_endpoints() -> Result<(), Box<dyn std::error::Error>>
{
    let (_directory, ca) = ca_file()?;
    assert!(matches!(
        SeedConfig::new("https://ec2.amazonaws.com:443", &ca, "us-east-1"),
        Err(SeedError::Invalid("seed endpoint must be loopback"))
    ));
    assert!(matches!(
        SeedConfig::new("http://127.0.0.1:4566", &ca, "us-east-1"),
        Err(SeedError::Invalid("seed endpoint must use HTTPS"))
    ));
    Ok(())
}

#[test]
fn missing_or_non_pem_ca_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let (_directory, ca) = ca_file()?;
    let missing = ca.with_file_name("missing.pem");
    assert!(SeedConfig::new("https://127.0.0.1:4566", &missing, "us-east-1").is_err());
    std::fs::write(&ca, b"not a certificate")?;
    assert!(matches!(
        SeedConfig::new("https://127.0.0.1:4566", &ca, "us-east-1"),
        Err(SeedError::Invalid(
            "trusted CA bundle must contain PEM certificates"
        ))
    ));
    Ok(())
}

#[test]
fn region_and_port_are_validated_before_permit_creation() -> Result<(), Box<dyn std::error::Error>>
{
    let (_directory, ca) = ca_file()?;
    assert!(SeedConfig::new("https://127.0.0.1:0", &ca, "us-east-1").is_err());
    assert!(SeedConfig::new("https://127.0.0.1:4566", &ca, "not-a-region").is_err());
    Ok(())
}

#[test]
fn ca_path_must_be_a_file() {
    let error = SeedConfig::new("https://127.0.0.1:4566", Path::new("/"), "us-east-1");
    assert!(error.is_err());
}

#[test]
fn oversized_trusted_ca_is_rejected_before_network_use() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let ca = directory.path().join("oversized.pem");
    std::fs::write(&ca, vec![b'A'; 1_048_577])?;
    assert!(SeedConfig::new("https://127.0.0.1:4566", &ca, "us-east-1").is_err());
    Ok(())
}
