use super::archive::{Entry, Kind, parse, validate};
use super::download::existing;
use super::{MAX_EXTRACT, MAX_MEMBERS, valid_jobs};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn file(name: &str, size: u64) -> Entry {
    Entry {
        kind: Kind::File,
        size,
        name: name.to_owned(),
    }
}

fn link(name: &str, target: &str) -> Entry {
    Entry {
        kind: Kind::Symlink(target.to_owned()),
        size: 0,
        name: name.to_owned(),
    }
}

#[test]
fn jobs_are_bounded_to_one_through_eight() {
    assert!(!valid_jobs(0));
    assert!((1..=8).all(valid_jobs));
    assert!(!valid_jobs(9));
}

#[test]
fn verbose_listing_parses_files_directories_and_expected_links() -> TestResult {
    let listing = "drwxr-xr-x  0 root   root        0 Nov 12  2026 redis-8.10.3/\n\
-rw-r--r--  0 root   root     1234 Nov 12  2026 redis-8.10.3/src/server.c\n\
lrwxrwxrwx  0 root   root        0 Nov 12  2026 redis-8.10.3/modules/redisearch/src/CLAUDE.md -> AGENTS.md\n";
    let entries = parse(listing)?;
    assert_eq!(
        entries,
        vec![
            Entry {
                kind: Kind::Directory,
                size: 0,
                name: "redis-8.10.3".to_owned()
            },
            file("redis-8.10.3/src/server.c", 1234),
            link("redis-8.10.3/modules/redisearch/src/CLAUDE.md", "AGENTS.md"),
        ]
    );
    validate(&entries)?;
    Ok(())
}

#[test]
fn hard_links_devices_and_malformed_lines_are_rejected() {
    let hard = "hrw-r--r--  0 root root 0 Nov 12  2026 redis-8.10.3/a link to redis-8.10.3/b";
    let device = "crw-r--r--  0 root root 0 Nov 12  2026 redis-8.10.3/dev";
    assert!(parse(hard).is_err());
    assert!(parse(device).is_err());
    assert!(parse("-rw-r--r-- 0 root").is_err());
}

#[test]
fn traversal_absolute_foreign_root_and_duplicates_are_rejected() {
    let bad = [
        vec![file("redis-8.10.3/../outside", 1)],
        vec![file("/etc/passwd", 1)],
        vec![file("other-1.0/file", 1)],
        vec![file("redis-8.10.3/a", 1), file("redis-8.10.3/a", 1)],
        vec![file("", 1)],
    ];
    assert!(bad.iter().all(|entries| validate(entries).is_err()));
}

#[test]
fn only_the_two_reviewed_symlinks_are_accepted() {
    let skills = "redis-8.10.3/modules/redisearch/src/.claude/skills";
    assert!(validate(&[link(skills, "../.skills")]).is_ok());
    assert!(validate(&[link(skills, "../../outside")]).is_err());
    assert!(validate(&[link("redis-8.10.3/unreviewed", "a")]).is_err());
    let below = [
        link(skills, "../.skills"),
        file(&format!("{skills}/escape"), 1),
    ];
    assert!(validate(&below).is_err());
}

#[test]
fn byte_and_member_bounds_are_enforced() {
    assert!(validate(&[file("redis-8.10.3/big", MAX_EXTRACT)]).is_ok());
    assert!(validate(&[file("redis-8.10.3/big", MAX_EXTRACT + 1)]).is_err());
    let halves = [
        file("redis-8.10.3/a", MAX_EXTRACT / 2 + 1),
        file("redis-8.10.3/b", MAX_EXTRACT / 2 + 1),
    ];
    assert!(validate(&halves).is_err());
    assert!(validate(&[]).is_err());
    let many: Vec<Entry> = (0..=MAX_MEMBERS)
        .map(|index| file(&format!("redis-8.10.3/f{index}"), 0))
        .collect();
    assert!(validate(&many).is_err());
}

#[test]
fn existing_download_with_foreign_digest_is_rejected_and_missing_is_absent() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("pinned");
    assert!(!existing(&path, "00")?);
    std::fs::write(&path, b"not the pinned content")?;
    assert!(existing(&path, "00").is_err());
    Ok(())
}
