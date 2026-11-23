use serde::Deserialize;
use std::process::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
}

fn workspace_packages() -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let output = Command::new("timeout")
        .args(["120", env!("CARGO"), "metadata", "--locked", "--offline"])
        .args(["--format-version", "1", "--all-features"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    if output.stdout.len() > 33_554_432 {
        return Err("cargo metadata output exceeds 32 MiB".into());
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata.packages)
}

#[test]
fn given_workspace_when_postgres_resolves_then_all_sqlx_crates_share_version_and_source()
-> TestResult {
    let packages = workspace_packages()?;
    let core = packages
        .iter()
        .find(|package| package.name == "sqlx-core")
        .ok_or("sqlx-core is missing")?;
    assert!(
        core.source.is_some(),
        "SQLx must retain its upstream source"
    );
    let postgres = packages
        .iter()
        .filter(|package| package.name == "sqlx-postgres")
        .count();
    assert_eq!(postgres, 1, "exactly one PostgreSQL driver must resolve");
    assert_matching_sqlx(&packages, core)?;
    Ok(())
}

fn assert_matching_sqlx(packages: &[Package], core: &Package) -> TestResult {
    packages
        .iter()
        .filter(|package| package.name == "sqlx" || package.name.starts_with("sqlx-"))
        .try_for_each(|package| {
            assert_eq!(
                package.version, core.version,
                "version mismatch: {package:?}"
            );
            assert_eq!(package.source, core.source, "source mismatch: {package:?}");
            Ok::<(), Box<dyn std::error::Error>>(())
        })
}
