use crate::{
    candidate::ALL,
    error::{Error, Result, syntax},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}
#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    license: Option<String>,
    manifest_path: PathBuf,
}
pub fn run() -> Result<()> {
    let metadata: Metadata =
        serde_json::from_slice(&cargo(&["metadata", "--format-version", "1", "--locked"])?)
            .map_err(syntax)?;
    let baseline = baseline()?;
    println!(
        "\n| Candidate | Version | License | Own source unsafe lines | Normal transitive packages | Added to existing aws-wire | Adapter lines (shared listed below) |\n|---|---|---|---:|---:|---:|---:|"
    );
    ALL.iter().try_for_each(|c| {
        if c.name() == "minimal" {
            println!(
                "| minimal | local | MIT | 0 | 0 | 0 | {} |",
                lines("src/json_minimal.rs")?.saturating_add(lines("src/minimal_lex.rs")?)
            );
            return Ok(());
        }
        let p = metadata
            .packages
            .iter()
            .find(|p| p.name == c.name())
            .ok_or(Error::Shape("metadata package missing"))?;
        let dependencies = tree(c.name())?;
        let added = dependencies.difference(&baseline).count();
        let unsafe_lines =
            unsafe_lines(&p.manifest_path.parent().ok_or(Error::Bound)?.join("src"))?;
        let license = p.license.as_deref().map_or("UNKNOWN", |s| s);
        println!(
            "| {} | {} | {license} | {unsafe_lines} | {} | {added} | {} |",
            c.name(),
            p.version,
            dependencies.len().saturating_sub(1),
            lines(adapter(c.name()))?
        );
        Ok(())
    })
}
fn adapter(name: &str) -> &'static str {
    match name {
        "aws-smithy-json" => "src/json_smithy.rs",
        "jiter" => "src/json_jiter.rs",
        "struson" => "src/json_struson.rs",
        "serde_json" => "src/json_serde.rs",
        "json-event-parser" => "src/json_event.rs",
        "aws-smithy-xml" => "src/xml_smithy.rs",
        _ => "src/xml_events.rs",
    }
}
fn cargo(args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("cargo").args(args).output()?;
    if !output.status.success() {
        return Err(Error::Syntax(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(output.stdout)
}
fn tree(name: &str) -> Result<BTreeSet<String>> {
    let bytes = cargo(&[
        "tree", "--locked", "-e", "normal", "--prefix", "none", "--format", "{p}", "-p", name,
    ])?;
    let text = std::str::from_utf8(&bytes).map_err(syntax)?;
    Ok(text
        .lines()
        .map(|line| line.trim_end_matches(" (*)").to_owned())
        .collect())
}
fn lines(path: &str) -> Result<usize> {
    Ok(fs::read_to_string(path)?.lines().count())
}
fn unsafe_lines(root: &Path) -> Result<usize> {
    let mut pending = vec![root.to_path_buf()];
    let mut total = 0usize;
    std::iter::from_fn(|| {
        pending.pop().map(|path| {
            if path.is_dir() {
                fs::read_dir(path)?.try_for_each(|entry| {
                    pending.push(entry?.path());
                    Ok::<_, std::io::Error>(())
                })?;
            } else if path.extension().is_some_and(|e| e == "rs") {
                total = total.saturating_add(
                    fs::read_to_string(path)?
                        .lines()
                        .filter(|s| s.contains("unsafe"))
                        .count(),
                );
            }
            Ok::<_, Error>(())
        })
    })
    .take(100_000)
    .try_for_each(|r| r)?;
    if !pending.is_empty() {
        return Err(Error::Bound);
    }
    Ok(total)
}
fn baseline() -> Result<BTreeSet<String>> {
    let lock = fs::read_to_string("../../Cargo.lock")?;
    let packages: Vec<_> = lock
        .split("[[package]]")
        .filter_map(parse_package)
        .collect();
    let by_name: BTreeMap<_, _> = packages
        .iter()
        .map(|(name, version, deps)| (format!("{name} v{version}"), deps))
        .collect();
    let root = packages
        .iter()
        .find(|(name, _, _)| name == "aws-wire")
        .ok_or(Error::Shape("aws-wire lock entry missing"))?;
    let mut pending = vec![format!("{} v{}", root.0, root.1)];
    let mut visited = BTreeSet::new();
    std::iter::from_fn(|| {
        pending.pop().map(|key| {
            if visited.insert(key.clone())
                && let Some(deps) = by_name.get(&key)
            {
                deps.iter()
                    .for_each(|dep| extend_dep(dep, &packages, &mut pending));
            }
        })
    })
    .take(100_000)
    .for_each(|()| {});
    if !pending.is_empty() {
        return Err(Error::Bound);
    }
    Ok(visited)
}
fn parse_package(block: &str) -> Option<(String, String, Vec<String>)> {
    let name = block
        .lines()
        .find_map(|s| s.strip_prefix("name = \""))
        .and_then(|s| s.strip_suffix('"'))?;
    let version = block
        .lines()
        .find_map(|s| s.strip_prefix("version = \""))
        .and_then(|s| s.strip_suffix('"'))?;
    let dependencies = block
        .split_once("dependencies = [")
        .map_or_else(Vec::new, |(_, tail)| {
            tail.lines()
                .skip(1)
                .take_while(|s| *s != "]")
                .filter_map(|s| {
                    s.trim()
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix("\","))
                        .map(str::to_owned)
                })
                .collect()
        });
    Some((name.to_owned(), version.to_owned(), dependencies))
}
fn extend_dep(dep: &str, packages: &[(String, String, Vec<String>)], pending: &mut Vec<String>) {
    let mut words = dep.split_whitespace();
    let name = words.next().map_or("", |s| s);
    let version = words.next();
    packages
        .iter()
        .filter(|(n, v, _)| n == name && version.is_none_or(|want| want == v))
        .for_each(|(n, v, _)| pending.push(format!("{n} v{v}")));
}
