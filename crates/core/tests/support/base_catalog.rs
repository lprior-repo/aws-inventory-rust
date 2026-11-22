//! The generated AWS catalog. `scripts/bootstrap-catalog.sh` produces it, so it is read at run
//! time: `inventory-core` is a dependency of `catalog-tool`, which generates this file, and must
//! therefore build without it.

use std::sync::LazyLock;

static BYTES: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog/aws-api.catalog");
    std::fs::read(path)
        .map_err(|error| format!("cannot read {path}: {error}; run scripts/bootstrap-catalog.sh"))
});

pub fn base() -> Result<&'static [u8], String> {
    BYTES.as_ref().map(Vec::as_slice).map_err(Clone::clone)
}
