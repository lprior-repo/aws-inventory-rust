use super::*;
pub(super) fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let embedded = overlay::embedded()?;
    let policies = SERVICES
        .iter()
        .map(|(service, date, operations)| {
            crate::pinned_model::install(&source.join("models"), service, date, operations)?;
            let policy = embedded
                .iter()
                .find(|policy| policy.service() == *service)
                .ok_or("production service absent")?;
            let selected = operations
                .iter()
                .map(|operation| {
                    let key = overlay::OperationName::new(operation)?;
                    Ok((
                        *operation,
                        policy
                            .operations()
                            .get(&key)
                            .ok_or("production operation absent")?
                            .clone(),
                    ))
                })
                .collect::<TestResult<Vec<_>>>()?;
            Ok(overlay::ServiceOverlay::new(service, selected)?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(
        &source,
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &endpoints::verified_endpoints(&source)?,
    )?)
}
