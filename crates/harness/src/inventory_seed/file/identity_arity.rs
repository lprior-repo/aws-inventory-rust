use super::load;
use crate::inventory_seed::SeedIdentity;
use inventory_core::catalog::runtime::{
    IdentityChoicePlan, IdentityLeafPlan, IdentityPlan, RuntimeCatalog,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type CompositeKinds = BTreeMap<String, BTreeSet<usize>>;

fn leaf_arity(leaf: &IdentityLeafPlan) -> usize {
    match leaf {
        IdentityLeafPlan::Key(paths) => paths.len(),
        IdentityLeafPlan::Fields(fields) => fields.len(),
        IdentityLeafPlan::Arn(_)
        | IdentityLeafPlan::OwnedArn { .. }
        | IdentityLeafPlan::Template(_) => 1,
    }
}

fn identity_arities(identity: &IdentityPlan) -> Vec<usize> {
    match identity {
        IdentityPlan::Key(paths) => vec![paths.len()],
        IdentityPlan::CorrelatedKey(fields) => vec![fields.len()],
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => {
            vec![1, leaf_arity(fallback)]
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches
            .iter()
            .map(|branch| leaf_arity(&branch.identity))
            .collect(),
        IdentityPlan::Arn(_) | IdentityPlan::Template(_) | IdentityPlan::MapArn { .. } => {
            vec![1]
        }
    }
}

fn composite_kinds(catalog: &RuntimeCatalog<'_>) -> CompositeKinds {
    let mut kinds = CompositeKinds::new();
    for emission in catalog
        .plans()
        .iter()
        .flat_map(|plan| plan.resource_emissions().iter())
    {
        let arities = identity_arities(emission.identity());
        let variants = emission
            .classification()
            .into_iter()
            .flat_map(|classification| classification.variants.iter().map(|(_, kind)| kind));
        for kind in std::iter::once(emission.kind()).chain(variants) {
            kinds
                .entry(kind.as_str().to_owned())
                .or_default()
                .extend(arities.iter().copied());
        }
    }
    kinds.retain(|_, arities| arities.iter().all(|arity| *arity > 1));
    kinds
}

fn part_is_well_formed(part: &str) -> bool {
    let Some((length, body)) = part.split_once(':') else {
        return false;
    };
    let Ok(length) = length.parse::<usize>() else {
        return false;
    };
    body.contains('{') || body.len() == length
}

fn template_arity(template: &str) -> Option<usize> {
    let parts: Vec<&str> = template.split('|').collect();
    parts
        .iter()
        .all(|part| part_is_well_formed(part))
        .then_some(parts.len())
}

fn declared_arity(identity: Option<&SeedIdentity>) -> Option<usize> {
    match identity {
        Some(SeedIdentity::Request {
            template: Some(template),
            ..
        }) => template_arity(template),
        _ => None,
    }
}

fn check_file(
    path: &Path,
    kinds: &CompositeKinds,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let scenario = load(&std::fs::read_to_string(path)?)?;
    Ok(scenario
        .steps()
        .iter()
        .filter_map(|step| {
            let expected = kinds.get(&step.kind)?;
            let declared = declared_arity(step.identity.as_ref());
            if declared.is_some_and(|arity| expected.contains(&arity)) {
                return None;
            }
            Some(format!(
                "{} step {} {}: catalog identity has {expected:?} components, seed declares {declared:?}",
                path.display(),
                step.id.0,
                step.kind
            ))
        })
        .collect())
}

#[test]
fn given_committed_seed_files_when_kind_has_composite_catalog_identity_then_seed_identity_is_matching_composite_template()
-> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join("../../catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let kinds = composite_kinds(&catalog);
    assert!(!kinds.is_empty());
    let mut failures = Vec::new();
    for directory in [root.join("seeds"), root.join("seeds/bench")] {
        if !directory.exists() {
            continue;
        }
        for path in crate::inventory_seed::command::directory::files(&directory)? {
            failures.extend(check_file(&path, &kinds)?);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
