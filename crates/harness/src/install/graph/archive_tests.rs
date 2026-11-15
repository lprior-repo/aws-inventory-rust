#[path = "archive_acquisition_tests.rs"]
pub(super) mod acquisition;

#[path = "graph_process_tests.rs"]
pub(super) mod lifecycle;
use super::*;
use std::io::Write;
use std::ops::Range;
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Fixture = (tempfile::TempDir, PathBuf, File);

struct Member<'a> {
    name: &'a str,
    kind: u8,
    target: &'a str,
    bytes: &'a [u8],
    size: u64,
}

fn member<'a>(name: &'a str, bytes: &'a [u8]) -> Member<'a> {
    Member {
        name,
        kind: b'0',
        target: "",
        bytes,
        size: u64::try_from(bytes.len()).map_or(u64::MAX, |n| n),
    }
}

fn link<'a>(name: &'a str, kind: u8, target: &'a str) -> Member<'a> {
    Member { name, kind, target, bytes: b"", size: 0 }
}

fn put(header: &mut [u8], range: Range<usize>, value: &[u8]) -> TestResult {
    let destination = header.get_mut(range).ok_or("invalid fixture range")?;
    if destination.len() != value.len() {
        return Err("invalid fixture value length".into());
    }
    destination.copy_from_slice(value);
    Ok(())
}

fn text(header: &mut [u8], start: usize, value: &str) -> TestResult {
    put(header, start..start.saturating_add(value.len()), value.as_bytes())
}

fn header(value: &Member<'_>) -> Result<[u8; 512], Box<dyn std::error::Error>> {
    let mut header = [0_u8; 512];
    text(&mut header, 0, value.name)?;
    text(&mut header, 100, "0000755\0")?;
    text(&mut header, 108, "0000000\0")?;
    text(&mut header, 116, "0000000\0")?;
    text(&mut header, 124, &format!("{:011o}\0", value.size))?;
    text(&mut header, 136, "00000000000\0")?;
    text(&mut header, 148, "        ")?;
    put(&mut header, 156..157, &[value.kind])?;
    text(&mut header, 157, value.target)?;
    text(&mut header, 257, "ustar\0")?;
    text(&mut header, 263, "00")?;
    let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
    text(&mut header, 148, &format!("{checksum:06o}\0 "))?;
    Ok(header)
}

fn archive(path: &Path, members: &[Member<'_>]) -> TestResult {
    let mut output = File::create(path)?;
    members.iter().try_for_each(|value| -> TestResult {
        output.write_all(&header(value)?)?;
        output.write_all(value.bytes)?;
        let padding = 512_usize.saturating_sub(value.bytes.len() % 512) % 512;
        output.write_all(&vec![0; padding])?;
        Ok(())
    })?;
    output.write_all(&[0; 1024])?;
    Ok(())
}

fn fixture(members: &[Member<'_>]) -> Result<Fixture, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("input.tar");
    archive(&path, members)?;
    let expected = crate::provenance::sha256_hex(&std::fs::read(&path)?);
    let mut file = File::open(path)?;
    verify(&mut file, &expected)?;
    let source = directory.path().join("source");
    Ok((directory, source, file))
}

fn rejected(members: &[Member<'_>]) -> TestResult {
    let (given, source, mut input) = fixture(members)?;
    let when = extract_open(&mut input, &source);
    assert!(when.is_err());
    assert!(!source.try_exists()?);
    assert!(!given.path().join("escaped").try_exists()?);
    Ok(())
}

#[test]
fn given_aliases_when_extracted_then_duplicate_paths_fail() -> TestResult {
    ["redis-8.10.3//a", "redis-8.10.3/./a", "redis-8.10.3/a/"]
        .iter()
        .try_for_each(|alias| {
            rejected(&[member("redis-8.10.3/a", b"first"), member(alias, b"second")])
        })
}

#[test]
fn given_link_descendant_aliases_when_extracted_then_source_is_cleaned() -> TestResult {
    let skills = "redis-8.10.3/modules/redisearch/src/.claude/skills";
    ["escape", "/escape", "./escape"]
        .iter()
        .try_for_each(|tail| {
            let name = format!("{skills}/{tail}");
            rejected(&[link(skills, b'2', "../.skills"), member(&name, b"escape")])
        })
}

#[test]
fn given_hostile_names_when_extracted_then_nothing_escapes() -> TestResult {
    [
        "../escaped", "/escaped", "redis-8.10.3/../escaped",
        "other/escaped", "redis-8.10.3/a\\b", "redis-8.10.3/a\nb",
    ].iter().try_for_each(|name| rejected(&[member(name, b"bad")]))?;
    rejected(&[member("redis-8.10.3/a", b"one"), member("redis-8.10.3/a", b"two")])
}

#[test]
fn given_unapproved_links_and_devices_when_extracted_then_source_is_cleaned() -> TestResult {
    [b'1', b'2', b'3', b'4', b'6'].iter().try_for_each(|kind| {
        rejected(&[link("redis-8.10.3/link", *kind, "../../escaped")])
    })
}

#[test]
fn given_spaces_when_extracted_then_distinct_names_are_preserved() -> TestResult {
    let entries = [
        member("redis-8.10.3/a  b", b"double"),
        member("redis-8.10.3/a b", b"single"),
        member("redis-8.10.3/a -> b", b"arrow"),
    ];
    let (_given, source, mut input) = fixture(&entries)?;
    let when = extract_open(&mut input, &source);
    let retained = when?;
    assert_eq!(std::fs::read(retained.anchor().join("a  b"))?, b"double");
    assert_eq!(std::fs::read(retained.anchor().join("a b"))?, b"single");
    assert_eq!(std::fs::read(retained.anchor().join("a -> b"))?, b"arrow");
    Ok(())
}

#[test]
fn given_reviewed_links_when_extracted_then_targets_are_exact() -> TestResult {
    let entries = EXPECTED_LINKS
        .iter()
        .map(|(name, target)| link(name, b'2', target))
        .collect::<Vec<_>>();
    let (_given, source, mut input) = fixture(&entries)?;
    let when = extract_open(&mut input, &source);
    when?;
    EXPECTED_LINKS
        .iter()
        .try_for_each(|(name, target)| -> TestResult {
            let relative = Path::new(name).strip_prefix(REDIS_ROOT)?;
            assert_eq!(
                std::fs::read_link(source.join(relative))?,
                Path::new(target)
            );
            Ok(())
        })
}

#[test]
fn given_replaced_path_when_extracted_then_verified_handle_is_used() -> TestResult {
    let (given, source, mut input) = fixture(&[member("redis-8.10.3/kept", b"verified")])?;
    let path = given.path().join("input.tar");
    std::fs::rename(&path, given.path().join("original.tar"))?;
    archive(&path, &[member("../escaped", b"replaced")])?;
    let when = extract_open(&mut input, &source);
    when?;
    assert_eq!(std::fs::read(source.join("kept"))?, b"verified");
    assert!(!given.path().join("escaped").try_exists()?);
    Ok(())
}

#[test]
fn given_foreign_source_when_claimed_then_existing_tree_is_untouched() -> TestResult {
    let (_given, source, mut input) = fixture(&[member("redis-8.10.3/a", b"new")])?;
    std::fs::create_dir(&source)?;
    std::fs::write(source.join("foreign"), b"keep")?;
    let when = extract_open(&mut input, &source);
    assert!(when.is_err());
    assert_eq!(std::fs::read(source.join("foreign"))?, b"keep");
    assert!(!source.join("a").try_exists()?);
    Ok(())
}

#[test]
fn given_wrong_digest_when_public_extract_runs_then_source_is_not_claimed() -> TestResult {
    let (given, source, _input) = fixture(&[member("redis-8.10.3/a", b"untrusted")])?;
    let when = extract(&given.path().join("input.tar"), &source);
    assert!(matches!(when, Err(HarnessError::Invalid(error)) if error == "unexpected Redis archive digest or download byte bound"));
    assert!(!source.try_exists()?);
    Ok(())
}

#[test]
fn given_oversized_output_when_read_then_listing_is_bounded() -> TestResult {
    let (_directory, _source, mut input) = fixture(&[member("redis-8.10.3/a", b"safe")])?;
    let count = MAX_LISTING.saturating_add(1).to_string();
    let when = run(Command::new("head").args(["-c", &count, "/dev/zero"]), &mut input);
    assert!(matches!(when, Err(HarnessError::Invalid(error)) if error.contains("listing byte bound")));
    Ok(())
}

#[test]
fn given_member_count_overflow_when_parsed_then_listing_is_rejected() {
    let given =
        "-rw-r--r-- 0 0 0 0 Nov 12 2026 redis-8.10.3/a\n".repeat(MAX_MEMBERS.saturating_add(1));
    let when = parse(&given);
    assert!(when.is_err());
}


#[test]
fn given_running_group_when_finalized_then_leader_is_reaped() -> TestResult {
    use std::os::unix::process::CommandExt;
    let mut given = Command::new("sleep").arg("10").process_group(0).spawn()?;
    let status = crate::install::graph::build::terminate_and_reap(&mut given)?;
    assert!(!status.success());
    assert_eq!(given.try_wait()?, Some(status));
    Ok(())
}

#[test]
fn given_source_symlink_when_claimed_then_target_is_untouched() -> TestResult {
    let (given, source, mut input) = fixture(&[member("redis-8.10.3/a", b"new")])?;
    let target = given.path().join("target");
    std::fs::create_dir(&target)?;
    std::fs::write(target.join("foreign"), b"keep")?;
    std::os::unix::fs::symlink(&target, &source)?;
    let when = extract_open(&mut input, &source);
    assert!(when.is_err());
    assert_eq!(std::fs::read_link(&source)?, target);
    assert_eq!(std::fs::read(source.join("foreign"))?, b"keep");
    assert!(!source.join("a").try_exists()?);
    Ok(())
}

#[test]
fn given_replaced_source_after_claim_when_extracted_then_only_owned_tree_is_written() -> TestResult {
    let (given, source, mut input) = fixture(&[member("redis-8.10.3/kept", b"owned")])?;
    let owned = OwnedDirectory::claim(&source)?;
    let moved = given.path().join("moved");
    let foreign = given.path().join("foreign");
    std::fs::rename(&source, &moved)?;
    std::fs::create_dir(&foreign)?;
    std::fs::write(foreign.join("kept"), b"foreign")?;
    std::os::unix::fs::symlink(&foreign, &source)?;
    let when = extract_claimed(&mut input, owned);
    let retained = when?;
    assert_eq!(std::fs::read(moved.join("kept"))?, b"owned");
    assert_eq!(std::fs::read(foreign.join("kept"))?, b"foreign");
    assert_eq!(std::fs::read(retained.anchor().join("kept"))?, b"owned");
    Ok(())
}

#[test]
fn given_replaced_source_when_extraction_fails_then_cleanup_stays_on_owned_tree() -> TestResult {
    let members = [member("redis-8.10.3/a", b"one"), member("redis-8.10.3/a", b"two")];
    let (given, source, mut input) = fixture(&members)?;
    let owned = OwnedDirectory::claim(&source)?;
    std::fs::write(owned.anchor().join("claimed"), b"remove")?;
    let moved = given.path().join("moved");
    let foreign = given.path().join("foreign");
    std::fs::rename(&source, &moved)?;
    std::fs::create_dir(&foreign)?;
    std::fs::write(foreign.join("claimed"), b"keep")?;
    std::os::unix::fs::symlink(&foreign, &source)?;
    let when = extract_claimed(&mut input, owned);
    assert!(when.is_err());
    assert!(!moved.join("claimed").try_exists()?);
    assert_eq!(std::fs::read(foreign.join("claimed"))?, b"keep");
    assert_eq!(std::fs::read_link(&source)?, foreign);
    Ok(())
}
