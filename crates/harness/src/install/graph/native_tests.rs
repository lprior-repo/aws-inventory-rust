use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use serde_json::Value;

use super::installer_tests::{CLIENT_VERSION, SERVER_VERSION, TestResult, executable, fixture};
use super::{
    DEFAULT_JOBS, LINUX_X86_64_MODULE, Layout, OwnedDirectory, REDIS_SHA256, install, receipt,
};

fn genuine_fixture() -> Result<(tempfile::TempDir, Layout), Box<dyn std::error::Error>> {
    let artifacts = std::env::var_os("AWS_INVENTORY_GRAPH_ARTIFACTS")
        .ok_or("native integration requires AWS_INVENTORY_GRAPH_ARTIFACTS")?;
    let artifacts = Path::new(&artifacts);
    let (directory, layout) = fixture()?;
    fs::copy(artifacts.join("redis-8.10.3.tar.gz"), &layout.archive)?;
    fs::copy(artifacts.join("falkordb.so"), &layout.module)?;
    assert_eq!(receipt::digest(&layout, &layout.archive)?, REDIS_SHA256);
    assert_eq!(
        receipt::digest(&layout, &layout.module)?,
        LINUX_X86_64_MODULE.sha256
    );
    version_program(&layout.server, SERVER_VERSION, &layout.root.join("probes"))?;
    version_program(&layout.client, CLIENT_VERSION, &layout.root.join("probes"))?;
    let source = OwnedDirectory::capture(&layout.source)?;
    receipt::publish(&layout, &source)?;
    Ok((directory, layout))
}

fn version_program(path: &Path, version: &str, marker: &Path) -> TestResult {
    let marker = marker
        .to_str()
        .ok_or("non-UTF8 native fixture path")?
        .replace('\'', "'\\''");
    executable(
        path,
        &format!("printf probe >> '{marker}'\nprintf '%s\\n' '{version}'"),
    )
}

fn cache_error(layout: &Layout, expected: &str) -> TestResult {
    let receipt_before = fs::read(&layout.receipt)?;
    let server_before = fs::read(&layout.server)?;
    let client_before = fs::read(&layout.client)?;
    let error = install(&layout.root, DEFAULT_JOBS)
        .err()
        .ok_or("stale cache accepted")?;
    assert_eq!(error.to_string(), expected);
    assert_eq!(fs::read(&layout.receipt)?, receipt_before);
    assert_eq!(fs::read(&layout.server)?, server_before);
    assert_eq!(fs::read(&layout.client)?, client_before);
    assert!(!layout.lock.exists());
    Ok(())
}

fn reject_binary_changes(layout: &Layout, path: &Path, digest_field: &str) -> TestResult {
    let original = fs::read(path)?;
    fs::write(path, b"changed binary bytes")?;
    cache_error(
        layout,
        &format!("receipt binary digest mismatch: {digest_field}"),
    )?;
    fs::write(path, original)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    cache_error(layout, "Redis binary has no executable permissions")?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[test]
fn given_genuine_pins_when_reusing_public_cache_then_no_native_program_runs() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    let marker = layout.root.join("probes");
    assert_eq!(fs::read(&marker)?, b"probeprobe");
    let expected = serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?;
    assert_eq!(install(&layout.root, DEFAULT_JOBS)?, expected);
    assert_eq!(fs::read(marker)?, b"probeprobe");
    Ok(())
}

#[test]
fn given_stale_binary_or_chmod_when_reusing_public_cache_then_reuse_is_rejected() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    for (path, digest_field) in [
        (&layout.server, "redis_server_sha256"),
        (&layout.client, "redis_cli_sha256"),
    ] {
        reject_binary_changes(&layout, path, digest_field)?;
    }
    Ok(())
}

#[test]
fn given_wrong_versions_when_reusing_public_cache_then_reuse_is_rejected() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    let original = serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?;
    for name in ["server", "client"] {
        let mut invalid = original.clone();
        let versions = invalid
            .get_mut("versions")
            .and_then(Value::as_object_mut)
            .ok_or("versions object missing")?;
        versions.insert(
            name.to_owned(),
            Value::String("wrong native version".to_owned()),
        );
        fs::write(&layout.receipt, serde_json::to_vec(&invalid)?)?;
        cache_error(&layout, "compiled Redis version differs from the pin")?;
    }
    Ok(())
}

#[test]
fn given_changed_download_when_reusing_public_cache_then_reuse_is_rejected() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    for path in [&layout.archive, &layout.module] {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut prefix = [0_u8; 1];
        file.read_exact(&mut prefix)?;
        prefix.iter_mut().for_each(|byte| *byte ^= 0xff);
        file.rewind()?;
        file.write_all(&prefix)?;
        cache_error(&layout, "installation download digest differs from pin")?;
        prefix.iter_mut().for_each(|byte| *byte ^= 0xff);
        file.rewind()?;
        file.write_all(&prefix)?;
    }
    Ok(())
}

#[test]
fn given_aliased_genuine_module_when_installing_then_foreign_permissions_are_unchanged()
-> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    fs::remove_file(&layout.receipt)?;
    OwnedDirectory::capture(&layout.source)?.remove_tree()?;
    fs::set_permissions(&layout.module, fs::Permissions::from_mode(0o600))?;
    let foreign = layout.root.join("foreign-module");
    fs::hard_link(&layout.module, &foreign)?;
    let error = install(&layout.root, DEFAULT_JOBS)
        .err()
        .ok_or("aliased module accepted")?;
    assert_eq!(
        error.to_string(),
        "refusing executable permission change on aliased inode"
    );
    assert_eq!(fs::metadata(&foreign)?.permissions().mode() & 0o777, 0o600);
    assert!(!layout.source.exists());
    assert!(!layout.receipt.exists());
    assert!(!layout.lock.exists());
    Ok(())
}

#[test]
fn given_wrong_schema_when_reusing_public_cache_then_receipt_is_preserved() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    let mut invalid = serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?;
    *invalid.get_mut("schema_version").ok_or("schema missing")? = Value::from(1);
    fs::write(&layout.receipt, serde_json::to_vec(&invalid)?)?;
    cache_error(&layout, "unsupported graph installation receipt")
}

#[test]
fn given_missing_fields_when_reusing_public_cache_then_receipt_is_preserved() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    let original = serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?;
    for (name, expected) in [
        (
            "redis_server_sha256",
            "receipt has no string field: redis_server_sha256",
        ),
        (
            "redis_cli_sha256",
            "receipt has no string field: redis_cli_sha256",
        ),
        ("redis_url", "receipt has no string field: redis_url"),
        ("versions", "receipt has no Redis versions"),
    ] {
        reject_missing_field(&layout, &original, name, expected)?;
    }
    Ok(())
}

fn reject_missing_field(
    layout: &Layout,
    original: &Value,
    name: &str,
    expected: &str,
) -> TestResult {
    let mut invalid = original.clone();
    invalid
        .as_object_mut()
        .ok_or("receipt object missing")?
        .remove(name)
        .ok_or("fixture field missing")?;
    fs::write(&layout.receipt, serde_json::to_vec(&invalid)?)?;
    cache_error(layout, expected)
}

#[test]
fn given_wrong_paths_when_reusing_public_cache_then_foreign_inode_is_preserved() -> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    install(&layout.root, DEFAULT_JOBS)?;
    let original = serde_json::from_slice::<Value>(&fs::read(&layout.receipt)?)?;
    let foreign = layout.root.join("foreign");
    fs::write(&foreign, b"foreign")?;
    let before = fs::metadata(&foreign)?;
    for name in ["redis_server", "redis_cli", "falkor_module"] {
        reject_wrong_path(&layout, &original, name, &foreign)?;
        let after = fs::metadata(&foreign)?;
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
        assert_eq!(fs::read(&foreign)?, b"foreign");
    }
    Ok(())
}

fn reject_wrong_path(layout: &Layout, original: &Value, name: &str, foreign: &Path) -> TestResult {
    let mut invalid = original.clone();
    *invalid.get_mut(name).ok_or("path field missing")? = serde_json::to_value(foreign)?;
    fs::write(&layout.receipt, serde_json::to_vec(&invalid)?)?;
    cache_error(layout, &format!("receipt path mismatch: {name}"))
}

#[test]
fn given_owner_without_execute_when_reusing_cache_then_other_class_bits_do_not_authorize()
-> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    let receipt_before = fs::read(&layout.receipt)?;
    let probes_before = fs::read(layout.root.join("probes"))?;
    [0o601, 0o610]
        .into_iter()
        .try_for_each(|mode| -> TestResult {
            fs::set_permissions(&layout.server, fs::Permissions::from_mode(mode))?;
            if fs::metadata(&layout.server)?.uid() == 0 {
                assert_eq!(
                    install(&layout.root, DEFAULT_JOBS)?,
                    serde_json::from_slice::<Value>(&receipt_before)?,
                );
            } else {
                cache_error(&layout, "Redis binary has no executable permissions")?;
            }
            Ok(())
        })?;
    assert_eq!(fs::read(&layout.receipt)?, receipt_before);
    assert_eq!(fs::read(layout.root.join("probes"))?, probes_before);
    Ok(())
}

#[test]
fn given_binary_swapped_by_version_probe_when_publishing_then_foreign_receipt_is_preserved()
-> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    let receipt_before = fs::read(&layout.receipt)?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let saved = layout.server.with_extension("probed");
    let server = layout
        .server
        .to_str()
        .ok_or("non-UTF8 binary path")?
        .replace('\'', "'\\''");
    let saved_path = saved
        .to_str()
        .ok_or("non-UTF8 saved path")?
        .replace('\'', "'\\''");
    executable(
        &layout.server,
        &format!(
            "mv -- '{server}' '{saved_path}' || exit 1\n\
             printf '%s' 'unprobed replacement' > '{server}' || exit 1\n\
             chmod 755 '{server}' || exit 1\n\
             printf '%s\\n' '{SERVER_VERSION}'"
        ),
    )?;
    let probed = fs::read(&layout.server)?;
    let error = receipt::publish(&layout, &source)
        .err()
        .ok_or("swapped bytes were published")?;
    assert!(matches!(&error, receipt::PublicationError::Uncommitted(_)));
    assert_eq!(
        error.to_string(),
        "Redis binary identity changed during attestation"
    );
    assert_eq!(fs::read(&layout.server)?, b"unprobed replacement");
    assert_eq!(fs::read(saved)?, probed);
    assert_eq!(fs::read(&layout.receipt)?, receipt_before);
    source.verify()?;
    Ok(())
}

#[test]
fn given_relocated_pinned_root_when_publishing_then_absolute_and_retained_layouts_must_match()
-> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    let retained = tempfile::tempdir()?;
    let saved = retained.path().join("original");
    let source = OwnedDirectory::capture(&layout.source)?;
    let original_receipt = fs::read(&layout.receipt)?;
    fs::rename(&layout.root, &saved)?;
    fs::create_dir(&layout.root)?;
    fs::copy(saved.join("redis-8.10.3.tar.gz"), &layout.archive)?;
    fs::copy(saved.join("falkordb.so"), &layout.module)?;
    fs::write(&layout.receipt, b"foreign receipt")?;
    assert_eq!(receipt::digest(&layout, &layout.archive)?, REDIS_SHA256);
    assert_eq!(
        receipt::digest(&layout, &layout.module)?,
        LINUX_X86_64_MODULE.sha256
    );
    let error = receipt::publish(&layout, &source)
        .err()
        .ok_or("replacement layout was published")?;
    assert_eq!(
        error.to_string(),
        "graph installation root identity changed"
    );
    assert_eq!(fs::read(&layout.receipt)?, b"foreign receipt");
    assert_eq!(fs::read(saved.join("installation.json"))?, original_receipt);
    assert_eq!(fs::read(saved.join("probes"))?, b"probeprobe");
    assert!(!layout.server.exists());
    assert_eq!(receipt::digest(&layout, &layout.archive)?, REDIS_SHA256);
    assert_eq!(
        receipt::digest(&layout, &layout.module)?,
        LINUX_X86_64_MODULE.sha256
    );
    source.verify()?;
    Ok(())
}

#[test]
fn given_same_inode_write_by_client_probe_when_publishing_then_old_server_version_is_not_attested()
-> TestResult {
    let (_directory, layout) = genuine_fixture()?;
    let source = OwnedDirectory::capture(&layout.source)?;
    let original_receipt = fs::read(&layout.receipt)?;
    let before = fs::metadata(&layout.server)?;
    let server = layout
        .server
        .to_str()
        .ok_or("non-UTF8 binary path")?
        .replace('\'', "'\\''");
    executable(
        &layout.client,
        &format!(
            "printf '%s' 'unprobed replacement' > '{server}' || exit 1\n\
             printf '%s\\n' '{CLIENT_VERSION}'"
        ),
    )?;
    let error = receipt::publish(&layout, &source)
        .err()
        .ok_or("mutated server inode was published")?;
    assert_eq!(
        error.to_string(),
        "Redis binary bytes changed during attestation"
    );
    let after = fs::metadata(&layout.server)?;
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert_eq!(fs::read(&layout.server)?, b"unprobed replacement");
    assert_eq!(fs::read(&layout.receipt)?, original_receipt);
    source.verify()?;
    Ok(())
}
