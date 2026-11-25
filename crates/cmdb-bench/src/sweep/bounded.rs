//! Bounded reads of collector reports and individual seed recipes.
use std::{io::Read, path::Path};
use tokio::io::AsyncReadExt;
pub(super) const REPORT_LIMIT: u64 = 16_777_216;
pub(super) const RECIPE_LIMIT: u64 = 1_048_576;
pub(super) async fn read(path: &Path, limit: u64) -> Result<String, String> {
    let file = tokio::fs::File::open(path).await.map_err(text)?;
    let mut bytes = Vec::new();
    file.take(limit.checked_add(1).ok_or("read limit overflow")?)
        .read_to_end(&mut bytes)
        .await
        .map_err(text)?;
    decode(bytes, path, limit)
}
pub(super) fn read_sync(path: &Path, limit: u64) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(text)?;
    let mut bytes = Vec::new();
    file.take(limit.checked_add(1).ok_or("read limit overflow")?)
        .read_to_end(&mut bytes)
        .map_err(text)?;
    decode(bytes, path, limit)
}
fn decode(bytes: Vec<u8>, path: &Path, limit: u64) -> Result<String, String> {
    if u64::try_from(bytes.len()).map_err(text)? > limit {
        return Err(format!(
            "{} exceeds {limit}-byte read limit",
            path.display()
        ));
    }
    String::from_utf8(bytes).map_err(text)
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
