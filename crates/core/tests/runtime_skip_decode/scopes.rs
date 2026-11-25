use super::*;
use inventory_core::catalog::runtime::BindingSource;
use std::collections::BTreeSet;

#[test]
fn given_a_grandchild_when_its_grandparent_lists_it_as_a_skip_then_only_a_direct_parent_can_skip_it()
-> TestResult {
    // ListAnalyzers binds ListArchiveRules, and ListArchiveRules binds GetArchiveRule.
    let parent = "ListArchiveRules";
    let grandchild = "GetArchiveRule";
    let base = base_catalog::base()?;
    let catalog = RuntimeCatalog::from_bytes(base)?;
    let bound_to = |operation| -> TestResult<_> {
        let plan = catalog.lookup(SERVICE, operation).ok_or("missing plan")?;
        plan.bindings()
            .iter()
            .map(|binding| match &binding.source {
                BindingSource::Operation {
                    service, operation, ..
                }
                | BindingSource::Context {
                    service, operation, ..
                } => Ok((service.as_ref(), operation.as_ref())),
                _ => Err("expected a direct operation binding".into()),
            })
            .collect::<TestResult<BTreeSet<_>>>()
    };
    assert_eq!(bound_to(parent)?, BTreeSet::from([(SERVICE, PLAN)]));
    assert_eq!(bound_to(grandchild)?, BTreeSet::from([(SERVICE, parent)]));

    let direct = artifact_for(SERVICE, parent, Some(skips([grandchild])))?;
    let indirect = artifact(Some(skips([grandchild])))?;

    let accepted = RuntimeCatalog::from_bytes(&direct)?;
    let carrier = accepted.lookup(SERVICE, parent).ok_or("missing carrier")?;
    assert_eq!(carrier.skips(), &[Box::<str>::from(grandchild)]);
    assert!(matches!(
        RuntimeCatalog::from_bytes(&indirect),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_one_unbound_branch_when_loading_a_skip_then_the_decoder_refuses_it() -> TestResult {
    // The second branch overrides the parent binding with a fixed analyzer name.
    let base = branched_child(Some("analyzer-without-parent"))?;
    RuntimeCatalog::from_bytes(&base)?;

    let bytes = branched_skip(&base)?;

    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_every_branch_bound_to_the_direct_parent_when_loading_a_skip_then_it_is_retained()
-> TestResult {
    // Both branches inherit GetAnalyzer's common binding to ListAnalyzers.
    let base = branched_child(None)?;

    let bytes = branched_skip(&base)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;

    let carrier = catalog.lookup(SERVICE, PLAN).ok_or("missing carrier")?;
    assert_eq!(carrier.skips(), &[Box::<str>::from(CHILD)]);
    Ok(())
}

const ARCHIVE_PARENT: &str = "ListArchiveRules";
const ARCHIVE_CHILD: &str = "GetArchiveRule";

#[test]
fn given_alternative_with_foreign_producer_when_loading_skip_then_decoder_refuses_it() -> TestResult
{
    let bytes = archive_rule_skip(AnalyzerBinding::MixedAlternatives)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_separate_binding_to_foreign_producer_when_loading_skip_then_decoder_refuses_it()
-> TestResult {
    let bytes = archive_rule_skip(AnalyzerBinding::Foreign)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(&bytes),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_alternative_only_from_carrier_when_loading_skip_then_decoder_accepts_it() -> TestResult {
    let bytes = archive_rule_skip(AnalyzerBinding::ParentAlternative)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let parent = catalog
        .lookup(SERVICE, ARCHIVE_PARENT)
        .ok_or("missing archive rule parent")?;
    assert_eq!(parent.skips(), &[Box::<str>::from(ARCHIVE_CHILD)]);
    Ok(())
}

enum AnalyzerBinding {
    MixedAlternatives,
    Foreign,
    ParentAlternative,
}

fn archive_rule_skip(binding: AnalyzerBinding) -> TestResult<Vec<u8>> {
    let base = base_catalog::base()?;
    let catalog = RuntimeCatalog::from_bytes(base)?;
    let parent = catalog
        .lookup(SERVICE, ARCHIVE_PARENT)
        .ok_or("missing archive rule parent")?;
    let Some(BindingSource::Operation { path, .. }) = parent.bindings().first().map(|b| &b.source)
    else {
        return Err("missing ListAnalyzers.name binding".into());
    };
    let name_path = Value::List(
        path.iter()
            .map(|id| Value::Int(i64::from(id.value())))
            .collect(),
    );
    let base = carrier_artifact(base, SERVICE, ARCHIVE_CHILD, false, |child| {
        let Some(Value::List(bindings)) = child.get_mut(&key(7)?) else {
            return Err("missing archive rule child bindings".into());
        };
        // Replace the request-context binding for analyzerName, retaining the
        // direct binding of ruleName to the carrier.
        let source_key = key(2)?;
        let analyzer = bindings
            .iter_mut()
            .find(|binding| matches!(binding, Value::Map(map) if map.get(&source_key) == Some(&Value::Int(7))))
            .ok_or("missing analyzerName binding")?;
        let destination = fields(analyzer)?
            .get(&key(1)?)
            .ok_or("missing analyzerName destination")?
            .clone();
        *analyzer = analyzer_binding(binding, destination, name_path)?;
        Ok(())
    })?;
    RuntimeCatalog::from_bytes(&base)?;
    carrier_artifact(&base, SERVICE, ARCHIVE_PARENT, false, |parent| {
        parent.insert(key(26)?, skips([ARCHIVE_CHILD]));
        Ok(())
    })
}

fn analyzer_binding(
    binding: AnalyzerBinding,
    destination: Value,
    name_path: Value,
) -> TestResult<Value> {
    if matches!(binding, AnalyzerBinding::Foreign) {
        return map_value(vec![
            (1, destination),
            (2, Value::Int(4)),
            (3, Value::Text(SERVICE.into())),
            (4, Value::Text(PLAN.into())),
            (5, name_path),
        ]);
    }
    let mut alternatives = vec![alternative(ARCHIVE_PARENT, 2, destination.clone())?];
    if matches!(binding, AnalyzerBinding::MixedAlternatives) {
        alternatives.push(alternative(PLAN, 0, name_path)?);
    }
    map_value(vec![
        (1, destination),
        (2, Value::Int(12)),
        (3, Value::List(alternatives)),
    ])
}

fn alternative(operation: &str, source: i64, path: Value) -> TestResult<Value> {
    map_value(vec![
        (1, Value::Text(SERVICE.into())),
        (2, Value::Text(operation.into())),
        (3, Value::Int(source)),
        (4, path),
    ])
}

fn branched_skip(base: &[u8]) -> TestResult<Vec<u8>> {
    carrier_artifact(base, SERVICE, PLAN, false, |plan| {
        plan.insert(key(26)?, skips([CHILD]));
        Ok(())
    })
}

fn branched_child(fixed_name: Option<&str>) -> TestResult<Vec<u8>> {
    carrier_artifact(base_catalog::base()?, SERVICE, CHILD, false, |child| {
        let second = fixed_name.map_or_else(
            || Ok(Vec::new()),
            |name| -> TestResult<_> {
                let Some(Value::List(bindings)) = child.get(&key(7)?) else {
                    return Err("missing child bindings".into());
                };
                let Some(Value::Map(binding)) = bindings.first() else {
                    return Err("missing child parent binding".into());
                };
                let destination = binding.get(&key(1)?).ok_or("missing binding destination")?;
                Ok(vec![map_value(vec![
                    (1, destination.clone()),
                    (2, Value::Int(2)),
                    (3, Value::Text(name.into())),
                ])?])
            },
        )?;
        child.insert(
            key(16)?,
            Value::List(vec![
                branch("first", Vec::new())?,
                branch("second", second)?,
            ]),
        );
        Ok(())
    })
}

fn branch(name: &str, bindings: Vec<Value>) -> TestResult<Value> {
    map_value(vec![
        (1, Value::Text(name.into())),
        (2, Value::List(bindings)),
        (3, Value::Int(0)),
    ])
}
