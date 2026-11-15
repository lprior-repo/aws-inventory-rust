use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::{fs, path::Path, sync::LazyLock};

const SERVICES: &[(&str, &str, &[&str])] = &[
    ("deadline", "2023-10-12", &["ListFarms", "ListFleets", "ListQueues", "ListLimits", "ListBudgets", "ListStorageProfiles", "ListQueueEnvironments", "ListQueueFleetAssociations", "ListQueueLimitAssociations", "ListVolumes"]),
    ("evs", "2023-07-27", &["ListEnvironments", "ListEnvironmentHosts", "ListEnvironmentConnectors", "ListEnvironmentVlans"]),
    ("datazone", "2018-05-10", &["ListDomains", "ListProjectProfiles"]),
    ("entityresolution", "2018-05-10", &["ListIdMappingWorkflows", "ListMatchingWorkflows", "ListIdMappingJobs", "ListMatchingJobs"]),
    ("fis", "2020-12-01", &["ListExperimentTemplates", "ListExperiments", "ListTargetAccountConfigurations", "ListExperimentTargetAccountConfigurations"]),
    ("fms", "2018-01-01", &["GetNotificationChannel", "ListResourceSets", "ListResourceSetResources"]),
    ("frauddetector", "2019-11-15", &["GetDetectors", "DescribeDetector"]),
    ("secrets-manager", "2017-10-17", &["ListSecrets"]),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned D-F production policies failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/aws-models/models");
    let embedded = overlay::embedded()?;
    let policies = SERVICES.iter().map(|(service, date, operations)| {
        let relative = format!("{service}/service/{date}/{service}-{date}.json");
        let destination = source.join("models").join(&relative);
        fs::create_dir_all(destination.parent().ok_or("model parent absent")?)?;
        fs::copy(pinned.join(&relative), destination)?;
        let policy = embedded.iter().find(|policy| policy.service() == *service).ok_or("production service absent")?;
        let selected = operations.iter().map(|operation| {
            let key = overlay::OperationName::new(operation)?;
            Ok((*operation, policy.operations().get(&key).ok_or("production operation absent")?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        Ok(overlay::ServiceOverlay::new(service, selected)?)
    }).collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}
