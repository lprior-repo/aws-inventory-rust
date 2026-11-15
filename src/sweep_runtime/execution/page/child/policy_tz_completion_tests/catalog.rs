use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::{fs, path::Path, sync::LazyLock};
use super::TestResult;

static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn bytes() -> TestResult<&'static [u8]> {
    CATALOG.as_ref().map(Vec::as_slice).map_err(|error| error.clone().into())
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let pinned = Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/aws-models/models");
    let embedded = overlay::embedded()?;
    let mut policies = Vec::new();
    for (service, date, operations) in [
        ("vpc-lattice", "2022-11-30", &["ListServices", "ListServiceNetworks", "ListListeners", "ListTargetGroups", "ListAccessLogSubscriptions", "ListRules", "ListTargets"][..]),
        ("cloudwatch-logs", "2014-03-28", &["DescribeLogGroups"][..]),
        ("firehose", "2015-08-04", &["ListDeliveryStreams"][..]),
        ("s3", "2006-03-01", &["ListBuckets"][..]),
        ("workmail", "2017-10-01", &["ListOrganizations", "ListGroups", "ListUsers", "ListResources", "ListImpersonationRoles", "ListMailDomains", "ListAvailabilityConfigurations", "ListMobileDeviceAccessOverrides", "ListPersonalAccessTokens", "ListAccessControlRules", "ListMobileDeviceAccessRules", "ListGroupMembers", "ListResourceDelegates", "ListAliases", "ListGroupsForEntity", "ListMailboxPermissions"][..]),
        ("wellarchitected", "2020-03-31", &["ListLenses", "ListAgentProfiles", "ListAgentGoals"][..]),
        ("wisdom", "2020-10-19", &["ListKnowledgeBases", "ListContents", "ListQuickResponses", "ListImportJobs"][..]),
        ("wickr", "2024-02-01", &["ListNetworks", "ListBots"][..]),
        ("voice-id", "2021-09-27", &["ListDomains", "ListWatchlists", "ListSpeakers", "ListFraudsters", "ListFraudsterRegistrationJobs", "ListSpeakerEnrollmentJobs"][..]),
        ("workspaces", "2015-04-08", &["DescribeWorkspaceDirectories", "DescribeConnectClientAddIns", "DescribeWorkspacesPools", "DescribeWorkspacesPoolSessions", "DescribeApplications", "DescribeApplicationAssociations", "DescribeWorkspaceBundles", "DescribeBundleAssociations", "DescribeWorkspaceImages", "DescribeImageAssociations", "DescribeWorkspaces", "DescribeWorkspaceAssociations"][..]),
        ("waf", "2015-08-24", &["ListLoggingConfigurations", "ListWebACLs", "GetWebACL"][..]),
        ("waf-regional", "2016-11-28", &["ListLoggingConfigurations", "ListWebACLs", "GetWebACL", "ListResourcesForWebACL"][..]),
    ] {
        let relative = format!("{service}/service/{date}/{service}-{date}.json");
        let destination = directory.path().join("models").join(&relative);
        fs::create_dir_all(destination.parent().ok_or("model parent absent")?)?;
        fs::copy(pinned.join(relative), destination)?;
        let policy = embedded.iter().find(|policy| policy.service() == service).ok_or("production service absent")?;
        let operations = operations.iter().map(|name| {
            let key = overlay::OperationName::new(name)?;
            Ok((*name, policy.operations().get(&key).ok_or("production operation absent")?.clone()))
        }).collect::<TestResult<Vec<_>>>()?;
        policies.push(overlay::ServiceOverlay::new(service, operations)?);
    }
    let checked = overlay::check_policies(directory.path(), &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?;
    let base = smithy::build_catalog(directory.path(), &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(directory.path())?)?)
}
