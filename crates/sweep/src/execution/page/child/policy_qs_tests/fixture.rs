use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

const SERVICES: &[(&str, &str, &[&str])] = &[
    (
        "quicksight",
        "2018-04-01",
        &[
            "ListActionConnectors",
            "ListAgents",
            "ListApps",
            "ListApprovalPolicies",
            "ListBrands",
            "ListDlpSettings",
            "ListFlows",
            "ListLimitsProfiles",
            "ListOAuthClientApplications",
            "ListSpaces",
            "ListTemplates",
            "ListTopics",
            "ListTopicsV2",
            "ListNamespaces",
            "ListGroups",
            "ListDataSets",
            "ListRefreshSchedules",
        ],
    ),
    (
        "qbusiness",
        "2023-11-27",
        &[
            "ListApplications",
            "ListIndices",
            "ListPlugins",
            "ListWebExperiences",
            "ListRetrievers",
            "ListDataSources",
        ],
    ),
    (
        "swf",
        "2012-01-25",
        &["ListDomains", "ListActivityTypes", "ListWorkflowTypes"],
    ),
    (
        "route53globalresolver",
        "2022-09-27",
        &[
            "ListGlobalResolvers",
            "ListAccessSources",
            "ListFirewallDomainLists",
            "ListHostedZoneAssociations",
            "ListDNSViews",
            "ListFirewallRules",
        ],
    ),
    ("rbin", "2021-06-15", &["ListRules"]),
    (
        "ssm",
        "2014-11-06",
        &[
            "ListDocuments",
            "DescribeMaintenanceWindows",
            "DescribeMaintenanceWindowTargets",
        ],
    ),
    (
        "s3files",
        "2025-05-05",
        &["ListFileSystems", "ListAccessPoints"],
    ),
    (
        "sagemaker",
        "2017-07-24",
        &["ListImages", "ListImageVersions"],
    ),
    (
        "sesv2",
        "2019-09-27",
        &["ListEmailIdentities", "ListEmailIdentityCertificates"],
    ),
    ("ram", "2018-01-04", &["ListPermissions"]),
    (
        "route53-recovery-control-config",
        "2020-11-02",
        &["ListControlPanels", "ListSafetyRules"],
    ),
    (
        "s3tables",
        "2018-05-10",
        &["ListTableBuckets", "ListNamespaces"],
    ),
    (
        "qconnect",
        "2020-10-19",
        &["ListAssistants", "ListAIPrompts", "ListAIPromptVersions"],
    ),
    ("securitylake", "2018-05-10", &["ListDataLakes"]),
    (
        "service-catalog",
        "2015-12-10",
        &[
            "ListPortfolios",
            "ListPrincipalsForPortfolio",
            "ListConstraintsForPortfolio",
        ],
    ),
    ("s3", "2006-03-01", &["ListDirectoryBuckets"]),
    ("rds", "2014-10-31", &["DescribeDBSnapshots"]),
    ("route-53", "2013-04-01", &["ListHostedZones", "GetDNSSEC"]),
    ("kms", "2014-11-01", &["ListKeys"]),
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned QS policy fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
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

pub(super) fn plan<'a, 'c>(
    catalog: &'a RuntimeCatalog<'c>,
    service: &str,
    operation: &str,
) -> TestResult<&'a CallablePlan<'c>> {
    catalog
        .lookup(service, operation)
        .ok_or_else(|| format!("missing production plan {service}.{operation}").into())
}

pub(super) fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{
        request::Protocol,
        response::{DecodeLimits, ResponseContext},
    };
    use inventory_core::catalog::runtime::RuntimeProtocol;
    let protocol = match plan.protocol() {
        RuntimeProtocol::RestXml => Protocol::RestXml,
        RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        RuntimeProtocol::AwsQuery => Protocol::AwsQuery,
        RuntimeProtocol::RpcV2Cbor => Protocol::RpcV2Cbor,
        _ => return Err("unexpected QS fixture wire protocol".into()),
    };
    let headers = http::HeaderMap::new();
    let cbor;
    let body = if matches!(protocol, Protocol::RpcV2Cbor) {
        cbor = crate::test_cbor::json_to_cbor(&serde_json::from_slice(body)?)?;
        cbor.as_slice()
    } else {
        body
    };
    Ok(aws_wire::response::decode(ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body,
        headers: &headers,
        status: http::StatusCode::OK,
        limits: DecodeLimits::default(),
    })?)
}
