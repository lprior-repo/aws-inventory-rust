//! Reproducible, offline source audit of the exact SDK catalog. No SDK features,
//! credentials, service clients, or AWS requests are involved.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MAX_SERVICES: usize = 1_024;
const MAX_DIRECTORY_ENTRIES: usize = 16_384;
const MAX_OPERATIONS: usize = 100_000;
const MAX_FILE_BYTES: u64 = 16_777_216;
const MAX_ARCHIVE_BYTES: u64 = 67_108_864;
const MAX_SOURCE_TOTAL_BYTES: u64 = 2_147_483_648;
const MAX_ARCHIVE_TOTAL_BYTES: u64 = 1_073_741_824;
const MAX_OUTPUT_BYTES: usize = 33_554_432;
const CONFIG_METHODS: [&str; 12] = [
    "retry_config",
    "http_client",
    "timeout_config",
    "retry_partition",
    "interceptor",
    "retry_classifier",
    "set_endpoint_resolver",
    "sleep_impl",
    "behavior_version",
    "runtime_plugin",
    "retry_strategy",
    "set_retry_strategy",
];
const CLASSIFIERS: [(&str, &str); 4] = [
    ("transient", "TransientErrorClassifier::<"),
    ("modeled_retryable", "ModeledAsRetryableClassifier::<"),
    ("aws_error_code", "AwsErrorCodeClassifier::<"),
    ("clock_skew", "ServiceClockSkewClassifier::<"),
];

struct Options {
    catalog: PathBuf,
    registry: PathBuf,
    cache: PathBuf,
    output: PathBuf,
}

#[derive(Default)]
struct Totals {
    source_bytes: u64,
    archive_bytes: u64,
    operations: usize,
    paginators: usize,
    waiters: usize,
    methods: BTreeMap<&'static str, usize>,
    classifiers: BTreeMap<&'static str, usize>,
}

fn main() -> Result<()> {
    let options = options()?;
    let catalog_bytes = read_bounded(&options.catalog, MAX_FILE_BYTES)?;
    let catalog: Value = serde_json::from_slice(&catalog_bytes)?;
    let services = catalog
        .get("services")
        .and_then(Value::as_array)
        .ok_or("missing services array")?;
    if services.is_empty() || services.len() > MAX_SERVICES {
        return Err("service count outside 1..=1024".into());
    }
    let declared = catalog
        .get("service_count")
        .and_then(Value::as_u64)
        .ok_or("missing service_count")?;
    if usize::try_from(declared)? != services.len()
        || services.len() != inventory_sdk_catalog::SERVICE_COUNT
    {
        return Err("catalog count does not match its entries or compiled catalog".into());
    }
    let mut totals = Totals::default();
    let mut records = BTreeMap::new();
    for service in services {
        let name = field(service, "name")?;
        if records.contains_key(name) {
            return Err(format!("duplicate catalog service: {name}").into());
        }
        let record = audit_service(service, &options, &mut totals)?;
        records.insert(name.to_owned(), record);
    }
    let report = json!({
        "schema_version": 1,
        "audit_kind": "offline lexical audit of exact published SDK source packages",
        "catalog_sha256": sha256(&catalog_bytes),
        "auditor_source_sha256": sha256(include_bytes!("audit.rs")),
        "official_release_tag": catalog.get("official_release_tag"),
        "official_revision": catalog.get("official_revision"),
        "smithy_rs_revision": catalog.get("smithy_rs_revision"),
        "evidence_scope": {
            "archives": "Every exact-version cached crate archive is SHA-256 checked against sdk-services.json.",
            "source": "Names and versions are parsed from each local Cargo.toml. Config, operation, paginator and waiter sources are read and fingerprinted. Source fingerprints are independent of archive verification; unpacked files are not byte-compared against archive entries.",
            "capabilities": "Public config method declarations and generated operation classifier registrations are matched lexically. Counts are source evidence, not compiled or exercised behavior.",
            "not_claimed": ["all SDKs compiled", "all operations tested", "read-only safety of arbitrary operations", "implemented collectors for all SDKs", "live AWS coverage", "per-service API quota values"]
        },
        "limits": {
            "services": MAX_SERVICES, "directory_entries": MAX_DIRECTORY_ENTRIES,
            "operations_total": MAX_OPERATIONS, "waiters_total": MAX_OPERATIONS,
            "single_source_bytes": MAX_FILE_BYTES,
            "single_archive_bytes": MAX_ARCHIVE_BYTES, "source_total_bytes": MAX_SOURCE_TOTAL_BYTES,
            "archive_total_bytes": MAX_ARCHIVE_TOTAL_BYTES, "output_bytes": MAX_OUTPUT_BYTES
        },
        "summary": {
            "services_inspected": records.len(), "archives_verified": records.len(),
            "operation_modules": totals.operations, "paginator_modules": totals.paginators,
            "waiter_modules_excluding_matchers": totals.waiters,
            "public_config_method_service_counts": totals.methods,
            "operation_classifier_registration_counts": totals.classifiers,
            "source_bytes_read": totals.source_bytes, "archive_bytes_hashed": totals.archive_bytes
        },
        "services": records.into_values().collect::<Vec<_>>()
    });
    let mut bytes = BoundedOutput(Vec::new());
    serde_json::to_writer_pretty(&mut bytes, &report)?;
    bytes.write_all(b"\n")?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&options.output)?;
    output.write_all(&bytes.0)?;
    output.sync_all()?;
    println!(
        "Audited {} exact SDKs, {} operations, {} paginators, {} waiters; {} output bytes.",
        services.len(),
        totals.operations,
        totals.paginators,
        totals.waiters,
        bytes.0.len()
    );
    Ok(())
}

fn options() -> Result<Options> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args_os().skip(1);
    for _ in 0..4 {
        let Some(flag) = args.next() else { break };
        let flag = flag
            .into_string()
            .map_err(|_| "argument name is not UTF-8")?;
        if !matches!(
            flag.as_str(),
            "--catalog" | "--registry-src" | "--registry-cache" | "--output"
        ) {
            return Err("usage: audit --catalog FILE --registry-src DIR [--registry-cache DIR] --output NEW_FILE".into());
        }
        let value = args.next().ok_or("missing argument value")?;
        if values.insert(flag, PathBuf::from(value)).is_some() {
            return Err("duplicate argument".into());
        }
    }
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let registry = values
        .remove("--registry-src")
        .ok_or("missing --registry-src")?;
    let cache = match values.remove("--registry-cache") {
        Some(cache) => cache,
        None => registry
            .parent()
            .and_then(Path::parent)
            .ok_or("cannot infer registry cache parent")?
            .join("cache")
            .join(
                registry
                    .file_name()
                    .ok_or("missing registry index directory name")?,
            ),
    };
    Ok(Options {
        catalog: values.remove("--catalog").ok_or("missing --catalog")?,
        registry,
        cache,
        output: values.remove("--output").ok_or("missing --output")?,
    })
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {name}").into())
}

fn checked_sum(value: &mut usize, amount: usize) -> Result<()> {
    *value = value.checked_add(amount).ok_or("counter overflow")?;
    Ok(())
}

fn add_bytes(value: &mut u64, amount: u64, maximum: u64) -> Result<()> {
    *value = value.checked_add(amount).ok_or("byte counter overflow")?;
    if *value > maximum {
        return Err("total byte bound exceeded".into());
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(format!("not a regular bounded file: {}", path.display()).into());
    }
    let mut bytes = Vec::new();
    file.take(maximum.checked_add(1).ok_or("read bound overflow")?)
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())? > maximum {
        return Err(format!("file grew beyond bound: {}", path.display()).into());
    }
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn source_text(path: &Path, totals: &mut Totals) -> Result<String> {
    let bytes = read_bounded(path, MAX_FILE_BYTES)?;
    add_bytes(
        &mut totals.source_bytes,
        u64::try_from(bytes.len())?,
        MAX_SOURCE_TOTAL_BYTES,
    )?;
    Ok(String::from_utf8(bytes)?)
}

fn source_digest_add(digest: &mut Sha256, relative_path: &str, text: &str) {
    // NUL separates paths from fixed-width SHA-256 strings; generated paths
    // cannot contain NUL. Files are processed in deterministic path order.
    digest.update(relative_path.as_bytes());
    digest.update(b"\0");
    digest.update(sha256(text.as_bytes()).as_bytes());
    digest.update(b"\n");
}

fn regular_rs_files(directory: &Path) -> Result<Vec<PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(directory)?;
    let mut paths = Vec::new();
    for _ in 0..MAX_DIRECTORY_ENTRIES {
        let Some(entry) = entries.next() else { break };
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().is_some_and(|value| value == "rs")
        {
            paths.push(entry.path());
        }
    }
    if entries.next().is_some() {
        return Err(format!("directory entry bound exceeded: {}", directory.display()).into());
    }
    paths.sort();
    Ok(paths)
}

fn module_name(path: &Path) -> Result<&str> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid module path".into())
}

fn declared_modules(index: &str) -> Result<BTreeSet<String>> {
    let mut modules = BTreeSet::new();
    for module in index.lines().filter_map(|line| {
        line.trim()
            .strip_prefix("pub mod ")
            .and_then(|value| value.strip_suffix(';'))
    }) {
        if !valid_component(module) || modules.len() >= MAX_DIRECTORY_ENTRIES {
            return Err("invalid or excessive module declarations".into());
        }
        if !modules.insert(module.to_owned()) {
            return Err("duplicate module declaration".into());
        }
    }
    Ok(modules)
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}

fn audit_service(service: &Value, options: &Options, totals: &mut Totals) -> Result<Value> {
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
        "audited_source_digest_sha256": format!("{:x}", digest.finalize()),
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

struct BoundedOutput(Vec<u8>);

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let new_length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("output length overflow"))?;
        if new_length > MAX_OUTPUT_BYTES {
            return Err(std::io::Error::other("output byte bound exceeded"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
