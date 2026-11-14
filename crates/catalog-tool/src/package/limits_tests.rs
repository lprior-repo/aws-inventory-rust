use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::limits::{self, Limits};
use super::{Payload, collect_payloads, package};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn limit_error<T>(
    result: Result<T, Box<dyn std::error::Error>>,
    path: &Path,
    detail: &str,
) -> TestResult {
    let error = result.err().ok_or("resource ceiling ignored")?;
    let error = error
        .downcast::<io::Error>()
        .map_err(|error| error.to_string())?;
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(
        error.to_string(),
        format!("packaging limit exceeded for {}: {detail}", path.display())
    );
    Ok(())
}

#[test]
fn given_file_at_byte_limit_when_collected_then_exact_bytes_are_retained() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("a"), b"abc")?;
    let limits = Limits {
        file_bytes: 3,
        ..limits::DEFAULT
    };

    let payloads = collect_payloads(root.path(), limits)?;

    assert_eq!(payloads, vec![("a".to_string(), b"abc".to_vec())]);
    Ok(())
}

#[test]
fn given_file_one_past_byte_limit_when_collected_then_path_and_limit_are_reported() -> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("a");
    fs::write(&path, b"abcd")?;
    let limits = Limits {
        file_bytes: 3,
        ..limits::DEFAULT
    };
    limit_error(
        collect_payloads(root.path(), limits),
        &path,
        "per-file bytes limit 3",
    )
}

#[test]
fn given_total_at_byte_limit_when_collected_then_both_payloads_are_retained() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("a"), b"ab")?;
    fs::write(root.path().join("b"), b"cd")?;
    let limits = Limits {
        total_bytes: 4,
        ..limits::DEFAULT
    };
    let payloads = collect_payloads(root.path(), limits)?;
    assert_eq!(
        payloads,
        vec![
            ("a".to_string(), b"ab".to_vec()),
            ("b".to_string(), b"cd".to_vec())
        ]
    );
    Ok(())
}

#[test]
fn given_total_one_past_byte_limit_when_collected_then_path_and_limit_are_reported() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("a"), b"ab")?;
    let path = root.path().join("b");
    fs::write(&path, b"cde")?;
    let limits = Limits {
        total_bytes: 4,
        ..limits::DEFAULT
    };
    limit_error(
        collect_payloads(root.path(), limits),
        &path,
        "total payload bytes limit 4",
    )
}

#[test]
fn given_empty_file_and_zero_byte_limits_when_collected_then_empty_payload_is_retained()
-> TestResult {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("a"), b"")?;
    let limits = Limits {
        file_bytes: 0,
        total_bytes: 0,
        ..limits::DEFAULT
    };
    assert_eq!(
        collect_payloads(root.path(), limits)?,
        vec![("a".to_string(), vec![])]
    );
    Ok(())
}

#[test]
fn given_directory_at_depth_limit_when_collected_then_payload_is_retained() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("a"))?;
    fs::write(root.path().join("a/file"), b"data")?;
    let limits = Limits {
        depth: 1,
        ..limits::DEFAULT
    };
    assert_eq!(
        collect_payloads(root.path(), limits)?,
        vec![("a/file".to_string(), b"data".to_vec())]
    );
    Ok(())
}

#[test]
fn given_directory_one_past_depth_limit_when_collected_then_path_and_limit_are_reported()
-> TestResult {
    let root = tempfile::tempdir()?;
    let path = root.path().join("a/b");
    fs::create_dir_all(&path)?;
    let limits = Limits {
        depth: 1,
        ..limits::DEFAULT
    };
    limit_error(
        collect_payloads(root.path(), limits),
        &path,
        "directory depth limit 1",
    )
}

#[test]
fn given_file_at_entry_limit_when_collected_then_payload_is_retained() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("a"), b"data")?;
    let limits = Limits {
        entries: 1,
        depth: 0,
        ..limits::DEFAULT
    };
    assert_eq!(
        collect_payloads(root.path(), limits)?,
        vec![("a".to_string(), b"data".to_vec())]
    );
    Ok(())
}

#[test]
fn given_entry_one_past_limit_when_collected_then_path_and_limit_are_reported() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("a"))?;
    let path = root.path().join("a/file");
    fs::write(&path, b"data")?;
    let limits = Limits {
        entries: 1,
        ..limits::DEFAULT
    };
    limit_error(
        collect_payloads(root.path(), limits),
        &path,
        "visited entries limit 1",
    )
}

#[test]
fn given_excluded_trees_past_depth_limit_when_collected_then_they_are_pruned() -> TestResult {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("target/a/b"))?;
    fs::create_dir_all(root.path().join(".local/a/b"))?;
    let limits = Limits {
        depth: 0,
        ..limits::DEFAULT
    };
    assert_eq!(
        collect_payloads(root.path(), limits)?,
        Vec::<Payload>::new()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn given_unreadable_excluded_directory_when_packaged_then_it_is_not_opened() -> TestResult {
    use std::os::unix::fs::PermissionsExt;

    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    let excluded = root.join("target");
    fs::create_dir_all(&excluded)?;
    fs::write(root.join("a.txt"), b"payload")?;
    fs::set_permissions(&excluded, fs::Permissions::from_mode(0o000))?;

    let result = package(&root, &fixture.path().join("out.zip"));
    fs::set_permissions(excluded, fs::Permissions::from_mode(0o700))?;

    assert_eq!(result?.files, 2);
    Ok(())
}

#[test]
fn given_archive_hash_boundaries_when_streamed_then_digest_matches_exact_bytes() -> TestResult {
    [0, 8192, 8193].into_iter().try_for_each(|length| {
        let mut archive = tempfile::NamedTempFile::new()?;
        let bytes = vec![b'x'; length];
        archive.write_all(&bytes)?;
        assert_eq!(
            super::hash_archive(archive.as_file_mut())?,
            super::sha256_hex(&bytes)
        );
        Ok(())
    })
}

#[test]
fn given_empty_root_with_zero_entries_when_collected_then_no_payloads_are_retained() -> TestResult {
    let root = tempfile::tempdir()?;
    let limits = Limits {
        entries: 0,
        depth: 0,
        ..limits::DEFAULT
    };
    assert_eq!(
        collect_payloads(root.path(), limits)?,
        Vec::<Payload>::new()
    );
    Ok(())
}

#[test]
fn given_oversized_sparse_payload_when_packaged_then_existing_outputs_are_preserved() -> TestResult
{
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("source");
    fs::create_dir(&root)?;
    let path = root.join("oversized");
    let length = limits::DEFAULT
        .file_bytes
        .checked_add(1)
        .ok_or("test size overflow")?;
    fs::File::create(&path)?.set_len(length)?;
    let manifest = root.join("MANIFEST.sha256");
    let destination = fixture.path().join("out.zip");
    fs::write(&manifest, b"old manifest\n")?;
    fs::write(&destination, b"old archive\n")?;
    let detail = format!("per-file bytes limit {}", limits::DEFAULT.file_bytes);
    limit_error(package(&root, &destination), &path, &detail)?;
    assert_eq!(fs::read(manifest)?, b"old manifest\n");
    assert_eq!(fs::read(destination)?, b"old archive\n");
    Ok(())
}
