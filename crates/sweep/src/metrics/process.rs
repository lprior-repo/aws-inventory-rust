use std::fs::File;
use std::io::Read;

const MAX_PROC_BYTES: u64 = 65_536;

pub(super) fn snapshot() -> Option<(u64, u64, u64)> {
    let (user, system) = cpu(&read("/proc/self/stat")?)?;
    Some((user, system, peak_rss(&read("/proc/self/status")?)?))
}

fn read(path: &str) -> Option<String> {
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(MAX_PROC_BYTES.checked_add(1)?)
        .read_to_string(&mut text)
        .ok()?;
    (u64::try_from(text.len()).ok()? <= MAX_PROC_BYTES).then_some(text)
}

pub(super) fn cpu(text: &str) -> Option<(u64, u64)> {
    let (_, fields) = text.rsplit_once(')')?;
    let mut fields = fields.split_ascii_whitespace();
    let user = fields.nth(11)?.parse::<u64>().ok()?.checked_mul(10)?;
    let system = fields.next()?.parse::<u64>().ok()?.checked_mul(10)?;
    Some((user, system))
}

pub(super) fn peak_rss(text: &str) -> Option<u64> {
    let value = text.lines().find_map(|line| line.strip_prefix("VmHWM:"))?;
    let mut fields = value.split_ascii_whitespace();
    let value = fields.next()?.parse().ok()?;
    (fields.next()? == "kB" && fields.next().is_none()).then_some(value)
}
