use std::collections::BTreeMap;

use super::model::{ReviewedCatalog, Signal};

pub(super) fn reviewed_parents(
    reviewed: &ReviewedCatalog,
    type_names: &[&str],
) -> BTreeMap<String, Option<String>> {
    let mut parents = BTreeMap::new();
    for row in &reviewed.resource_types {
        if row.list_model_validation.as_deref() != Some("reviewed_parent_model") {
            continue;
        }
        let family = row.type_name.split("::").nth(1).unwrap_or_default();
        let mut candidate = None;
        for pointer in &row.primary_identifier {
            let Some(property) = pointer.rsplit('/').next() else {
                continue;
            };
            let key = normalize(stem(property));
            candidate = same_family(type_names, family, &key, Some(&row.type_name));
            if candidate.is_some() {
                break;
            }
        }
        parents.insert(row.type_name.clone(), candidate);
    }
    parents
}

pub(super) fn infer(
    type_name: &str,
    properties: &[String],
    type_names: &[&str],
    global: &BTreeMap<String, Vec<String>>,
) -> Vec<Signal> {
    let family = type_name.split("::").nth(1).unwrap_or_default();
    properties
        .iter()
        .map(|property| signal(property, type_name, family, type_names, global))
        .collect()
}

fn signal(
    property: &str,
    type_name: &str,
    family: &str,
    type_names: &[&str],
    global: &BTreeMap<String, Vec<String>>,
) -> Signal {
    let key = normalize(stem(property));
    let (candidate_parent, confidence) = classify(&key, type_name, family, type_names, global);
    Signal {
        property: property.to_owned(),
        candidate_parent,
        confidence,
    }
}

fn classify(
    key: &str,
    type_name: &str,
    family: &str,
    type_names: &[&str],
    global: &BTreeMap<String, Vec<String>>,
) -> (Option<String>, &'static str) {
    match same_family(type_names, family, key, Some(type_name)) {
        Some(name) => (Some(name), "high"),
        None => cross_family(key, type_name, global),
    }
}

fn cross_family(
    key: &str,
    type_name: &str,
    global: &BTreeMap<String, Vec<String>>,
) -> (Option<String>, &'static str) {
    let candidates: Vec<&String> = global
        .get(key)
        .map(|names| {
            names
                .iter()
                .filter(|name| name.as_str() != type_name)
                .collect()
        })
        .unwrap_or_default();
    match candidates.as_slice() {
        [name] => (Some((*name).clone()), "medium"),
        [] => (None, "unmapped"),
        _ => (None, "low"),
    }
}

fn same_family(
    type_names: &[&str],
    family: &str,
    key: &str,
    exclude: Option<&str>,
) -> Option<String> {
    let prefix = format!("AWS::{family}::");
    type_names
        .iter()
        .filter(|name| name.starts_with(&prefix))
        .filter(|name| Some(**name) != exclude)
        .find(|name| {
            name.rsplit("::")
                .next()
                .is_some_and(|short| normalize(short) == key)
        })
        .map(|name| (*name).to_owned())
}

pub(super) fn names_by_suffix(type_names: &[&str]) -> BTreeMap<String, Vec<String>> {
    let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in type_names {
        if let Some(short) = name.rsplit("::").next() {
            names
                .entry(normalize(short))
                .or_default()
                .push((*name).to_owned());
        }
    }
    names
}

fn stem(value: &str) -> &str {
    for suffix in ["Identifier", "Id", "Arn", "NAME", "Name", "ARN", "ID"] {
        if let Some(stripped) = value.strip_suffix(suffix)
            && !stripped.is_empty()
        {
            return stripped;
        }
    }
    value
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_strip_resource_suffixes() {
        assert_eq!(stem("RestApiId"), "RestApi");
        assert_eq!(stem("ClusterName"), "Cluster");
        assert_eq!(stem("LoadBalancerArn"), "LoadBalancer");
        assert_eq!(stem("Name"), "Name");
    }

    #[test]
    fn same_family_property_proposes_high_confidence_parent() {
        let names = ["AWS::ApiGateway::RestApi", "AWS::ApiGateway::Model"];
        let global = names_by_suffix(&names);
        let signals = infer(
            "AWS::ApiGateway::Model",
            &["RestApiId".to_owned()],
            &names,
            &global,
        );
        assert_eq!(
            signals.first().map(|signal| signal.confidence),
            Some("high")
        );
        assert_eq!(
            signals
                .first()
                .and_then(|signal| signal.candidate_parent.as_deref()),
            Some("AWS::ApiGateway::RestApi")
        );
    }

    #[test]
    fn unique_cross_family_property_proposes_medium_confidence_parent() {
        let names = ["AWS::IAM::Role", "AWS::Lambda::Function"];
        let global = names_by_suffix(&names);
        let signals = infer(
            "AWS::Lambda::Function",
            &["RoleArn".to_owned()],
            &names,
            &global,
        );
        assert_eq!(
            signals.first().map(|signal| signal.confidence),
            Some("medium")
        );
        assert_eq!(
            signals
                .first()
                .and_then(|signal| signal.candidate_parent.as_deref()),
            Some("AWS::IAM::Role")
        );
    }

    #[test]
    fn ambiguous_or_unknown_properties_stay_unmapped() {
        let names = ["AWS::A::Thing", "AWS::B::Thing"];
        let global = names_by_suffix(&names);
        let signals = infer(
            "AWS::C::Child",
            &["ThingId".to_owned(), "Mystery".to_owned()],
            &names,
            &global,
        );
        assert_eq!(signals.first().map(|signal| signal.confidence), Some("low"));
        assert!(
            signals
                .first()
                .is_some_and(|signal| signal.candidate_parent.is_none())
        );
        assert_eq!(
            signals.get(1).map(|signal| signal.confidence),
            Some("unmapped")
        );
    }
}
