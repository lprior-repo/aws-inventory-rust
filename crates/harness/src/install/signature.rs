use std::path::Path;

use super::{Metadata, output_owned, path_text, run, run_capture};
use crate::HarnessError;

fn gpg(keyring: &Path, arguments: &[&str]) -> Vec<String> {
    let mut command = vec![
        "gpg".to_owned(),
        "--homedir".to_owned(),
        path_text(keyring).unwrap_or_default(),
        "--no-autostart".to_owned(),
        "--batch".to_owned(),
    ];
    command.extend(arguments.iter().map(|value| (*value).to_owned()));
    command
}

fn import_keys(keyring: &Path, destination: &Path) -> Result<String, HarnessError> {
    let public_keys = keyring.join("trusted.gpg");
    let keys = path_text(&public_keys)?;
    run(&gpg(
        keyring,
        &[
            "--yes",
            "--dearmor",
            "--output",
            &keys,
            &path_text(&destination.join("aws-publishing.asc"))?,
        ],
    ))?;
    Ok(keys)
}

pub(super) fn verify_signature(
    metadata: &Metadata,
    destination: &Path,
    package: &Path,
) -> Result<(), HarnessError> {
    let keyring = destination.join("keyring");
    std::fs::create_dir_all(&keyring)?;
    let public_keys = import_keys(&keyring, destination)?;
    check_fingerprint(metadata, &keyring, &public_keys)?;
    let verified = verify_status(
        &metadata.signature_file(destination),
        package,
        &keyring,
        &public_keys,
    )?;
    if !signed_by(&verified, &metadata.signing_fingerprint) {
        return Err(HarnessError::invalid(
            "Package was not signed by the pinned AWS primary key",
        ));
    }
    Ok(())
}

fn check_fingerprint(
    metadata: &Metadata,
    keyring: &Path,
    public_keys: &str,
) -> Result<(), HarnessError> {
    let listed = output_owned(&gpg(
        keyring,
        &["--with-colons", "--show-keys", public_keys],
    ))?;
    if !listed.contains(&metadata.signing_fingerprint) {
        return Err(HarnessError::invalid(
            "AWS package signing key fingerprint mismatch",
        ));
    }
    Ok(())
}

fn verify_status(
    signature: &Path,
    package: &Path,
    keyring: &Path,
    public_keys: &str,
) -> Result<Vec<u8>, HarnessError> {
    run_capture(&gpg(
        keyring,
        &[
            "--no-default-keyring",
            "--keyring",
            public_keys,
            "--status-fd",
            "1",
            "--verify",
            &path_text(signature)?,
            &path_text(package)?,
        ],
    ))
}

pub fn signed_by(status_output: &[u8], fingerprint: &str) -> bool {
    for line in status_output.split(|byte| *byte == b'\n') {
        let fields: Vec<&[u8]> = line
            .split(|byte| *byte == b' ')
            .filter(|field| !field.is_empty())
            .collect();
        let prefix = b"[GNUPG:] VALIDSIG ";
        if !line.starts_with(prefix) || fields.len() < 3 {
            continue;
        }
        let matches = |field: Option<&&[u8]>| {
            field
                .and_then(|value| std::str::from_utf8(value).ok())
                .is_some_and(|value| value == fingerprint)
        };
        let signer = matches(fields.get(2));
        let trailing = matches(fields.last());
        if signer || trailing {
            return true;
        }
    }
    false
}
