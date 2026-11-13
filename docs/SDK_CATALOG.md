# Official AWS SDK catalog

## What the catalog means

`crates/sdk-catalog` records every AWS service SDK listed in the official SDK
release snapshot. It is a dependency and provenance catalog, not an assertion
that every service has an implemented inventory collector.

The initial snapshot follows AWS SDK for Rust release `release-2026-11-11` and
contains **437 service crates**. The official generated service index and
`versions.toml` agree on those service names. Configuration, credentials,
runtime, and Smithy support crates are separate and are resolved transitively;
unpublished fuzz packages in the upstream workspace are not service SDKs.

Collector coverage is determined by `crates/aws` and the scan's recorded
coverage outcomes. Adding a crate does not supply pagination policy, permissions,
normalization, relationship extraction, rate-bucket configuration, tests, or
coverage accounting for that service.

## Source and reproducibility

The catalog retains three kinds of evidence:

1. `upstream-versions.toml` is the exact original official version manifest.
2. `sdk-services.json` records every crate's exact pinned version, official
   source origin, source/model fingerprints, registry checksum, registry URL,
   required Rust version when published, and registry yanked status at check
   time. Its metadata includes the official release/revision, check time,
   source manifest SHA-256, and service-index cross-check result.
3. The workspace `Cargo.lock` records the dependency resolution, including
   transitive packages. `cargo fetch --locked` reproduces the locked downloads.

Every SDK uses an exact `=version` requirement, not a floating major-version
range. Registry validation rejects a missing or yanked version. The generator
also records a newer registry version if one exists, rather than silently
mixing a newer package with an older source/model fingerprint.

Branch and release-tag inputs are resolved to a full Git commit before any
source URL is recorded. The initial source commit is
`0f29bbb75b57d337bd00cf46c0d29d6988e2c372`. The retained manifest checksum
protects offline reproduction even if an upstream branch or tag later moves.

## Refresh and offline checks

The generator uses Python 3.11 or later and only the standard library:

```bash
# Resolve the current official commit and refresh registry metadata.
python scripts/generate_sdk_catalog.py --refresh

# Reproduce a specified official release's service set.
python scripts/generate_sdk_catalog.py --refresh --revision release-2026-11-11

# Rebuild Cargo.toml and lib.rs from the retained local snapshot, offline.
python scripts/generate_sdk_catalog.py

# Verify the snapshot and generated files, offline, without modifying them.
python scripts/generate_sdk_catalog.py --check
```

Refresh has a hard ceiling of 1,024 services, at most eight concurrent registry
requests, three attempts per fetch, a 30-second per-request timeout, and an
8 MB per-response bound. A failed registry validation prevents publishing a
partial new snapshot. Requests use only public official SDK/registry endpoints;
the generator never loads AWS account credentials or enumerates account data.

## Fetching versus compiling

The catalog's default feature set is empty. Every service has a named feature
and a corresponding conditional crate re-export. For example, the `ec2`
feature exposes `inventory_sdk_catalog::ec2`.

For the initial snapshot, all **437** exact SDK archives were downloaded and
their SHA-256 hashes matched the registry records. Their combined compressed
size was **241,799,148 bytes**; transitive packages and extracted source files
consume additional space. This is download verification, not an all-services
compilation result.

```bash
# Download the entire locked workspace dependency set, including the SDK pins.
cargo fetch --locked

# Confirm all 437 SDK archives exist locally and match registry checksums.
python scripts/generate_sdk_catalog.py --check --verify-cache

# Compile the catalog's metadata surface without compiling all SDKs.
cargo check -p inventory-sdk-catalog --no-default-features

# Compile a selected SDK when implementing its collector.
cargo check -p inventory-sdk-catalog --features sts

# Validate the feature graph without compiling all SDK libraries.
cargo metadata --format-version 1 --all-features --no-deps
```

`all-services` is an explicit opt-in feature selecting every service dependency.
An all-services build is large and should be scheduled with a stated memory,
CPU, and time budget. Normal inventory builds compile the concrete adapters
they use. Downloaded crates and a valid feature graph do not establish that an
all-services build has completed or that every SDK combination was tested.

The catalog is marked `publish = false`. Its 437 individual service features
exceed crates.io's usual 300-feature publishing limit; that limit does not
prevent using this local workspace package. If the catalog is published later,
split it into smaller catalogs or obtain a registry exception before release.

## Verification boundaries

Source checks and focused collector builds should identify the SDK catalog
explicitly. Avoid accidentally enabling every SDK in an otherwise small CI
job through a blanket workspace `--all-features` flag. A practical split is:

```bash
cargo check --workspace --exclude inventory-sdk-catalog --all-features
cargo check -p inventory-sdk-catalog --no-default-features
python scripts/generate_sdk_catalog.py --check
```

The first command still builds all features of the implemented application and
adapter crates. The second checks the catalog metadata library. Any optional
service feature actually used by a collector requires its own compile/test
evidence. Full all-services compilation is a separate opt-in job.

## Primary references

- [AWS's generated SDK service catalog](https://awslabs.github.io/aws-sdk-rust/)
- [Official AWS SDK for Rust repository](https://github.com/awslabs/aws-sdk-rust)
- [Official AWS SDK releases](https://github.com/awslabs/aws-sdk-rust/releases)
- [AWS SDK crate fundamentals](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/fundamentals.html)
- [Cargo feature and optional-dependency semantics](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo fetch and locked dependency downloads](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)

The precise immutable source URL and package-specific registry URLs are stored
with the snapshot rather than inferred from these overview links.
