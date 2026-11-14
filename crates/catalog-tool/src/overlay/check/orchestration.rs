use super::{CheckError, CheckedOverlays, Violation, ViolationRule, classifier, report, service};
use crate::{
    classes::{self, Class, DenyList},
    plans::{self, Model},
    smithy,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Default)]
struct Accumulation {
    checked: Vec<super::ReviewedService>,
    rows: Vec<(String, String, String, String)>,
    violations: Vec<Violation>,
    unreviewed: [usize; 3],
}

pub(super) fn check(root: &Path, source: &Path) -> Result<CheckedOverlays, CheckError> {
    let overlays = super::super::load_dir(root)?;
    let deny_text = read_deny(root)?;
    let deny = DenyList::parse(&deny_text)?;
    let values = read_model_values(source)?;
    let models = parse_models(&values)?;
    enforce_model_limit(models.len())?;
    let vocabulary = model_vocabulary(&models)?;
    let mut accumulation = Accumulation::default();
    unknown_overlay_services(&overlays, &values, &mut accumulation.violations);
    process_models(
        &values,
        &models,
        &overlays,
        &deny,
        &vocabulary,
        &mut accumulation,
    )?;
    finish(overlays.len(), models.len(), accumulation)
}

fn read_deny(root: &Path) -> Result<String, CheckError> {
    let path = root.join("catalog/overlay/must-deny.txt");
    fs::read_to_string(&path).map_err(|source| CheckError::Io { file: path, source })
}

fn read_model_values(source: &Path) -> Result<BTreeMap<String, Value>, CheckError> {
    let models = source.join("models");
    let files = smithy::find_model_files(&models)?;
    let mut values = BTreeMap::new();
    for file in files {
        let (service, value) = read_model_value(&models, &file)?;
        if values.contains_key(&service) {
            return Err(CheckError::Invalid {
                service,
                operation: String::new(),
                key: "model".to_owned(),
                value: file.display().to_string(),
                rule: "duplicate service model",
            });
        }
        values.insert(service, value);
    }
    Ok(values)
}

fn read_model_value(models: &Path, file: &Path) -> Result<(String, Value), CheckError> {
    let service = classes::model::service(models, file)?;
    Ok((service.to_owned(), smithy::read_json(file)?))
}

fn parse_models<'a>(values: &'a BTreeMap<String, Value>) -> Result<Vec<Model<'a>>, CheckError> {
    values
        .iter()
        .map(|(service, value)| {
            Model::parse(value, service).map_err(|error| CheckError::Invalid {
                service: service.clone(),
                operation: String::new(),
                key: "model".to_owned(),
                value: error.to_string(),
                rule: "invalid model",
            })
        })
        .collect()
}

fn enforce_model_limit(count: usize) -> Result<(), CheckError> {
    if count > super::MAX_SERVICES {
        Err(CheckError::Classes(classes::Error::Limit))
    } else {
        Ok(())
    }
}

fn model_vocabulary(models: &[Model<'_>]) -> Result<plans::Vocabulary, CheckError> {
    let mut vocabulary = plans::Vocabulary::default();
    for model in models {
        vocabulary.record(model)?;
    }
    Ok(vocabulary)
}

fn unknown_overlay_services(
    overlays: &[crate::overlay::ServiceOverlay],
    values: &BTreeMap<String, Value>,
    violations: &mut Vec<Violation>,
) {
    let names: BTreeSet<&str> = values.keys().map(String::as_str).collect();
    let services: BTreeSet<&str> = overlays.iter().map(|item| item.service()).collect();
    for service in services {
        if !names.contains(service) {
            violations.push(Violation {
                service: service.to_owned(),
                operation: String::new(),
                key: "service".to_owned(),
                value: service.to_owned(),
                rule: ViolationRule::UnknownService,
            });
        }
    }
}

fn process_models(
    values: &BTreeMap<String, Value>,
    models: &[Model<'_>],
    overlays: &[crate::overlay::ServiceOverlay],
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
    accumulation: &mut Accumulation,
) -> Result<(), CheckError> {
    for ((name, value), model) in values.iter().zip(models) {
        process_model(name, value, model, overlays, deny, vocabulary, accumulation)?;
    }
    Ok(())
}

fn process_model(
    service_name: &str,
    value: &Value,
    model: &Model<'_>,
    overlays: &[crate::overlay::ServiceOverlay],
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
    accumulation: &mut Accumulation,
) -> Result<(), CheckError> {
    let facts = classifier(value, service_name, deny)?;
    let overlay = overlays.iter().find(|item| item.service() == service_name);
    if let Some(overlay) = overlay {
        review_service(overlay, model, &facts, deny, vocabulary, accumulation);
    } else {
        count_unreviewed(model, &facts, &mut accumulation.unreviewed)?;
    }
    Ok(())
}

fn review_service(
    overlay: &crate::overlay::ServiceOverlay,
    model: &Model<'_>,
    facts: &std::collections::BTreeMap<String, Class>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
    accumulation: &mut Accumulation,
) {
    let (reviewed, rows, errors) = service::check_service(overlay, model, facts, deny, vocabulary);
    accumulation.checked.push(reviewed);
    accumulation.rows.extend(rows);
    accumulation.violations.extend(errors);
}

fn count_unreviewed(
    model: &Model<'_>,
    facts: &std::collections::BTreeMap<String, Class>,
    counts: &mut [usize; 3],
) -> Result<(), CheckError> {
    let [enumerate, expand, unclassified] = counts;
    for operation in model.operation_names() {
        match facts.get(operation).copied().unwrap_or(Class::Unclassified) {
            Class::Enumerate => increment_count(enumerate)?,
            Class::Expand => increment_count(expand)?,
            Class::Unclassified => increment_count(unclassified)?,
            Class::Denied(_) => {}
        }
    }
    Ok(())
}

fn increment_count(count: &mut usize) -> Result<(), CheckError> {
    *count = count.checked_add(1).ok_or(CheckError::CountOverflow)?;
    Ok(())
}

fn finish(
    reviewed: usize,
    services: usize,
    mut accumulation: Accumulation,
) -> Result<CheckedOverlays, CheckError> {
    if !accumulation.violations.is_empty() {
        accumulation.violations.sort();
        return Err(CheckError::Violations(accumulation.violations));
    }
    accumulation.rows.sort();
    accumulation
        .checked
        .sort_by(|left, right| left.service.cmp(&right.service));
    let report = report::render(
        &accumulation.rows,
        services,
        reviewed,
        accumulation.unreviewed,
    )?;
    Ok(CheckedOverlays {
        services: accumulation.checked,
        report,
    })
}
