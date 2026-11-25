use std::fs::{self, File};
use std::os::unix::fs::{FileExt, MetadataExt, PermissionsExt};

use super::{attest, digest_file, same_binary, snapshot, verify_binary_digest};
use crate::HarnessError;
use crate::install::graph::installer_tests::{
    CLIENT_VERSION, SERVER_VERSION, TestResult, fixture, value,
};
use crate::install::graph::{OwnedDirectory, executable, open_regular};

fn observed_version(file: &File, server: bool) -> Result<String, HarnessError> {
    if server {
        let mut bytes = [0_u8; 3];
        file.read_exact_at(&mut bytes, 0)?;
        assert_eq!(&bytes, b"abc");
        Ok(SERVER_VERSION.to_owned())
    } else {
        let mut bytes = [0_u8; 6];
        file.read_exact_at(&mut bytes, 0)?;
        assert_eq!(&bytes, b"client");
        Ok(CLIENT_VERSION.to_owned())
    }
}

#[test]
fn given_retained_binaries_when_attesting_then_versions_and_hashes_describe_same_bytes()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let expected = value(&layout)?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let mut probes = 0_u32;
    let attestation = attest(&source, |file| {
        probes = probes.saturating_add(1);
        observed_version(file, probes == 1)
    })?;
    assert_eq!(probes, 2);
    assert_eq!(attestation.server_version, SERVER_VERSION);
    assert_eq!(attestation.client_version, CLIENT_VERSION);
    assert_eq!(
        Some(attestation.server_sha256.as_str()),
        expected
            .get("redis_server_sha256")
            .and_then(serde_json::Value::as_str)
    );
    assert_eq!(
        Some(attestation.client_sha256.as_str()),
        expected
            .get("redis_cli_sha256")
            .and_then(serde_json::Value::as_str)
    );
    assert!(!layout.receipt.exists());
    Ok(())
}

fn reject_swap(server: bool, after_first_probe: bool) -> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let path = if server {
        &layout.server
    } else {
        &layout.client
    };
    let original = fs::read(path)?;
    let saved = path.with_extension("probed");
    fs::write(&layout.receipt, b"foreign receipt")?;
    let mut probes: usize = 0;
    let result = attest(&source, |file| {
        probes = probes.saturating_add(1);
        let version = observed_version(file, probes == 1)?;
        if probes == if after_first_probe { 1 } else { 2 } {
            fs::rename(path, &saved)?;
            fs::write(path, b"unprobed replacement")?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
        }
        Ok(version)
    });
    let error = result.err().ok_or("unprobed replacement was attested")?;
    assert_eq!(
        error.to_string(),
        "Redis binary identity changed during attestation"
    );
    assert_eq!(probes, 2);
    assert_eq!(fs::read(path)?, b"unprobed replacement");
    assert_eq!(fs::read(saved)?, original);
    assert_eq!(fs::read(&layout.receipt)?, b"foreign receipt");
    Ok(())
}

#[test]
fn given_filename_swap_after_probes_when_hashing_then_unprobed_bytes_are_rejected() -> TestResult {
    [true, false]
        .into_iter()
        .try_for_each(|server| reject_swap(server, false))
}

#[test]
fn given_client_swap_during_server_probe_when_attesting_then_client_descriptor_is_retained()
-> TestResult {
    reject_swap(false, true)
}

#[test]
fn given_filename_replacement_when_verifying_digest_then_retained_file_attests_original_bytes()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let expected = value(&layout)?;
    let retained = open_regular(&layout.server)?;
    let saved = layout.server.with_extension("probed");
    fs::rename(&layout.server, &saved)?;
    fs::write(&layout.server, b"unprobed replacement")?;
    fs::set_permissions(&layout.server, fs::Permissions::from_mode(0o755))?;
    verify_binary_digest(&expected, &retained, "redis_server_sha256")?;
    let replacement = open_regular(&layout.server)?;
    assert!(verify_binary_digest(&expected, &replacement, "redis_server_sha256").is_err());
    assert_eq!(
        same_binary(&retained, &replacement)
            .err()
            .ok_or("replacement identity accepted")?
            .to_string(),
        "Redis binary identity changed during attestation"
    );
    assert_eq!(fs::read(saved)?, b"abc");
    assert_eq!(fs::read(&layout.server)?, b"unprobed replacement");
    Ok(())
}

#[test]
fn given_owner_without_execute_when_other_class_can_execute_then_effective_identity_is_checked()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let retained = open_regular(&layout.server)?;
    [0o601, 0o610]
        .into_iter()
        .try_for_each(|mode| -> TestResult {
            retained.set_permissions(fs::Permissions::from_mode(mode))?;
            let result = executable(&retained);
            if retained.metadata()?.uid() == 0 {
                result?;
            } else {
                assert_eq!(
                    result
                        .err()
                        .ok_or("owner inherited another class's execute bit")?
                        .to_string(),
                    "Redis binary has no executable permissions"
                );
            }
            Ok(())
        })?;
    retained.set_permissions(fs::Permissions::from_mode(0o700))?;
    executable(&retained)?;
    retained.set_permissions(fs::Permissions::from_mode(0o600))?;
    assert_eq!(
        executable(&retained)
            .err()
            .ok_or("non-executable file accepted")?
            .to_string(),
        "Redis binary has no executable permissions"
    );
    Ok(())
}

#[test]
fn given_same_inode_write_during_client_probe_when_attesting_then_unprobed_bytes_are_rejected()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let before = fs::metadata(&layout.server)?;
    let writer = fs::OpenOptions::new().write(true).open(&layout.server)?;
    fs::write(&layout.receipt, b"foreign receipt")?;
    let mut probes: usize = 0;
    let result = attest(&source, |file| {
        probes = probes.saturating_add(1);
        let version = observed_version(file, probes == 1)?;
        if probes == 2 {
            writer.write_all_at(b"xyz", 0)?;
        }
        Ok(version)
    });
    let error = result
        .err()
        .ok_or("mutated inode was attested with an old version")?;
    assert_eq!(
        error.to_string(),
        "Redis binary bytes changed during attestation"
    );
    let after = fs::metadata(&layout.server)?;
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert_eq!(fs::read(&layout.server)?, b"xyz");
    assert_eq!(fs::read(&layout.receipt)?, b"foreign receipt");
    assert_eq!(probes, 2);
    Ok(())
}

#[test]
fn given_sealed_snapshot_when_original_or_snapshot_is_written_then_attested_bytes_are_immutable()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let original = open_regular(&layout.server)?;
    let sealed = snapshot(&original)?;
    let write = sealed
        .write_all_at(b"xyz", 0)
        .err()
        .ok_or("sealed snapshot was writable")?;
    assert_eq!(write.raw_os_error(), Some(libc::EPERM));
    assert_eq!(
        sealed
            .set_len(0)
            .err()
            .ok_or("sealed snapshot was shrinkable")?
            .raw_os_error(),
        Some(libc::EPERM)
    );
    assert_eq!(
        sealed
            .set_len(4)
            .err()
            .ok_or("sealed snapshot was growable")?
            .raw_os_error(),
        Some(libc::EPERM)
    );
    let writer = fs::OpenOptions::new().write(true).open(&layout.server)?;
    writer.write_all_at(b"xyz", 0)?;
    assert_eq!(
        digest_file(&sealed)?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_ne!(digest_file(&original)?, digest_file(&sealed)?);
    assert_eq!(fs::read(&layout.server)?, b"xyz");
    Ok(())
}

#[test]
fn given_replaced_layout_root_when_publishing_then_both_roots_receipts_are_preserved() -> TestResult
{
    let (_directory, layout) = fixture()?;
    let retained = tempfile::tempdir()?;
    let saved = retained.path().join("original");
    fs::write(&layout.receipt, b"original receipt")?;
    let source = OwnedDirectory::capture(&layout.source)?;
    fs::rename(&layout.root, &saved)?;
    fs::create_dir(&layout.root)?;
    fs::write(&layout.receipt, b"replacement receipt")?;
    fs::write(&layout.archive, b"replacement archive")?;
    let error = super::publish(&layout, &source)
        .err()
        .ok_or("replacement root was accepted")?;
    assert_eq!(
        error.to_string(),
        "graph installation root identity changed"
    );
    assert_eq!(fs::read(&layout.receipt)?, b"replacement receipt");
    assert_eq!(fs::read(&layout.archive)?, b"replacement archive");
    assert_eq!(
        fs::read(saved.join("installation.json"))?,
        b"original receipt"
    );
    assert_eq!(
        fs::read(saved.join("redis-8.10.3/src/redis-server"))?,
        b"abc"
    );
    source.verify()?;
    Ok(())
}

fn inspect_visibility(
    path: &std::path::Path,
    phase: super::PublicationPhase,
    staged_identity: (u64, u64),
    expected: &[u8],
) -> std::io::Result<()> {
    use std::io::Read;
    if phase == super::PublicationPhase::Before {
        assert!(!path.try_exists()?);
        return Ok(());
    }
    let mut visible = File::open(path)?;
    let metadata = visible.metadata()?;
    assert_eq!((metadata.dev(), metadata.ino()), staged_identity);
    let mut bytes = Vec::new();
    visible.read_to_end(&mut bytes)?;
    assert_eq!(bytes, expected);
    assert_eq!(
        metadata.len(),
        u64::try_from(expected.len()).map_err(std::io::Error::other)?
    );
    Ok(())
}

#[test]
fn given_unpublished_receipt_when_link_becomes_visible_then_first_observer_sees_complete_staged_inode()
-> TestResult {
    let (_directory, layout) = fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let value = value(&layout)?;
    let expected = serde_json::to_vec(&value)?;
    let mut staged = super::stage_file(&source)?;
    let metadata = staged.metadata()?;
    let identity = (metadata.dev(), metadata.ino());
    let mut phases = Vec::new();
    let mut observations = Vec::new();
    super::write_receipt_observed(&source, &mut staged, &value, |phase| {
        phases.push(phase);
        observations.push(std::thread::scope(|scope| {
            scope
                .spawn(|| inspect_visibility(&layout.receipt, phase, identity, &expected))
                .join()
        }));
    })?;
    assert_eq!(
        phases,
        [
            super::PublicationPhase::Before,
            super::PublicationPhase::After
        ]
    );
    observations
        .into_iter()
        .try_for_each(|observation| -> TestResult {
            match observation {
                Ok(result) => result.map_err(Into::into),
                Err(_) => Err("publication visibility observer panicked".into()),
            }
        })?;
    assert_eq!(fs::read(&layout.receipt)?, expected);
    Ok(())
}
