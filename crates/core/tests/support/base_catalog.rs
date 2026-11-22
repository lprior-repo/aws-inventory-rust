//! The generated AWS catalog. `scripts/bootstrap-catalog.sh` produces it, so it is read at run
//! time: `inventory-core` is a dependency of `catalog-tool`, which generates this file, and must
//! therefore build without it.

use std::sync::LazyLock;

static BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog/aws-api.catalog");
    match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => panic!("cannot read {path}: {error}; run scripts/bootstrap-catalog.sh"),
    }
});

pub fn base() -> &'static [u8] {
    &BYTES
}
