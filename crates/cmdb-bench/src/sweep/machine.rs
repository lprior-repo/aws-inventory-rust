use sha2::{Digest, Sha256};
use std::path::Path;
pub(crate) fn headers(workspace: &Path) -> Vec<(String, String)> {
    let read = |path| match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => "unavailable".into(),
    };
    let cpu = read("/proc/cpuinfo");
    let memory = read("/proc/meminfo");
    let model = cpu
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.trim() == "model name")
                .map(|(_, value)| value.trim())
        })
        .map_or("unavailable", |value| value);
    let memory = memory
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .map_or("unavailable", str::trim);
    let cores = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    let (commit, commit_state) = revision(workspace);
    vec![
        ("machine".into(), model.into()),
        ("cores".into(), cores.to_string()),
        ("memory".into(), memory.into()),
        (
            "os".into(),
            format!(
                "{} {}",
                read("/proc/sys/kernel/ostype").trim(),
                read("/proc/sys/kernel/osrelease").trim()
            ),
        ),
        ("commit".into(), commit),
        ("commit_state".into(), commit_state),
        (
            "shared_machine".into(),
            "collector, emulator, graph and PostgreSQL share this host".into(),
        ),
    ]
}
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    base16ct::lower::encode_string(&Sha256::digest(bytes))
}
fn revision(workspace: &Path) -> (String, String) {
    let git = |arguments: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(workspace)
            .args(arguments)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
    };
    let commit = git(&["rev-parse", "HEAD"])
        .filter(|text| !text.trim().is_empty())
        .map_or_else(|| "unavailable".into(), |text| text.trim().to_owned());
    let state = git(&["status", "--porcelain"]).map_or("unavailable", |text| {
        if text.is_empty() { "clean" } else { "dirty" }
    });
    (commit, state.into())
}
#[cfg(test)]
mod tests {
    #[test]
    fn digest_is_standard_sha256() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn failed_git_command_is_unavailable_and_dirty_tree_is_recorded()
    -> Result<(), Box<dyn std::error::Error>> {
        let work = tempfile::tempdir()?;
        let headers = super::headers(work.path());
        assert!(headers.contains(&("commit".into(), "unavailable".into())));
        let initialized = std::process::Command::new("git")
            .arg("-C")
            .arg(work.path())
            .arg("init")
            .output()?;
        assert!(initialized.status.success());
        std::fs::write(work.path().join("dirty"), "untracked")?;
        let headers = super::headers(work.path());
        assert!(headers.contains(&("commit_state".into(), "dirty".into())));
        Ok(())
    }
}
