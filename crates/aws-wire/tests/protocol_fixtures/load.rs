//! Runtime loader for the Smithy protocol-test fixtures. The seven JSON files
//! are generated and gitignored; scripts/bootstrap-protocol-tests.sh writes them.
//! Included by path from both unit tests (src) and integration tests.

use std::{error::Error, fs, io, path::PathBuf};

pub(crate) fn load(name: &str) -> Result<String, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/protocol-tests")
        .join(name);
    fs::read_to_string(&path).map_err(|source| -> Box<dyn Error> {
        let reason = if source.kind() == io::ErrorKind::NotFound {
            format!(
                "missing Smithy protocol fixture {}; run scripts/bootstrap-protocol-tests.sh",
                path.display()
            )
        } else {
            format!(
                "cannot read Smithy protocol fixture {}: {source}",
                path.display()
            )
        };
        Box::new(io::Error::new(source.kind(), reason))
    })
}
