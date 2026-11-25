use super::*;
use inventory_core::catalog::runtime::{ModelSourceProvenance, RuntimeCatalog};
use sha2::{Digest, Sha256};

fn fixture_models_sha256(model: &str) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"aws-model-source-v1\0");
    hash.update(18_u64.to_le_bytes());
    hash.update(b"example/model.json");
    hash.update(u64::try_from(model.len()).unwrap_or(u64::MAX).to_le_bytes());
    hash.update(model.as_bytes());
    hash.finalize().into()
}

fn embedded_sources_sha256() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    fn collect(
        path: &Path,
        files: &mut std::collections::BTreeMap<String, Vec<u8>>,
        root: &Path,
    ) -> TestResult {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                collect(&entry?.path(), files, root)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let relative = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF-8 source path")?
                .replace('\\', "/");
            files.insert(relative, fs::read(path)?);
        }
        Ok(())
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = std::collections::BTreeMap::new();
    for name in [
        "build.rs",
        "src/overlay.rs",
        "src/overlay",
        "src/kind_map.rs",
        "src/kind_map",
        "src/names.rs",
        "src/names",
        "src/classes.rs",
        "src/classes",
    ] {
        collect(&root.join(name), &mut files, root)?;
    }
    let mut hash = Sha256::new();
    hash.update(b"catalog-tool-embedded-sources-v1\0");
    for (path, bytes) in files {
        hash.update(u64::try_from(path.len())?.to_le_bytes());
        hash.update(path.as_bytes());
        hash.update(u64::try_from(bytes.len())?.to_le_bytes());
        hash.update(&bytes);
    }
    Ok(hash.finalize().into())
}

fn rejected(output: &Output, root: &Path, message: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    assert!(!root.join("catalog/aws-api.catalog").exists());
}

#[test]
fn nested_plain_model_directory_cannot_claim_its_parent_repository_commit() -> TestResult {
    let parent = source()?;
    let nested = parent.path().join("plain");
    fs::create_dir_all(nested.join("models/example"))?;
    fs::write(nested.join("models/example/model.json"), MODEL)?;
    let root = root()?;
    let output = compile(&nested, root.path(), &["--write"])?;
    rejected(&output, root.path(), "not its repository root");
    Ok(())
}

#[test]
fn modified_tracked_model_reports_the_first_uncommitted_path() -> TestResult {
    let source = source()?;
    fs::write(
        source.path().join("models/example/model.json"),
        MODEL.replace("ListItems", "GetItems"),
    )?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/example/model.json",
    );
    Ok(())
}

#[test]
fn untracked_model_reports_the_first_uncommitted_path() -> TestResult {
    let source = source()?;
    fs::write(source.path().join("models/example/extra.json"), MODEL)?;
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/example/extra.json",
    );
    Ok(())
}

#[test]
fn clean_repository_is_accepted_and_its_own_commit_is_embedded() -> TestResult {
    let source = source()?;
    let root = root()?;
    let head = git::command(source.path(), &["rev-parse", "HEAD"])?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    let bytes = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    success(&output);
    assert_eq!(Catalog::borrow(&bytes)?.models_commit()?, head.trim());
    Ok(())
}

fn ignored_source() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let source = source()?;
    fs::write(source.path().join(".git/info/exclude"), "/models/extra/\n")?;
    fs::create_dir_all(source.path().join("models/extra"))?;
    fs::write(
        source.path().join("models/extra/model.json"),
        r#"{"shapes":{"extra#Service":{"type":"service"}}}"#,
    )?;
    Ok(source)
}

#[test]
fn ignored_untracked_model_is_rejected_even_when_git_status_is_clean() -> TestResult {
    let source = ignored_source()?;
    let status = git::command(
        source.path(),
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            "models",
        ],
    )?;
    assert!(status.is_empty());
    let root = root()?;
    let output = compile(source.path(), root.path(), &["--write"])?;
    rejected(
        &output,
        root.path(),
        "model source has uncommitted changes: models/extra/model.json",
    );
    Ok(())
}

#[test]
fn published_receipt_binds_raw_models_cfn_and_active_and_inactive_embedded_sources() -> TestResult {
    let source = source()?;
    let root = root()?;
    let head = git::command(source.path(), &["rev-parse", "HEAD"])?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let bytes = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let pin = runtime.source_provenance();
    assert_eq!(pin.version, 1);
    assert_eq!(
        pin.models,
        ModelSourceProvenance::Repository {
            commit: head.trim().to_owned(),
            sha256: fixture_models_sha256(MODEL)
        }
    );
    let cfn = fs::read(root.path().join("catalog/cloudcontrol-universal.json"))?;
    assert_eq!(pin.cfn_sha256, Some(Sha256::digest(&cfn).into()));
    let kinds = catalog_tool::kind_map::embedded(&cfn)?;
    assert_eq!(pin.kind_map_version, u32::from(kinds.source_version()));
    assert_eq!(&pin.kind_map_sha256, kinds.source_sha256());
    assert_eq!(pin.embedded_sources_sha256, embedded_sources_sha256()?);
    Ok(())
}

#[test]
fn committed_model_only_whitespace_changes_raw_model_pin_not_reviewed_capabilities() -> TestResult {
    let source = source()?;
    let root = root()?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let before = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    let before = RuntimeCatalog::from_bytes(&before)?
        .source_provenance()
        .clone();
    let updated = format!("{MODEL}\n");
    fs::write(source.path().join("models/example/model.json"), &updated)?;
    git::command(source.path(), &["add", "--", "models"])?;
    git::command(
        source.path(),
        &["commit", "--quiet", "-m", "raw model whitespace"],
    )?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let after = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    let after = RuntimeCatalog::from_bytes(&after)?
        .source_provenance()
        .clone();
    let head = git::command(source.path(), &["rev-parse", "HEAD"])?;
    assert_eq!(
        after.models,
        ModelSourceProvenance::Repository {
            commit: head.trim().to_owned(),
            sha256: fixture_models_sha256(&updated)
        }
    );
    assert_ne!(after.models, before.models);
    assert_eq!(after.cfn_sha256, before.cfn_sha256);
    assert_eq!(after.kind_map_sha256, before.kind_map_sha256);
    assert_eq!(
        after.embedded_sources_sha256,
        before.embedded_sources_sha256
    );
    assert_eq!(after.reviewed_inputs_sha256, before.reviewed_inputs_sha256);
    Ok(())
}

#[test]
fn cfn_only_denominator_addition_changes_cfn_and_completed_kind_map_pins() -> TestResult {
    let source = source()?;
    let root = root()?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let before = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    let before = RuntimeCatalog::from_bytes(&before)?
        .source_provenance()
        .clone();
    let path = root.path().join("catalog/cloudcontrol-universal.json");
    let mut cfn: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    cfn.get_mut("resource_types")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or("CFN fixture resource_types missing")?
        .push(serde_json::json!({"type_name":"AWS::Example::Unreviewed"}));
    let cfn = serde_json::to_vec(&cfn)?;
    fs::write(&path, &cfn)?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let after = fs::read(root.path().join("catalog/aws-api.catalog"))?;
    let after = RuntimeCatalog::from_bytes(&after)?
        .source_provenance()
        .clone();
    assert_eq!(after.models, before.models);
    assert_eq!(after.cfn_sha256, Some(Sha256::digest(&cfn).into()));
    assert_ne!(after.cfn_sha256, before.cfn_sha256);
    assert_ne!(after.kind_map_sha256, before.kind_map_sha256);
    assert_eq!(
        after.embedded_sources_sha256,
        before.embedded_sources_sha256
    );
    Ok(())
}

fn hidden_tracked_model_change_is_rejected(flag: &str) -> TestResult {
    let source = source()?;
    git::command(
        source.path(),
        &["update-index", flag, "--", "models/example/model.json"],
    )?;
    fs::write(
        source.path().join("models/example/model.json"),
        format!("{MODEL}\n"),
    )?;
    assert!(git::command(source.path(), &["status", "--porcelain", "--", "models"])?.is_empty());
    let root = root()?;
    rejected(
        &compile(source.path(), root.path(), &["--write"])?,
        root.path(),
        "model source has uncommitted changes: models/example/model.json",
    );
    Ok(())
}

#[test]
fn assume_unchanged_does_not_allow_uncommitted_bytes_to_claim_the_head_commit() -> TestResult {
    hidden_tracked_model_change_is_rejected("--assume-unchanged")
}

#[test]
fn skip_worktree_does_not_allow_uncommitted_bytes_to_claim_the_head_commit() -> TestResult {
    hidden_tracked_model_change_is_rejected("--skip-worktree")
}

#[test]
fn unchanged_model_bytes_with_index_flags_retain_the_real_repository_source_receipt() -> TestResult
{
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        let source = source()?;
        git::command(
            source.path(),
            &["update-index", flag, "--", "models/example/model.json"],
        )?;
        let root = root()?;
        success(&compile(source.path(), root.path(), &["--write"])?);
        let bytes = fs::read(root.path().join("catalog/aws-api.catalog"))?;
        let head = git::command(source.path(), &["rev-parse", "HEAD"])?;
        assert_eq!(
            RuntimeCatalog::from_bytes(&bytes)?
                .source_provenance()
                .models,
            ModelSourceProvenance::Repository {
                commit: head.trim().to_owned(),
                sha256: fixture_models_sha256(MODEL)
            }
        );
    }
    Ok(())
}

#[test]
fn assembled_catalog_source_digest_is_verified_against_the_actual_base_catalog() -> TestResult {
    use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog};
    use inventory_core::{
        catalog::runtime::RuntimeCatalogError,
        record::{FieldId, Key, Record, Value, ValueRef},
    };
    let source = source()?;
    let root = root()?;
    success(&compile(source.path(), root.path(), &["--write"])?);
    let record = Record::from_bytes(fs::read(root.path().join("catalog/aws-api.catalog"))?)?;
    let Value::Map(mut fields) = ValueRef::Map(record.root()).to_value()? else {
        return Err("root is not a map".into());
    };
    fields
        .remove(&Key::Field(FieldId::new(60_000)?))
        .ok_or("missing runtime extension")?;
    let base = Record::encode(&Value::Map(fields))?.into_bytes();
    let checked = overlay::check(root.path(), source.path())?;
    let cfn = fs::read(root.path().join("catalog/cloudcontrol-universal.json"))?;
    let kinds = kind_map::embedded(&cfn)?;
    let endpoints = endpoints::verified_endpoints(source.path())?;
    let assembled = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
    assert_eq!(
        RuntimeCatalog::from_bytes(&assembled)?
            .source_provenance()
            .models,
        ModelSourceProvenance::CompiledCatalog {
            sha256: Sha256::digest(&base).into()
        }
    );
    let record = Record::from_bytes(assembled)?;
    let Value::Map(mut fields) = ValueRef::Map(record.root()).to_value()? else {
        return Err("root is not a map".into());
    };
    let Some(Value::Map(extension)) = fields.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
        return Err("missing extension".into());
    };
    let Some(Value::Map(receipt)) = extension.get_mut(&Key::Field(FieldId::new(7)?)) else {
        return Err("missing receipt".into());
    };
    let Some(Value::Map(models)) = receipt.get_mut(&Key::Field(FieldId::new(2)?)) else {
        return Err("missing models".into());
    };
    let Some(Value::Bytes(digest)) = models.get_mut(&Key::Field(FieldId::new(2)?)) else {
        return Err("missing source digest".into());
    };
    assert_eq!(digest.len(), 32);
    *digest.first_mut().ok_or("empty source digest")? ^= 1;
    let tampered = Record::encode(&Value::Map(fields))?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(tampered.as_bytes()),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn replacement_refs_cannot_change_the_model_tree_claimed_by_the_original_commit() -> TestResult {
    let source = source()?;
    let original = git::command(source.path(), &["rev-parse", "HEAD"])?;
    fs::write(
        source.path().join("models/example/model.json"),
        format!("{MODEL}\n"),
    )?;
    git::command(source.path(), &["add", "--", "models"])?;
    git::command(
        source.path(),
        &["commit", "--quiet", "-m", "replacement model"],
    )?;
    let replacement = git::command(source.path(), &["rev-parse", "HEAD"])?;
    git::command(source.path(), &["update-ref", "HEAD", original.trim()])?;
    git::command(
        source.path(),
        &["replace", original.trim(), replacement.trim()],
    )?;
    assert!(git::command(source.path(), &["status", "--porcelain", "--", "models"])?.is_empty());
    let root = root()?;
    rejected(
        &compile(source.path(), root.path(), &["--write"])?,
        root.path(),
        "model source has uncommitted changes: models/example/model.json",
    );
    Ok(())
}
