//! Bounded source-tree fingerprints shared by every inventory row.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// Maximum number of files one crate fingerprint may cover.
pub const MAX_TREE_FILES: usize = 20_000;
/// Maximum total bytes one crate fingerprint may cover.
pub const MAX_TREE_BYTES: u64 = 512 * 1024 * 1024;

/// Lowercase hexadecimal SHA-256 of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(bytes);
    format!("{:x}", base16ct::HexDisplay(&digest))
}

/// Lowercase hexadecimal SHA-256 of a file's bytes.
pub fn file_sha256(path: &Path) -> Result<String, Error> {
    let bytes = fs::read(path).map_err(|source| Error::io(path, source))?;
    Ok(sha256_hex(&bytes))
}

/// SHA-256 over every file in a crate tree, sorted by relative slash path.
///
/// The encoding is the canonical bounded form used by the harness: a JSON
/// array of `{"file", "sha256"}` rows sorted by file name, so the digest never
/// depends on directory iteration order.
pub fn tree_digest(root: &Path) -> Result<String, Error> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    if files.is_empty() {
        return Err(Error::EmptyTree {
            path: root.to_path_buf(),
        });
    }
    files.sort();
    let mut total = 0_u64;
    let mut rows = Vec::with_capacity(files.len());
    for path in &files {
        let metadata = fs::metadata(path).map_err(|source| Error::io(path, source))?;
        total = total.saturating_add(metadata.len());
        if total > MAX_TREE_BYTES {
            return Err(Error::TreeBudget {
                path: root.to_path_buf(),
            });
        }
        let relative = path.strip_prefix(root).map_err(|_| Error::TreeEscape {
            path: root.to_path_buf(),
        })?;
        rows.push((slash_path(relative), file_sha256(path)?));
    }
    Ok(sha256_hex(canonical_rows(&rows).as_bytes()))
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
    let entries = fs::read_dir(directory).map_err(|source| Error::io(directory, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::io(directory, source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| Error::io(&path, source))?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
            if files.len() > MAX_TREE_FILES {
                return Err(Error::TreeBudget {
                    path: directory.to_path_buf(),
                });
            }
        }
    }
    Ok(())
}

fn canonical_rows(rows: &[(String, String)]) -> String {
    let mut out = String::from("[");
    for (index, (file, hash)) in rows.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str("{\"file\":");
        push_json_string(&mut out, file);
        out.push_str(",\"sha256\":");
        push_json_string(&mut out, hash);
        out.push('}');
    }
    out.push(']');
    out
}

fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => {
                let code = u32::from(other);
                if code < 0x20 {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    out.push(other);
                }
            }
        }
    }
    out.push('"');
}

fn slash_path(relative: &Path) -> String {
    let mut text = String::new();
    for component in relative.components() {
        if !text.is_empty() {
            text.push('/');
        }
        text.push_str(&component.as_os_str().to_string_lossy());
    }
    text
}
