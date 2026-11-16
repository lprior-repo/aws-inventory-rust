use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

pub(super) const SERVICES: &[(&str, &str, &[&str])] = &[
    ("gamelift", "2015-10-01", &["ListScripts", "ListLocations", "DescribeScript", "RequestUploadCredentials", "GetComputeAuthToken"]),
    ("imagebuilder", "2019-12-02", &["ListComponentBuildVersions", "ListImageBuildVersions", "ListWorkflowBuildVersions", "ListComponents", "ListImages", "ListWorkflows", "GetComponent", "GetWorkflow", "GetImageRecipe", "StartImagePipelineExecution"]),
    ("glue", "2017-03-31", &["ListBlueprints", "ListDevEndpoints", "ListMLTransforms", "ListTriggers", "ListDataQualityRulesets", "ListUsageProfiles", "GetSecurityConfigurations", "BatchGetBlueprints", "GetDevEndpoint", "GetMLTransform", "GetTrigger", "GetDataQualityRuleset", "GetConnection", "GetJob"]),
    ("glacier", "2012-06-01", &["ListProvisionedCapacity", "PurchaseProvisionedCapacity", "InitiateJob", "GetJobOutput", "ListJobs"]),
    ("inspector2", "2020-06-08", &["ListConnectors", "ListConnectorScanConfigurations", "StartOnDemandScan", "ListFindings", "BatchGetCodeSnippet"]),
    ("invoicing", "2024-12-01", &["ListProcurementPortalPreferences", "GetProcurementPortalPreference", "GetInvoicePDF", "ListProcurementPortals", "SendProcurementPortalValidation"]),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned G-K root fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let embedded = overlay::embedded()?;
    let policies = SERVICES.iter().map(|(service, date, operations)| {
        crate::sweep_runtime::pinned_model::install(&source.join("models"), service, date, operations)?;
        let policy = embedded.iter().find(|policy| policy.service() == *service).ok_or("embedded policy absent")?;
        let selected = operations.iter().map(|operation| {
            let key = overlay::OperationName::new(operation)?;
            Ok((*operation, policy.operations().get(&key).ok_or("embedded operation absent")?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        Ok(overlay::ServiceOverlay::new(service, selected)?)
    }).collect::<TestResult<Vec<_>>>()?;
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}
