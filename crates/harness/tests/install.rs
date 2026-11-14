use serde_json::json;

use inventory_harness::install::{install, signed_by};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const KEY: &str = "98BCF336A3BE53B734E967C52E5BF49BD940F624";

#[test]
fn given_validsig_status_when_signer_is_third_field_then_it_is_accepted() {
    let status = format!("[GNUPG:] VALIDSIG {KEY} 2026-11-12 1893456000 0 4 0 1 10 00 {KEY}\n");
    assert!(signed_by(status.as_bytes(), KEY));
}

#[test]
fn given_validsig_status_when_trailing_field_is_the_signer_then_it_is_accepted() {
    let status = format!("[GNUPG:] VALIDSIG DEADBEEF 2026-11-12 1893456000 0 4 0 1 10 00 {KEY}\n");
    assert!(signed_by(status.as_bytes(), KEY));
}

#[test]
fn given_other_signer_when_verifying_then_status_is_rejected() {
    let status = "[GNUPG:] VALIDSIG 1111AAAA 2026-11-12 1893456000 0 4 0 1 10 00 1111AAAA\n";
    assert!(!signed_by(status.as_bytes(), KEY));
}

#[test]
fn given_human_readable_noise_when_verifying_then_only_status_lines_count() {
    let noise = format!(
        "gpg: Signature made Thu Nov 12 2026\n[GNUPG:] NEWSIG\ngpg: Good signature from \"{KEY}\"\n"
    );
    assert!(!signed_by(noise.as_bytes(), KEY));
}

#[tokio::test]
async fn given_incomplete_pinned_metadata_when_installing_then_it_is_rejected_early() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    let metadata = root.join("scripts/emulator/upstream.json");
    std::fs::create_dir_all(metadata.parent().ok_or("metadata parent")?)?;
    std::fs::write(
        &metadata,
        serde_json::to_string(&json!({
            "repository": "https://github.com/aws/aws-emulator.git",
            "commit": inventory_harness::emulator::PINNED_COMMIT
        }))?,
    )?;
    let destination = root.join("runtime");
    let error = install(root, &destination)
        .err()
        .ok_or("incomplete metadata is refused")?;
    assert!(
        error
            .to_string()
            .contains("upstream.json lacks package_url")
    );
    assert!(
        !destination.exists(),
        "nothing is created before verification"
    );
    Ok(())
}

#[tokio::test]
async fn given_pinned_metadata_when_signature_check_runs_then_the_real_key_is_honoured()
-> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let metadata: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        root.join("scripts/emulator/upstream.json"),
    )?)?;
    let key = metadata
        .get("package_signing_key_fingerprint")
        .and_then(serde_json::Value::as_str)
        .ok_or("pinned fingerprint")?;
    assert_eq!(key.len(), 40);
    let status = format!("[GNUPG:] VALIDSIG {key} 2026-11-12 1893456000 0 4 0 1 10 00 {key}\n");
    assert!(signed_by(status.as_bytes(), key));
    Ok(())
}
