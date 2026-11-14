use super::{
    Error, Model, Vocabulary,
    counts::{Counts, FIELDS, SERVICE_FIELDS},
    tally,
};
use crate::{
    classes::{self, DenyList},
    smithy::{self, ReportMode},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::{Path, PathBuf},
};

const LEGEND: &str = "# derivations from the pinned models; nothing here makes an operation callable (VISION 5.1).\n";

#[derive(Default)]
struct Report {
    totals: Counts,
    services: BTreeMap<String, Counts>,
    matched: BTreeSet<usize>,
}

pub(super) fn build(root: &Path, source: &Path) -> Result<String, Error> {
    let text = classes::report::read_text(&root.join("catalog/overlay/must-deny.txt"))?;
    let deny = DenyList::parse(&text).map_err(classes::Error::from)?;
    let models = source.join("models");
    let files = input_files(&models)?;
    let mut vocabulary = Vocabulary::default();
    files.iter().try_for_each(|file| {
        let value: Value = smithy::read_json(file)?;
        let service = classes::model::service(&models, file)?;
        vocabulary.record(&Model::parse(&value, service)?)
    })?;
    let mut report = Report::default();
    files
        .iter()
        .try_for_each(|file| report.add(&models, file, &vocabulary, &deny))?;
    report.validate(&deny)?;
    report.render(&vocabulary)
}

fn input_files(models: &Path) -> Result<Vec<PathBuf>, Error> {
    let files = smithy::find_model_files(models)?;
    if files.is_empty() {
        return Err(smithy::Error::NoModels.into());
    }
    if files.len() > 4096 {
        return Err(Error::Limit);
    }
    Ok(files)
}

impl Report {
    fn add(
        &mut self,
        models: &Path,
        file: &Path,
        vocab: &Vocabulary,
        deny: &DenyList<'_>,
    ) -> Result<(), Error> {
        let value: Value = smithy::read_json(file)?;
        let service = classes::model::service(models, file)?;
        let model = Model::parse(&value, service)?;
        let counts = tally::collect(&model, vocab, deny, &mut self.matched)?;
        self.totals.merge(&counts)?;
        self.services
            .entry(service.to_owned())
            .or_default()
            .merge(&counts)
    }

    fn validate(&self, deny: &DenyList<'_>) -> Result<(), Error> {
        if let Some(entry) = deny.0.iter().find(|e| !self.matched.contains(&e.line)) {
            return Err(classes::Error::UnmatchedEntry {
                line: entry.line,
                key: entry.key.to_owned(),
            }
            .into());
        }
        self.validate_sums()
    }

    fn validate_sums(&self) -> Result<(), Error> {
        self.totals.validate()?;
        let sum = self
            .services
            .values()
            .try_fold(Counts::default(), |mut sum, counts| {
                counts.validate()?;
                sum.merge(counts)?;
                Ok::<_, Error>(sum)
            })?;
        if FIELDS
            .iter()
            .any(|key| sum.count(key) != self.totals.count(key))
        {
            return Err(Error::CountMismatch);
        }
        Ok(())
    }

    fn render(&self, vocabulary: &Vocabulary) -> Result<String, Error> {
        self.validate_sums()?;
        let mut text = String::new();
        text.try_reserve(4_194_304).map_err(|_| Error::Limit)?;
        text.push_str(LEGEND);
        FIELDS
            .iter()
            .try_for_each(|key| writeln!(text, "{key} {}", self.totals.count(key)))?;
        writeln!(text, "token_pairs {}", vocabulary.pairs.len())?;
        self.services
            .iter()
            .try_for_each(|(service, counts)| render_service(&mut text, service, counts))?;
        Ok(text)
    }
}

fn render_service(text: &mut String, service: &str, counts: &Counts) -> Result<(), Error> {
    write!(text, "service {service}")?;
    SERVICE_FIELDS
        .iter()
        .try_for_each(|key| write!(text, " {}", counts.count(key)))?;
    Ok(writeln!(text)?)
}

pub(super) fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build(root, source)?;
    Ok(classes::report::publish(
        root,
        "catalog/aws-plans.report",
        &text,
        mode,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn violated_service_sum_is_a_typed_error() -> Result<(), Error> {
        let mut report = Report::default();
        report.totals.increment("operations")?;
        report.totals.increment("unclassified")?;
        assert!(matches!(
            report.render(&Vocabulary::default()),
            Err(Error::CountMismatch)
        ));
        Ok(())
    }
}
