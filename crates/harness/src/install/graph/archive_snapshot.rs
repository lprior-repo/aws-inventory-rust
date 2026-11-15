use std::fs::File;
use std::io::{Read, Seek, Write};

use rustix::fs::{MemfdFlags, SealFlags};
use sha2::{Digest, Sha256};

use super::{MAX_DOWNLOAD, reject};
use crate::HarnessError;

// This snapshot has no mutable pathname and seals cover every writable descriptor alias.
pub(super) fn capture(input: File) -> Result<File, HarnessError> {
    let metadata = input.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_DOWNLOAD {
        return Err(reject("input type or download byte bound"));
    }
    let flags = MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING;
    let mut snapshot = File::from(
        rustix::fs::memfd_create("graph-archive", flags).map_err(std::io::Error::from)?,
    );
    let size = std::io::copy(&mut input.take(MAX_DOWNLOAD.saturating_add(1)), &mut snapshot)?;
    if size > MAX_DOWNLOAD {
        return Err(reject("download byte bound"));
    }
    // Seal before hashing: neither the original writer nor a proc-FD alias can alter later reads.
    let seals = SealFlags::WRITE | SealFlags::SHRINK | SealFlags::GROW | SealFlags::SEAL;
    rustix::fs::fcntl_add_seals(&snapshot, seals).map_err(std::io::Error::from)?;
    snapshot.rewind()?;
    Ok(snapshot)
}

pub(super) fn verify(file: &mut File, expected: &str) -> Result<(), HarnessError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_DOWNLOAD {
        return Err(reject("input type or download byte bound"));
    }
    file.rewind()?;
    let mut hash = HashWriter(Sha256::new());
    let size = std::io::copy(&mut file.take(MAX_DOWNLOAD.saturating_add(1)), &mut hash)?;
    let digest = format!("{:x}", base16ct::HexDisplay(&hash.0.finalize()));
    if size > MAX_DOWNLOAD || digest != expected {
        return Err(reject("digest or download byte bound"));
    }
    Ok(())
}

struct HashWriter(Sha256);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
