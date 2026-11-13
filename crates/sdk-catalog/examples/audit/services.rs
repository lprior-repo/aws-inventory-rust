//! services operations.
use super::*;

pub(super) fn audit_service(
    service: &Value,
    options: &Options,
    totals: &mut Totals,
) -> Result<Value> {
    let name = field(service, "name")?;
    let version = field(service, "version")?;
    if !name.starts_with("aws-sdk-") || !valid_component(name) || !valid_component(version) {
        return Err("invalid crate name or version path component".into());
    }
    let directory = options.registry.join(format!("{name}-{version}"));
    let archive = read_bounded(
        &options.cache.join(format!("{name}-{version}.crate")),
        MAX_ARCHIVE_BYTES,
    )?;
    add_bytes(
        &mut totals.archive_bytes,
        u64::try_from(archive.len())?,
        MAX_ARCHIVE_TOTAL_BYTES,
    )?;
    let archive_sha256 = sha256(&archive);
    if archive_sha256 != field(service, "checksum_sha256")? {
        return Err(format!("archive checksum mismatch for {name}-{version}").into());
    }
    drop(archive);
    let manifest_text = source_text(&directory.join("Cargo.toml"), totals)?;
    let manifest: toml::Table = manifest_text.parse()?;
    let package = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or("missing package table")?;
    if package.get("name").and_then(toml::Value::as_str) != Some(name)
        || package.get("version").and_then(toml::Value::as_str) != Some(version)
    {
        return Err(format!("local manifest identity mismatch for {name}-{version}").into());
    }
    let config = source_text(&directory.join("src/config.rs"), totals)?;
    let operation_index = source_text(&directory.join("src/operation.rs"), totals)?;
    let mut digest = Sha256::new();
    source_digest_add(&mut digest, "Cargo.toml", &manifest_text);
    source_digest_add(&mut digest, "src/config.rs", &config);
    source_digest_add(&mut digest, "src/operation.rs", &operation_index);
    let mut methods = BTreeMap::new();
    for method in CONFIG_METHODS {
        let marker = format!("pub fn {method}(");
        let line = config
            .lines()
            .position(|line| line.trim_start().starts_with(&marker))
            .and_then(|line| line.checked_add(1));
        if line.is_some() {
            checked_sum(totals.methods.entry(method).or_default(), 1)?;
        } else {
            totals.methods.entry(method).or_default();
        }
        methods.insert(
            method,
            json!({"public_method_present": line.is_some(), "line": line}),
        );
    }
    let operations = regular_rs_files(&directory.join("src/operation"))?;
    checked_sum(&mut totals.operations, operations.len())?;
    if totals.operations > MAX_OPERATIONS {
        return Err("total operation bound exceeded".into());
    }
    let mut operation_names = BTreeSet::new();
    let mut paginators = BTreeSet::new();
    let mut long_polling = BTreeSet::new();
    let mut classifiers = BTreeMap::new();
    let mut paginator_duplicate_guard = 0usize;
    for path in &operations {
        let module = module_name(path)?;
        operation_names.insert(module.to_owned());
        let text = source_text(path, totals)?;
        source_digest_add(&mut digest, &format!("src/operation/{module}.rs"), &text);
        for (classifier, marker) in CLASSIFIERS {
            let count = usize::from(text.contains(marker));
            checked_sum(classifiers.entry(classifier).or_default(), count)?;
            checked_sum(totals.classifiers.entry(classifier).or_default(), count)?;
        }
        if text.contains("crate::long_polling::LongPollingInterceptor") {
            long_polling.insert(module.to_owned());
        }
        let paginator = path.with_extension("").join("paginator.rs");
        if paginator.is_file() {
            let text = source_text(&paginator, totals)?;
            source_digest_add(
                &mut digest,
                &format!("src/operation/{module}/paginator.rs"),
                &text,
            );
            paginators.insert(module.to_owned());
            if text.contains("pub fn stop_on_duplicate_token(") {
                checked_sum(&mut paginator_duplicate_guard, 1)?;
            }
        }
    }
    if operation_names != declared_modules(&operation_index)? {
        return Err(format!("operation source/index mismatch for {name}").into());
    }
    if !long_polling.is_empty() {
        let text = source_text(&directory.join("src/long_polling.rs"), totals)?;
        if !text.contains("spec.with_long_polling(true)") {
            return Err(format!("long-polling helper marker missing for {name}").into());
        }
        source_digest_add(&mut digest, "src/long_polling.rs", &text);
    }
    let mut waiters = BTreeSet::new();
    for path in regular_rs_files(&directory.join("src/waiters"))? {
        let module = module_name(&path)?;
        let text = source_text(&path, totals)?;
        source_digest_add(&mut digest, &format!("src/waiters/{module}.rs"), &text);
        if module != "matchers" {
            waiters.insert(module.to_owned());
        }
    }
    let waiter_index_path = directory.join("src/waiters.rs");
    if waiter_index_path.is_file() {
        let waiter_index = source_text(&waiter_index_path, totals)?;
        source_digest_add(&mut digest, "src/waiters.rs", &waiter_index);
        if waiters != declared_modules(&waiter_index)? {
            return Err(format!("waiter source/index mismatch for {name}").into());
        }
    } else if !waiters.is_empty() {
        return Err(format!("missing waiter index for {name}").into());
    }
    checked_sum(&mut totals.paginators, paginators.len())?;
    checked_sum(&mut totals.waiters, waiters.len())?;
    if totals.waiters > MAX_OPERATIONS {
        return Err("total waiter bound exceeded".into());
    }
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or("missing dependency table")?;
    let runtime_requirements: BTreeMap<_, _> = dependencies
        .iter()
        .filter(|(name, _)| {
            name.starts_with("aws-smithy-")
                || matches!(
                    name.as_str(),
                    "aws-runtime" | "aws-types" | "aws-credential-types"
                )
        })
        .map(|(name, value)| {
            (
                name,
                value
                    .get("version")
                    .and_then(toml::Value::as_str)
                    .or_else(|| value.as_str()),
            )
        })
        .collect();
    let metadata = package
        .get("metadata")
        .and_then(|value| value.get("smithy"));
    Ok(json!({
        "name": name, "version": version, "origin": service.get("origin"),
        "archive_sha256": archive_sha256, "archive_sha256_matches_catalog": true,
        "manifest_sha256": sha256(manifest_text.as_bytes()), "config_sha256": sha256(config.as_bytes()),
        "audited_source_digest_sha256": format!("{:x}", base16ct::HexDisplay(&digest.finalize())),
        "source_digest_algorithm": "SHA256 of ordered relative-path NUL hex-SHA256-of-file newline records; manifest, config, operation index, sorted operations with paginator after each, registered long-polling helper when present, sorted waiters including matchers, waiter index when present",
        "rust_version": package.get("rust-version").and_then(toml::Value::as_str),
        "codegen_revision": metadata.and_then(|value| value.get("codegen-version")).and_then(toml::Value::as_str),
        "protocol": metadata.and_then(|value| value.get("protocol")).and_then(toml::Value::as_str),
        "runtime_dependency_requirements": runtime_requirements,
        "config_methods": methods,
        "operation_count": operation_names.len(), "operations": operation_names,
        "operation_classifier_registrations": classifiers,
        "paginator_count": paginators.len(), "paginators": paginators,
        "paginators_with_duplicate_token_control": paginator_duplicate_guard,
        "waiter_count_excluding_matchers": waiters.len(), "waiters": waiters,
        "long_polling_retry_marker_operations": long_polling,
        "dynamodb_tuned_retry_markers": config.contains("with_max_attempts(4)") && config.contains("with_non_throttling_initial_backoff(::std::time::Duration::from_millis(25))"),
        "collector_coverage": "not established by this SDK source audit"
    }))
}
