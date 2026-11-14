use super::{
    DenyList, Error, ReportMode, classify,
    counts::{Counts, FIELDS},
    model,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::{self, Read, Write as _},
    path::Path,
};

const REPORT: &str = "catalog/aws-classes.report";
const LEGEND: &str = concat!(
    "# enumerate and expand are candidates; ",
    "callable only with a reviewed service overlay (VISION 5.1).\n"
);

#[derive(Default)]
struct Report {
    totals: Counts,
    services: BTreeMap<String, Counts>,
    matched: BTreeSet<usize>,
}

pub(super) fn build(root: &Path, source: &Path) -> Result<String, Error> {
    let text = read_text(&root.join("catalog/overlay/must-deny.txt"))?;
    let deny = DenyList::parse(&text)?;
    let models = source.join("models");
    let files = crate::smithy::find_model_files(&models)?;
    if files.is_empty() {
        return Err(crate::smithy::Error::NoModels.into());
    }
    if files.len() > 4096 {
        return Err(Error::Limit);
    }
    let report = files
        .iter()
        .try_fold(Report::default(), |mut report, file| {
            report.add(&models, file, &deny)?;
            Ok::<_, Error>(report)
        })?;
    report.render(&deny)
}

impl Report {
    fn add(&mut self, models: &Path, file: &Path, deny: &DenyList<'_>) -> Result<(), Error> {
        let model: model::Model = crate::smithy::read_json(file)?;
        model.validate(file)?;
        let service = model::service(models, file)?;
        let counts = self.services.entry(service.to_owned()).or_default();
        model.operations(service).try_for_each(|facts| {
            let facts = facts?;
            let class = classify(&facts, deny);
            self.totals.record(&facts, class)?;
            counts.record(&facts, class)?;
            self.matched
                .extend(deny.0.iter().filter(|e| e.matches(&facts)).map(|e| e.line));
            Ok(())
        })
    }

    fn render(&self, deny: &DenyList<'_>) -> Result<String, Error> {
        self.validate(deny)?;
        let mut text = String::new();
        text.try_reserve(4_194_304).map_err(|_| Error::Limit)?;
        text.push_str(LEGEND);
        FIELDS
            .iter()
            .try_for_each(|key| writeln!(text, "{key} {}", self.totals.count(key)))?;
        writeln!(text, "must_deny_entries {}", deny.len())?;
        self.services
            .iter()
            .try_for_each(|(service, c)| c.render_service(&mut text, service))?;
        Ok(text)
    }

    fn validate(&self, deny: &DenyList<'_>) -> Result<(), Error> {
        if let Some(entry) = deny.0.iter().find(|e| !self.matched.contains(&e.line)) {
            return Err(Error::UnmatchedEntry {
                line: entry.line,
                key: entry.key.to_owned(),
            });
        }
        self.totals.validate()
    }
}

pub(super) fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build(root, source)?;
    publish(root, REPORT, &text, mode)
}

pub(crate) fn publish(
    root: &Path,
    relative: &str,
    text: &str,
    mode: ReportMode,
) -> Result<(), Error> {
    io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(|source| io_error(Path::new("<stdout>"), source))?;
    let path = root.join(relative);
    match mode {
        ReportMode::Print => Ok(()),
        ReportMode::Check => check(&path, text),
        ReportMode::Write => write_report(&path, text),
    }
}

fn write_report(path: &Path, text: &str) -> Result<(), Error> {
    let parent = path.parent().ok_or(Error::Limit)?;
    fs::create_dir_all(parent).map_err(|source| io_error(path, source))?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|source| io_error(path, source))?;
    temporary
        .write_all(text.as_bytes())
        .map_err(|source| io_error(path, source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| io_error(path, source))?;
    temporary
        .persist(path)
        .map_err(|error| io_error(path, error.error))?;
    Ok(())
}

fn check(path: &Path, expected: &str) -> Result<(), Error> {
    let actual = read_text(path)?;
    let differences = crate::smithy::report_differences(expected, &actual)?;
    if differences.is_empty() {
        Ok(())
    } else {
        Err(Error::ReportMismatch(differences))
    }
}

pub(crate) fn read_text(path: &Path) -> Result<String, Error> {
    let file = fs::File::open(path).map_err(|source| io_error(path, source))?;
    let mut text = String::new();
    text.try_reserve(4_194_305).map_err(|_| Error::Limit)?;
    file.take(4_194_305)
        .read_to_string(&mut text)
        .map_err(|source| io_error(path, source))?;
    if text.len() > 4_194_304 {
        return Err(Error::Limit);
    }
    Ok(text)
}

fn io_error(file: &Path, source: io::Error) -> Error {
    Error::Io {
        file: file.to_path_buf(),
        source,
    }
}
