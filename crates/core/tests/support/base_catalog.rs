use std::sync::LazyLock;

static BYTES: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog/aws-api.catalog");
    std::fs::read(path)
        .map_err(|error| format!("cannot read {path}: {error}; run scripts/bootstrap-catalog.sh"))
});

pub fn base() -> Result<&'static [u8], String> {
    BYTES.as_ref().map(Vec::as_slice).map_err(Clone::clone)
}
