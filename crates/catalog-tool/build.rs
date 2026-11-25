use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};

type Failure = Box<dyn std::error::Error>;
const SOURCE_ROOTS: &[&str] = &[
    "build.rs",
    "src/overlay.rs",
    "src/overlay",
    "src/kind_map.rs",
    "src/kind_map",
    "src/names.rs",
    "src/names",
    "src/classes.rs",
    "src/classes",
];

fn main() -> Result<(), Failure> {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing crate directory")?);
    let first = snapshot(&root)?;
    if snapshot(&root)? != first {
        return Err("embedded catalog sources changed during provenance capture".into());
    }
    for name in SOURCE_ROOTS {
        println!("cargo:rerun-if-changed={name}");
    }
    for path in &first.1 {
        println!("cargo:rerun-if-changed={}", relative(&root, path)?);
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing build output directory")?);
    thin_services(&root, &output)?;
    fs::write(
        output.join("embedded_sources_sha256.rs"),
        format!(
            "pub(crate) const EMBEDDED_SOURCES_SHA256: [u8; 32] = {:?};\n",
            first.0,
        ),
    )?;
    Ok(())
}

fn snapshot(root: &Path) -> Result<([u8; 32], Vec<PathBuf>), Failure> {
    let mut files = BTreeSet::new();
    for name in SOURCE_ROOTS {
        collect(&root.join(name), &mut files)?;
    }
    let mut files = files
        .into_iter()
        .map(|path| Ok((relative(root, &path)?, path)))
        .collect::<Result<Vec<_>, Failure>>()?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hash = Sha256::new();
    hash.update(b"catalog-tool-embedded-sources-v1\0");
    for (name, path) in &files {
        hash.update(u64::try_from(name.len())?.to_le_bytes());
        hash.update(name.as_bytes());
        hash_file(path, &mut hash)?;
    }
    Ok((
        hash.finalize().into(),
        files.into_iter().map(|(_, path)| path).collect(),
    ))
}

fn collect(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<(), Failure> {
    let kind = fs::symlink_metadata(path)?.file_type();
    if kind.is_dir() {
        for entry in fs::read_dir(path)? {
            collect(&entry?.path(), files)?;
        }
    } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
        files.insert(path.to_path_buf());
    } else if kind.is_symlink() {
        return Err(format!(
            "embedded catalog source must not be a symlink: {}",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> Result<String, Failure> {
    path.strip_prefix(root)?
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| "non-UTF-8 embedded source path".into())
        })
        .collect::<Result<Vec<_>, Failure>>()
        .map(|parts| parts.join("/"))
}

fn hash_file(path: &Path, hash: &mut Sha256) -> Result<(), Failure> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len();
    hash.update(length.to_le_bytes());
    let mut total = 0_u64;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(u64::try_from(read)?)
            .filter(|total| *total <= length)
            .ok_or("embedded source changed during capture")?;
        hash.update(
            buffer
                .get(..read)
                .ok_or("invalid embedded source read length")?,
        );
    }
    if total != length {
        return Err("embedded source changed during capture".into());
    }
    Ok(())
}

fn thin_services(root: &Path, output: &Path) -> Result<(), Failure> {
    let directory = root.join("../harness/seeds/bench");
    println!("cargo:rerun-if-changed={}", directory.display());
    let services = fs::read_dir(&directory)?.try_fold(BTreeSet::new(), |mut services, entry| {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            let service = path
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or("thin seed filename is not UTF-8")?;
            services.insert(service.to_owned());
        }
        Ok::<_, Failure>(services)
    })?;
    if services.is_empty() {
        return Err("thin seed service set is empty".into());
    }
    let services = services.into_iter().collect::<Vec<_>>();
    fs::write(
        output.join("thin_services.rs"),
        format!("const THIN_SERVICES: &[&str] = &{services:?};\n"),
    )?;
    Ok(())
}
