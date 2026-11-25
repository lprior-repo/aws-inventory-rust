use std::path::Path;

const REQUIRED: &[&str] = &[
    "catalog/aws-api.catalog",
    "catalog/aws-models.report",
    "catalog/cloudcontrol-universal.json",
    "catalog/iam-read-only-policy.json",
];

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in REQUIRED {
        let path = root.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        assert!(
            path.is_file(),
            "missing generated catalog file {}; run scripts/bootstrap-catalog.sh",
            path.display()
        );
    }
}
