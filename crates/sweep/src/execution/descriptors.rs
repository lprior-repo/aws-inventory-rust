//! Connection bound taken from the process's open-file limit.

/// Descriptors kept back for the store, the log and name lookups.
pub(super) const RESERVED_DESCRIPTORS: u64 = 256;
/// Connection bound used when the limit cannot be read.
pub(super) const FALLBACK_CONNECTIONS: usize = 256;

/// The soft limit on open files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SoftLimit {
    Unlimited,
    Files(u64),
}

/// Reads the soft `Max open files` value from the text of `/proc/self/limits`.
pub(super) fn parse_soft_open_files(limits: &str) -> Option<SoftLimit> {
    let rest = limits
        .lines()
        .find_map(|line| line.strip_prefix("Max open files"))?;
    match rest.split_whitespace().next()? {
        "unlimited" => Some(SoftLimit::Unlimited),
        files => files.parse().ok().map(SoftLimit::Files),
    }
}

/// The soft limit of this process, or `None` when it cannot be read.
pub(super) fn soft_open_files() -> Option<SoftLimit> {
    std::fs::read_to_string("/proc/self/limits")
        .ok()
        .as_deref()
        .and_then(parse_soft_open_files)
}

/// Most connections the transport may open for `concurrency` pages in flight.
pub(super) fn connection_bound(concurrency: usize, limit: Option<SoftLimit>) -> usize {
    let cap = match limit {
        None => FALLBACK_CONNECTIONS,
        Some(SoftLimit::Unlimited) => usize::MAX,
        Some(SoftLimit::Files(n)) => {
            let room = n.saturating_sub(RESERVED_DESCRIPTORS);
            usize::try_from(room).unwrap_or(usize::MAX)
        }
    };
    concurrency.min(cap).max(1)
}

#[cfg(test)]
#[path = "descriptors/tests.rs"]
mod tests;
