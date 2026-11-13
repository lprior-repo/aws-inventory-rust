//! Fetch the pinned emulator source clone and verify/extract its signed binary.
//!
//! No system install and no service changes: everything lands inside the
//! operator-supplied destination. Verification happens before extraction, and
//! the signer is taken only from machine-readable `VALIDSIG` status lines.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::HarnessError;
use crate::provenance::file_sha256;

mod signature;

pub use signature::signed_by;

/// Pinned upstream identities and artefact URLs.
struct Metadata {
    repository: String,
    commit: String,
    package_url: String,
    package_sha256: String,
    signing_key_url: String,
    signing_fingerprint: String,
}

impl Metadata {
    fn read(path: &Path) -> Result<Self, HarnessError> {
        let (document, _) = crate::snapshot::read_json(path)?;
        let text = |key: &str| -> Result<String, HarnessError> {
            document
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| HarnessError::invalid(format!("upstream.json lacks {key}")))
        };
        Ok(Self {
            repository: text("repository")?,
            commit: text("commit")?,
            package_url: text("package_url")?,
            package_sha256: text("package_sha256")?,
            signing_key_url: text("package_signing_key_url")?,
            signing_fingerprint: text("package_signing_key_fingerprint")?,
        })
    }
}

fn require_platform() -> Result<(), HarnessError> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Ok(());
    }
    Err(HarnessError::invalid(
        "This pinned package is Linux amd64; see upstream docs for other platforms.",
    ))
}

fn ensure_destination(destination: &Path) -> Result<PathBuf, HarnessError> {
    std::fs::canonicalize(destination)
        .or_else(|_| {
            std::fs::create_dir_all(destination)?;
            std::fs::canonicalize(destination)
        })
        .map_err(HarnessError::Io)
}

/// Fetch, verify and extract the pinned emulator into `destination`.
pub fn install(root: &Path, destination: &Path) -> Result<Value, HarnessError> {
    require_platform()?;
    let metadata = Metadata::read(&root.join("scripts/emulator/upstream.json"))?;
    let destination = ensure_destination(destination)?;
    let source = checkout(&metadata, &destination)?;
    let package = download(&metadata, &destination)?;
    signature::verify_signature(&metadata, &destination, &package)?;
    let unpacked = destination.join("unpacked");
    extract(&package, &unpacked)?;
    let binary = unpacked.join("opt/aws-emulator/bin/aws-emulator");
    run(&[path_text(&binary)?, "--version".to_owned()])?;
    Ok(json!({
        "source": path_text(&source)?,
        "binary": path_text(&binary)?,
        "verified_sha256": metadata.package_sha256,
    }))
}

fn clone_source(metadata: &Metadata, source: &Path) -> Result<(), HarnessError> {
    run(&[
        "git".to_owned(),
        "clone".to_owned(),
        metadata.repository.clone(),
        path_text(source)?,
    ])?;
    run(&[
        "git".to_owned(),
        "-C".to_owned(),
        path_text(source)?,
        "checkout".to_owned(),
        "--detach".to_owned(),
        metadata.commit.clone(),
    ])
}

fn checkout(metadata: &Metadata, destination: &Path) -> Result<PathBuf, HarnessError> {
    let source = destination.join("aws-emulator");
    if !source.exists() {
        clone_source(metadata, &source)?;
    }
    let actual = output(&["git", "-C", &path_text(&source)?, "rev-parse", "HEAD"])?;
    let shallow = output(&[
        "git",
        "-C",
        &path_text(&source)?,
        "rev-parse",
        "--is-shallow-repository",
    ])?;
    if actual != metadata.commit || shallow != "false" {
        return Err(HarnessError::invalid(
            "Existing source does not match the pinned full clone; choose a new destination.",
        ));
    }
    Ok(source)
}

fn download(metadata: &Metadata, destination: &Path) -> Result<PathBuf, HarnessError> {
    let package = destination.join("aws-emulator_amd64.deb");
    fetch(&metadata.package_url, &package)?;
    if file_sha256(&package)? != metadata.package_sha256 {
        return Err(HarnessError::invalid(
            "AWS Emulator package checksum mismatch",
        ));
    }
    fetch(
        &format!("{}.sig", metadata.package_url),
        &destination.join("aws-emulator_amd64.deb.sig"),
    )?;
    fetch(
        &metadata.signing_key_url,
        &destination.join("aws-publishing.asc"),
    )?;
    Ok(package)
}

fn extract_dpkg(package: &Path, destination: &Path) -> Result<(), HarnessError> {
    run(&[
        "dpkg-deb".to_owned(),
        "--extract".to_owned(),
        path_text(package)?,
        path_text(destination)?,
    ])
}

fn tar_payload(package: &Path, payload: &Path) -> Result<(), HarnessError> {
    let extracted = Command::new("bsdtar")
        .args(["-xOf", &path_text(package)?, "data.tar.xz"])
        .output()?;
    if !extracted.status.success() {
        return Err(HarnessError::invalid(
            "bsdtar could not read the package payload",
        ));
    }
    std::fs::write(payload, extracted.stdout)?;
    Ok(())
}

fn extract(package: &Path, destination: &Path) -> Result<(), HarnessError> {
    if which("dpkg-deb") {
        return extract_dpkg(package, destination);
    }
    if !which("bsdtar") {
        return Err(HarnessError::invalid(
            "Install dpkg-deb or bsdtar to extract the verified emulator package",
        ));
    }
    std::fs::create_dir_all(destination)?;
    let payload = destination
        .parent()
        .unwrap_or(destination)
        .join("data.tar.xz");
    tar_payload(package, &payload)?;
    let extracted = run(&[
        "bsdtar".to_owned(),
        "-xf".to_owned(),
        path_text(&payload)?,
        "-C".to_owned(),
        path_text(destination)?,
    ]);
    let _ = std::fs::remove_file(&payload);
    extracted
}

fn curl_command(path: &Path, url: &str) -> Command {
    let mut command = Command::new("curl");
    command
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "300",
            "--output",
        ])
        .arg(path)
        .arg(url);
    command
}

fn fetch(url: &str, path: &Path) -> Result<(), HarnessError> {
    let mut command = curl_command(path, url);
    // A shell scoped to an emulator exports that emulator's CA bundle, which
    // cannot validate the vendor host. The package is downloaded against the
    // system trust store; its own signature is verified separately.
    for name in [
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "CURL_CA_BUNDLE",
        "AWS_CA_BUNDLE",
    ] {
        command.env_remove(name);
    }
    let status = command.status()?;
    if !status.success() {
        return Err(HarnessError::invalid("curl failed"));
    }
    Ok(())
}

fn which(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}

fn run(arguments: &[String]) -> Result<(), HarnessError> {
    let (program, rest) = arguments
        .split_first()
        .ok_or_else(|| HarnessError::invalid("empty command"))?;
    let status = Command::new(program).args(rest).status()?;
    if !status.success() {
        return Err(HarnessError::invalid(format!("{program} failed")));
    }
    Ok(())
}

fn run_capture(arguments: &[String]) -> Result<Vec<u8>, HarnessError> {
    let (program, rest) = arguments
        .split_first()
        .ok_or_else(|| HarnessError::invalid("empty command"))?;
    let output = Command::new(program).args(rest).output()?;
    if !output.status.success() {
        return Err(HarnessError::invalid(format!("{program} failed")));
    }
    Ok(output.stdout)
}

fn output(arguments: &[&str]) -> Result<String, HarnessError> {
    output_owned(
        &arguments
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
}

fn output_owned(arguments: &[String]) -> Result<String, HarnessError> {
    let (program, rest) = arguments
        .split_first()
        .ok_or_else(|| HarnessError::invalid("empty command"))?;
    let output = Command::new(program).args(rest).output()?;
    if !output.status.success() {
        return Err(HarnessError::invalid(format!("{program} failed")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn path_text(path: &Path) -> Result<String, HarnessError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| HarnessError::invalid("path is not valid UTF-8"))
}
