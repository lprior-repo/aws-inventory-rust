use super::*;
use catalog_tool::{endpoints, kind_map, overlay, runtime_catalog, smithy};
use std::sync::LazyLock;

pub(super) const FORBIDDEN: &[&str] = &[
    "DescribeContainerFleet", "DescribeContainerGroupDefinition", "DescribeContainerGroupPortMappings",
    "DescribeFleetDeployment", "DescribeFleetEvents", "DescribeGameSessionDetails", "DescribeGameSessions",
    "DescribePlayerSessions", "GetPlayerConnectionDetails", "GetGameSessionLogUrl", "GetComputeAccess",
    "GetComputeAuthToken", "GetInstanceAccess", "RequestUploadCredentials", "CreateContainerFleet",
    "UpdateContainerFleet", "DeleteContainerFleet",
];
static CATALOG: LazyLock<Result<Vec<u8>, String>> = LazyLock::new(|| compile().map_err(|error| error.to_string()));

pub(super) fn catalog() -> TestResult<RuntimeCatalog<'static>> {
    match &*CATALOG {
        Ok(bytes) => Ok(RuntimeCatalog::from_bytes(bytes)?),
        Err(error) => Err(format!("pinned container fleet fixture failed: {error}").into()),
    }
}

fn compile() -> TestResult<Vec<u8>> {
    let directory = tempfile::tempdir()?;
    let source = directory.path().join("source");
    let operations = std::iter::once("ListContainerFleets").chain(FORBIDDEN.iter().copied()).collect::<Vec<_>>();
    crate::sweep_runtime::pinned_model::install(&source.join("models"), "gamelift", "2015-10-01", &operations)?;
    let embedded = overlay::embedded()?;
    let policy = embedded.iter().find(|policy| policy.service() == "gamelift").ok_or("embedded GameLift policy absent")?;
    let selected = std::iter::once("ListContainerFleets").chain(FORBIDDEN.iter().copied()).map(|operation| {
        let key = overlay::OperationName::new(operation)?;
        Ok((operation, policy.operations().get(&key).ok_or("embedded operation absent")?.clone()))
    }).collect::<TestResult<Vec<_>>>()?;
    let policies = [overlay::ServiceOverlay::new("gamelift", selected)?];
    let checked = overlay::check_policies(&source, &policies, &catalog_tool::classes::DenyList::builtin())?;
    let fields = fields()?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/cloudcontrol-universal.json")))?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &endpoints::verified_endpoints(&source)?)?)
}

pub(super) fn fields() -> TestResult<smithy::Registry> {
    Ok(smithy::Registry::parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/field-ids.txt")))?)
}

pub(super) fn arn(ordinal: &str) -> String {
    format!("arn:aws:gamelift:us-east-1:123456789012:containerfleet/containerfleet-{ordinal}")
}

pub(super) fn row(ordinal: &str) -> Json {
    // Exercise every pinned ContainerFleet member, including all nested metadata structures.
    // References are not ownership evidence: the role below belongs to another account.
    json!({
        "FleetId":format!("containerfleet-{ordinal}"), "FleetArn":arn(ordinal),
        "FleetRoleArn":"arn:aws:iam::999999999999:role/shared-fleet-runtime",
        "GameServerContainerGroupDefinitionName":"game-server",
        "GameServerContainerGroupDefinitionArn":"arn:aws:gamelift:us-east-1:123456789012:containergroupdefinition/game-server:1",
        "PerInstanceContainerGroupDefinitionName":"support",
        "PerInstanceContainerGroupDefinitionArn":"arn:aws:gamelift:us-east-1:123456789012:containergroupdefinition/support:2",
        "InstanceConnectionPortRange":{"FromPort":5000,"ToPort":5010},
        "InstanceInboundPermissions":[{"FromPort":5000,"ToPort":5010,"IpRange":"192.0.2.0/24","Protocol":"UDP"}],
        "GameServerContainerGroupsPerInstance":1, "MaximumGameServerContainerGroupsPerInstance":4,
        "InstanceType":"c5.large", "BillingType":"ON_DEMAND", "Description":"same-display-description",
        "CreationTime":1.0, "MetricGroups":["test-fleet-metrics"],
        "NewGameSessionProtectionPolicy":"FullProtection",
        "GameSessionCreationLimitPolicy":{"NewGameSessionsPerCreator":2,"PolicyPeriodInMinutes":5},
        "Status":"ACTIVE", "DeploymentDetails":{"LatestDeploymentId":"deployment-1"},
        "LogConfiguration":{"LogDestination":"CLOUDWATCH","S3BucketName":"fleet-log-reference",
            "LogGroupArn":"arn:aws:logs:us-east-1:123456789012:log-group:fleet-reference"},
        "LocationAttributes":[{"Location":"us-west-2","Status":"ACTIVE","PlayerGatewayStatus":"ENABLED"}],
        "PlayerGatewayMode":"ENABLED"
    })
}

pub(super) fn response(page: usize) -> TestResult<Json> {
    Ok(match page {
        0 => json!({"ContainerFleets":[],"NextToken":"second"}),
        1 => json!({"ContainerFleets":[row("first")],"NextToken":"third"}),
        2 => json!({"ContainerFleets":[row("later")]}),
        _ => return Err("unexpected fleet page".into()),
    })
}

// Read persisted members independently of the native decoder/item selector so a dropped
// ContainerFleet field cannot disappear from both the observation and its test oracle.
pub(super) fn json_value(value: ValueRef<'_>, fields: &smithy::Registry) -> TestResult<Json> {
    match value {
        ValueRef::Text(value) => Ok(json!(value)),
        ValueRef::Int(value) => Ok(json!(value)),
        ValueRef::Bool(value) => Ok(json!(value)),
        ValueRef::Float(value) => Ok(json!(value.to_f64())),
        ValueRef::Time(value) => {
            let seconds = f64::from(i32::try_from(value.seconds())?);
            Ok(json!(seconds + f64::from(value.nanoseconds()) / 1_000_000_000.0))
        }
        ValueRef::List(values) => Ok(Json::Array(values.iter()?.map(|value| {
            json_value(value?, fields)
        }).collect::<TestResult<_>>()?)),
        ValueRef::Map(values) => Ok(Json::Object(values.iter()?.map(|entry| {
            let (key, value) = entry?;
            let name = match key {
                KeyRef::Field(id) => fields.name(id).ok_or("unknown persisted field")?.to_owned(),
                KeyRef::Name(name) => name.to_owned(),
            };
            Ok((name, json_value(value, fields)?))
        }).collect::<TestResult<_>>()?)),
        _ => Err("unexpected non-metadata value in GameLift container fleet".into()),
    }
}
