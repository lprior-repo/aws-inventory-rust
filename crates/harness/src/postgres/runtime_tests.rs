use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::{Action, Runtime, compose_at, detect_in};
use crate::HarnessError;

fn script(path: &Path, body: &str) -> Result<(), HarnessError> {
    fs::write(
        path,
        format!("#!/bin/sh\n[ -z \"$FIXTURE_PROBE\" ] || exit 0\n{body}\n"),
    )?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    for _ in 0..500 {
        match std::process::Command::new(path)
            .env("FIXTURE_PROBE", "1")
            .status()
        {
            Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            other => return other.map(|_| ()).map_err(Into::into),
        }
    }
    Err(HarnessError::invalid("fixture script stayed busy"))
}

fn stub(directory: &Path, runtime: &str, status: u8) -> Result<(), HarnessError> {
    script(&directory.join(runtime), &format!("exit {status}"))
}

#[tokio::test]
async fn postgres_runtime_prefers_podman() -> Result<(), HarnessError> {
    let directory = tempfile::tempdir()?;
    stub(directory.path(), "podman", 0)?;
    stub(directory.path(), "docker", 0)?;
    let selected = detect_in(directory.path().as_os_str(), None).await?;
    assert_eq!(selected, directory.path().join("podman"));
    Ok(())
}

#[tokio::test]
async fn postgres_runtime_falls_back_to_docker() -> Result<(), HarnessError> {
    let directory = tempfile::tempdir()?;
    stub(directory.path(), "podman", 1)?;
    stub(directory.path(), "docker", 0)?;
    let selected = detect_in(directory.path().as_os_str(), None).await?;
    assert_eq!(selected, directory.path().join("docker"));
    assert!(
        detect_in(directory.path().as_os_str(), Some(Runtime::Podman))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn postgres_runtime_missing_names_native_instructions() -> Result<(), HarnessError> {
    let directory = tempfile::tempdir()?;
    let result = detect_in(directory.path().as_os_str(), None).await;
    assert!(result.is_err_and(|error| error.to_string().contains("deploy/local/README.md")));
    Ok(())
}

#[tokio::test]
async fn postgres_runtime_explicit_docker_overrides_preference() -> Result<(), HarnessError> {
    let directory = tempfile::tempdir()?;
    stub(directory.path(), "podman", 0)?;
    stub(directory.path(), "docker", 0)?;
    let selected = detect_in(directory.path().as_os_str(), Some(Runtime::Docker)).await?;
    assert_eq!(selected, directory.path().join("docker"));
    Ok(())
}

#[tokio::test]
async fn postgres_runtime_compose_arguments_and_failures() -> Result<(), HarnessError> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("docker");
    let trace = directory.path().join("arguments");
    script(
        &executable,
        &format!("printf '%s\\n' \"$@\" > '{}'", trace.display()),
    )?;
    let file = directory.path().join("compose.yaml");
    compose_at(&executable, &file, Action::Up).await?;
    let prefix = format!(
        "compose\n--project-name\ninventory-local\n-f\n{}\n",
        file.display()
    );
    assert_eq!(
        fs::read_to_string(&trace)?,
        format!("{prefix}up\n-d\n--wait\n")
    );
    compose_at(&executable, &file, Action::DownWithVolumes).await?;
    assert_eq!(
        fs::read_to_string(&trace)?,
        format!("{prefix}down\n--volumes\n")
    );
    stub(directory.path(), "docker", 1)?;
    assert!(compose_at(&executable, &file, Action::Down).await.is_err());
    Ok(())
}
