use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::HarnessError;

pub(super) struct DatabaseName(String);

impl DatabaseName {
    pub(super) fn at(seconds: u32) -> Self {
        let random = RandomState::new().build_hasher().finish();
        Self(format!(
            "inv_t_{seconds:08x}_{:x}_{:08x}",
            std::process::id(),
            random & 0xffff_ffff
        ))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(super) fn now() -> Result<u32, HarnessError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| HarnessError::invalid("clock precedes Unix epoch"))?
        .as_secs();
    u32::try_from(seconds).map_err(|_| HarnessError::invalid("database timestamp exceeds u32"))
}

pub(super) fn generate() -> Result<DatabaseName, HarnessError> {
    Ok(DatabaseName::at(now()?))
}

pub(super) fn created_at(name: &str) -> Option<u32> {
    let mut parts = name.strip_prefix("inv_t_")?.split('_');
    let time = parts.next()?;
    let pid = parts.next()?;
    let random = parts.next()?;
    if time.len() != 8
        || pid.is_empty()
        || pid.len() > 8
        || random.len() != 8
        || parts.next().is_some()
        || !name.bytes().all(valid_byte)
    {
        return None;
    }
    u32::from_str_radix(pid, 16).ok()?;
    u32::from_str_radix(random, 16).ok()?;
    u32::from_str_radix(time, 16).ok()
}

fn valid_byte(byte: u8) -> bool {
    byte.is_ascii_digit()
        || (b'a'..=b'f').contains(&byte)
        || matches!(byte, b'i' | b'n' | b'v' | b't' | b'_')
}
