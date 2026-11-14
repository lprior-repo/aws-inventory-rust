#[cfg(unix)]
mod fifo;

use std::io::{Read, Seek};

use serde_json::Value;

use super::{HarnessError, MAX_DOCUMENT_BYTES, read_json, read_json_with_limit};
use crate::files::{ReadError, read_limited};
use crate::provenance::sha256_hex;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_exact_byte_limit_when_json_is_read_then_value_and_raw_digest_are_returned() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("exact.json");
    std::fs::write(&path, b"true")?;
    let (value, digest) = read_json_with_limit(&path, 4)?;
    assert_eq!(value, Value::Bool(true));
    assert_eq!(digest, sha256_hex(b"true"));
    Ok(())
}

#[test]
fn given_one_byte_over_limit_when_json_is_read_then_size_error_is_preserved() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("oversized.json");
    std::fs::write(&path, b"true ")?;
    let result = read_json_with_limit(&path, 4);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "JSON document exceeds its byte budget: oversized.json"));
    Ok(())
}

#[test]
fn given_large_file_when_bounded_reader_rejects_then_only_sentinel_byte_is_consumed() -> TestResult
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("large.json");
    std::fs::write(&path, b"true                         ")?;
    let mut file = std::fs::File::open(&path)?;
    let result = read_limited(&mut file, 4);
    assert!(matches!(result, Err(ReadError::TooLarge)));
    assert_eq!(file.stream_position()?, 5);
    Ok(())
}

#[test]
fn given_invalid_json_within_limit_when_read_then_json_error_is_preserved() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("invalid.json");
    std::fs::write(&path, b"tru")?;
    let result = read_json_with_limit(&path, 3);
    assert!(matches!(result, Err(HarnessError::Json(error)) if error.is_eof()));
    Ok(())
}

#[test]
fn given_missing_file_when_json_is_read_then_io_error_is_preserved() -> TestResult {
    let directory = tempfile::tempdir()?;
    let result = read_json(&directory.path().join("missing.json"));
    assert!(matches!(result, Err(HarnessError::Io(error))
        if error.kind() == std::io::ErrorKind::NotFound));
    Ok(())
}

#[test]
fn given_zero_limit_when_nonempty_json_is_read_then_size_error_precedes_decoding() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("zero.json");
    std::fs::write(&path, b"0")?;
    let result = read_json_with_limit(&path, 0);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "JSON document exceeds its byte budget: zero.json"));
    Ok(())
}

#[test]
fn given_empty_file_with_zero_limit_when_json_is_read_then_json_error_is_returned() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("empty.json");
    std::fs::write(&path, b"")?;
    let result = read_json_with_limit(&path, 0);
    assert!(matches!(result, Err(HarnessError::Json(error)) if error.is_eof()));
    Ok(())
}

struct SentinelReader {
    consumed: usize,
}

impl Read for SentinelReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.consumed >= 5 {
            return Err(std::io::Error::other("read beyond sentinel"));
        }
        let byte = buffer
            .first_mut()
            .ok_or_else(|| std::io::Error::other("missing read buffer"))?;
        *byte = b' ';
        self.consumed = self
            .consumed
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("consumption overflow"))?;
        Ok(1)
    }
}

struct FailedReader;

impl Read for FailedReader {
    fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }
}

#[test]
fn given_public_limit_exceeded_when_json_is_read_then_size_error_precedes_decoding() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("public-oversized.json");
    let length = u64::try_from(MAX_DOCUMENT_BYTES)?
        .checked_add(1)
        .ok_or("document length overflow")?;
    std::fs::File::create(&path)?.set_len(length)?;
    let result = read_json(&path);
    assert!(matches!(result, Err(HarnessError::Invalid(reason))
        if reason == "JSON document exceeds its byte budget: public-oversized.json"));
    Ok(())
}

#[test]
fn given_one_byte_reads_when_limit_is_exceeded_then_reader_stops_at_sentinel() {
    let mut reader = SentinelReader { consumed: 0 };
    let result = read_limited(&mut reader, 4);
    assert!(matches!(result, Err(ReadError::TooLarge)));
    assert_eq!(reader.consumed, 5);
}

#[test]
fn given_read_time_failure_when_reader_runs_then_io_kind_is_preserved() {
    let result = read_limited(FailedReader, 4);
    assert!(matches!(result, Err(ReadError::Io(error))
        if error.kind() == std::io::ErrorKind::PermissionDenied));
}

#[test]
fn given_directory_when_json_is_read_then_read_time_io_kind_is_preserved() -> TestResult {
    let directory = tempfile::tempdir()?;
    let result = read_json(directory.path());
    assert!(matches!(result, Err(HarnessError::Io(error))
        if error.kind() == std::io::ErrorKind::IsADirectory));
    Ok(())
}

#[test]
fn given_json_whitespace_when_read_then_digest_uses_original_bytes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("whitespace.json");
    let bytes = b" \n true \t\n";
    std::fs::write(&path, bytes)?;
    let (value, digest) = read_json(&path)?;
    assert_eq!(value, Value::Bool(true));
    assert_eq!(digest, sha256_hex(bytes));
    assert_ne!(digest, sha256_hex(b"true"));
    Ok(())
}
