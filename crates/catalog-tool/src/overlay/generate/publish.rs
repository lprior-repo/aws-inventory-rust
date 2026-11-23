use super::{Failure, Generated};
use crate::{ReportMode, overlay::directory, smithy::fields::publish};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

const REPORT: &str = "catalog/aws-overlay-generated.report";
const FILE_LIMIT: u64 = 1_048_576;
const REPORT_LIMIT: u64 = 4_194_304;

pub(super) struct Snapshot {
    files: BTreeMap<PathBuf, Vec<u8>>,
    report: Option<Vec<u8>>,
}

impl Snapshot {
    pub(super) fn read(root: &Path) -> Result<Self, Failure> {
        let mut files = BTreeMap::new();
        let mut total = 0_usize;
        for file in directory::generated_files(&root.join("catalog/overlay/generated"))? {
            let bytes = publish::read_bytes(&file, FILE_LIMIT)?.ok_or("generated file disappeared")?;
            total = total.checked_add(bytes.len()).ok_or("snapshot byte count overflow")?;
            if total > 67_108_864 {
                return Err("generated snapshot exceeds 64 MiB".into());
            }
            files.insert(file, bytes);
        }
        let report = publish::read_bytes(&root.join(REPORT), REPORT_LIMIT)?;
        Ok(Self { files, report })
    }

    pub(super) fn publish(
        self,
        root: &Path,
        generated: &Generated,
        mode: ReportMode,
    ) -> Result<(), Failure> {
        match mode {
            ReportMode::Print => Ok(crate::report::write_stdout(generated.report())?),
            ReportMode::Check => self.check(root, generated),
            ReportMode::Write => self.write(root, generated),
        }
    }

    fn check(&self, root: &Path, generated: &Generated) -> Result<(), Failure> {
        if self.files.len() != generated.files.len() {
            return Err("generated file set differs".into());
        }
        for (service, text) in &generated.files {
            let path = file(root, service);
            if self.files.get(&path).map(Vec::as_slice) != Some(text.as_bytes()) {
                return Err(format!("{} differs or is missing", path.display()).into());
            }
        }
        if self.report.as_deref() != Some(generated.report.as_bytes()) {
            return Err("generated overlay report differs or is missing".into());
        }
        Ok(())
    }

    fn write(&self, root: &Path, generated: &Generated) -> Result<(), Failure> {
        for (service, text) in &generated.files {
            let path = file(root, service);
            let original = self.files.get(&path).map(Vec::as_slice);
            publish::publish_bytes(&path, original, text.as_bytes(), FILE_LIMIT)?;
        }
        self.remove_stale(generated)?;
        publish::publish_bytes(&root.join(REPORT), self.report.as_deref(), generated.report.as_bytes(), REPORT_LIMIT)?;
        Ok(())
    }

    fn remove_stale(&self, generated: &Generated) -> Result<(), Failure> {
        for (path, original) in &self.files {
            let service = path.file_stem().and_then(|name| name.to_str()).ok_or("invalid service filename")?;
            if generated.files.contains_key(service) {
                continue;
            }
            if publish::read_bytes(path, FILE_LIMIT)?.as_deref() != Some(original.as_slice()) {
                return Err(format!("{} changed during generation", path.display()).into());
            }
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

fn file(root: &Path, service: &str) -> PathBuf {
    root.join("catalog/overlay/generated").join(format!("{service}.toml"))
}
