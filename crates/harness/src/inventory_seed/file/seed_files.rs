use super::*;
use crate::inventory_seed::model::SeedModels;
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::path::Path;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_committed_seed_files_when_loaded_then_every_operation_and_path_resolves_offline()
-> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join("../../catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    validate_directory(&root.join("seeds"), &catalog)?;
    let bench = root.join("seeds/bench");
    if bench.exists() {
        validate_directory(&bench, &catalog)?;
    }
    Ok(())
}

fn validate_directory(directory: &Path, catalog: &RuntimeCatalog<'_>) -> TestResult {
    let entries = std::fs::read_dir(directory)?
        .take(65)
        .collect::<Result<Vec<_>, _>>()?;
    assert!(entries.len() <= 64);
    entries
        .iter()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "toml"))
        .try_for_each(|entry| validate_file(&entry.path(), catalog))
}

fn validate_file(path: &Path, catalog: &RuntimeCatalog<'_>) -> TestResult {
    let content = std::fs::read_to_string(path)?;
    let scenario = load(&content)?;
    SeedModels::load(catalog, &scenario).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(())
}

#[test]
fn given_request_identity_and_service_override_when_loaded_then_expansion_preserves_both()
-> TestResult {
    let scenario = load(
        "format=1\nservice='sns'\n[[step]]\nid='queue'\nservice='sqs'\noperation='CreateQueue'\nkind='AWS::SQS::Queue'\nidentity={input='QueueName'}\n[step.input]\nQueueName='seed'",
    )?;
    let step = scenario.steps().first().ok_or("step missing")?;
    assert_eq!(step.service, "sqs");
    assert!(
        matches!(&step.identity, Some(crate::inventory_seed::SeedIdentity::Request { input_field, template: None })
        if input_field.parts() == ["QueueName"])
    );
    Ok(())
}
