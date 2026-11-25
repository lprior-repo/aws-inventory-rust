pub(super) const RESERVED_DESCRIPTORS: u64 = 256;
pub(super) const FALLBACK_CONNECTIONS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SoftLimit {
    Unlimited,
    Files(u64),
}

pub(super) fn parse_soft_open_files(limits: &str) -> Option<SoftLimit> {
    let rest = limits
        .lines()
        .find_map(|line| line.strip_prefix("Max open files"))?;
    match rest.split_whitespace().next()? {
        "unlimited" => Some(SoftLimit::Unlimited),
        files => files.parse().ok().map(SoftLimit::Files),
    }
}

pub(super) fn soft_open_files() -> Option<SoftLimit> {
    std::fs::read_to_string("/proc/self/limits")
        .ok()
        .as_deref()
        .and_then(parse_soft_open_files)
}

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
