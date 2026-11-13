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
    format!("{:x}", base16ct::HexDisplay(&Sha256::digest(bytes)))
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

#[path = "audit/services.rs"]
mod services;
use services::audit_service;
#[path = "audit/source_files.rs"]
mod source_files;
use source_files::{
    declared_modules, module_name, regular_rs_files, source_digest_add, source_text,
    valid_component,
};
