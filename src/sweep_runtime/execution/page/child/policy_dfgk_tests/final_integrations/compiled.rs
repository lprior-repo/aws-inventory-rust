use super::*;
use catalog_tool::{classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

const SERVICES: &[(&str, &str, &[&str])] = &[
    ("iot-managed-integrations", "2025-03-03", &[
        "ListAccountAssociations", "ListDestinations", "ListEventLogConfigurations", "ListNotificationConfigurations", "ListOtaTaskConfigurations", "ListOtaTasks",
        "GetCredentialLocker", "GetManagedThingState", "GetOtaTask", "GetOtaTaskConfiguration", "SendManagedThingCommand", "ListOtaTaskExecutions",
    ]),
    ("iot-wireless", "2020-11-22", &[
        "ListNetworkAnalyzerConfigurations", "ListWirelessDeviceImportTasks", "ListWirelessGatewayTaskDefinitions",
        "GetNetworkAnalyzerConfiguration", "GetWirelessDevice", "GetPartnerAccount", "ListPartnerAccounts", "UpdateNetworkAnalyzerConfiguration", "SendDataToWirelessDevice",
    ]),
    ("ivs", "2020-07-14", &["ListChannels", "ListStreamKeys", "GetStreamKey", "BatchGetStreamKey", "GetChannel", "GetStreamSession", "CreateStreamKey"]),
    ("ivs-realtime", "2020-07-14", &["ListStages", "ListPublicKeys", "ListIngestConfigurations", "ListCompositions", "GetIngestConfiguration", "CreateParticipantToken", "StartComposition", "GetParticipant"]),
    ("healthlake", "2017-07-01", &["ListDataTransformationProfiles", "ListDataTransformationProfileVersions", "GetDataTransformationProfile", "ListDataTransformationJobs"]),
];

static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(error.clone().into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let embedded = overlay::embedded()?;
    let policies = SERVICES.iter().map(|(service, date, operations)| {
        crate::sweep_runtime::pinned_model::install(&source.join("models"), service, date, operations)?;
        let policy = embedded.iter().find(|policy| policy.service() == *service).ok_or("embedded service absent")?;
        let selected = operations.iter().map(|name| {
            let operation = overlay::OperationName::new(name)?;
            Ok((*name, policy.operations().get(&operation).ok_or_else(|| format!("embedded operation absent: {service}.{name}"))?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        Ok(overlay::ServiceOverlay::new(service, selected)?)
    }).collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(&source, &policies, &DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}

#[test]
fn given_reviewed_summary_plans_when_compiled_then_credentials_content_commands_and_paid_calls_are_not_callable() -> TestResult {
    let catalog = catalog()?;
    for (service, operations) in [
        ("iot-managed-integrations", &["GetCredentialLocker", "GetManagedThingState", "GetOtaTask", "GetOtaTaskConfiguration", "SendManagedThingCommand", "ListOtaTaskExecutions"][..]),
        ("iot-wireless", &["GetNetworkAnalyzerConfiguration", "GetWirelessDevice", "GetPartnerAccount", "ListPartnerAccounts", "UpdateNetworkAnalyzerConfiguration", "SendDataToWirelessDevice"][..]),
        ("ivs", &["GetStreamKey", "BatchGetStreamKey", "GetChannel", "GetStreamSession", "CreateStreamKey"][..]),
        ("ivs-realtime", &["GetIngestConfiguration", "CreateParticipantToken", "StartComposition", "GetParticipant"][..]),
        ("healthlake", &["GetDataTransformationProfile", "TransformData", "ListDataTransformationJobs"][..]),
    ] {
        for operation in operations { assert!(catalog.lookup(service, operation).is_none(), "forbidden plan {service}.{operation}"); }
    }
    Ok(())
}
