use super::*;
pub(super) fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    crate::pinned_model::install(
        &source.join("models"),
        "firehose",
        "2015-08-04",
        &["ListDeliveryStreams", "DescribeDeliveryStream"],
    )?;
    let embedded = overlay::embedded()?;
    let policy = embedded
        .iter()
        .find(|policy| policy.service() == "firehose")
        .ok_or("embedded Firehose policy absent")?;
    let selected = ["ListDeliveryStreams", "DescribeDeliveryStream"]
        .into_iter()
        .map(|name| {
            let operation = overlay::OperationName::new(name)?;
            Ok((
                name,
                policy
                    .operations()
                    .get(&operation)
                    .ok_or("production Firehose operation absent")?
                    .clone(),
            ))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let enumeration = selected
        .iter()
        .find(|(name, _)| *name == "ListDeliveryStreams")
        .ok_or("Firehose enumeration absent")?;
    assert_eq!(
        enumeration.1.paginator,
        Some(overlay::Paginator::LastListItem {
            input: overlay::MemberPath::new("ExclusiveStartDeliveryStreamName")?,
            output: overlay::MemberPath::new("DeliveryStreamNames")?,
            truncated: overlay::MemberPath::new("HasMoreDeliveryStreams")?,
        })
    );
    let policies = [overlay::ServiceOverlay::new("firehose", selected)?];
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
