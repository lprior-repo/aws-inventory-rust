use super::{
    DenyList, Error, classify,
    counts::{Counts, FIELDS},
    model,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
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
    let text = crate::report::read_text(&root.join("catalog/overlay/must-deny.txt"))?;
    let deny = DenyList::parse(&text)?;
    let models = source.join("models");
    let files = crate::model_source::discover_models(&models)?;
    if files.is_empty() {
        return Err(crate::model_source::Error::NoModels.into());
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
        let model: model::Model = crate::model_source::read_model(file)?;
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

pub(super) fn run(root: &Path, source: &Path, mode: crate::ReportMode) -> Result<(), Error> {
    let text = build(root, source)?;
    crate::report::publish(&root.join(REPORT), &text, mode)?;
    Ok(())
}
