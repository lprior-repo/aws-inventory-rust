//! ACTIVE ECS task-definition revisions, with separate list/description coverage.
//! SDK responses may contain environment values; only explicit reference and
//! architecture fields below are retained in inventory events.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::Mutex;

use aws_sdk_ecs::{
    Client,
    operation::{
        describe_task_definition::DescribeTaskDefinitionOutput,
        list_task_definitions::ListTaskDefinitionsOutput,
    },
    types::{
        ContainerDefinition, LogConfiguration, Secret, SortOrder, TaskDefinitionField,
        TaskDefinitionStatus, Volume,
    },
};
use aws_smithy_types::DateTime;
use inventory_core::{Edge, Resource, Source, validate_identifier};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{
        Context, MAX_CHILDREN, PageBody, collect_bounded, paginate, require_budget, required,
        sdk_error,
    },
    ecs::{merge_reference_edge, tags},
};

const DESCRIBE_CONCURRENCY: usize = 4;
const MAX_IDENTITY_BYTES: usize = 8_388_608;
const MAX_COMPONENTS: usize = 4_096;
const LIST_SCOPE: &str = "aws_ecs_task_definition_list";
const DEFINITION_KIND: &str = "aws_ecs_task_definition";

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    collect_with(
        ctx,
        |token| {
            let client = &client;
            async move {
                ctx.execute("ecs", "read", || async {
                    client
                        .list_task_definitions()
                        .status(TaskDefinitionStatus::Active)
                        .sort(SortOrder::Asc)
                        .max_results(ctx.page_size.clamp(1, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await
            }
        },
        |arn: String| {
            let client = &client;
            async move {
                ctx.execute("ecs", "read", || async {
                    client
                        .describe_task_definition()
                        .task_definition(arn.clone())
                        .include(TaskDefinitionField::Tags)
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await
            }
        },
    )
    .await
}

/// The only scan-wide state is a bounded identity ledger. No SDK definitions or
/// mapped resources accumulate across pages; descriptions stream independently.
#[derive(Default)]
struct ListedIdentities {
    arns: BTreeSet<String>,
    bytes: usize,
}

fn remember_list_page(
    seen: &Mutex<ListedIdentities>,
    arns: &[String],
    budget: usize,
) -> Result<(), AwsScanError> {
    require_budget(arns.len(), budget)?;
    if arns.len() > 100 {
        return Err(AwsScanError::InvalidData(
            "ListTaskDefinitions exceeded its 100-result API page limit".into(),
        ));
    }
    let mut seen = seen.lock().map_err(|_| {
        AwsScanError::InvalidData("task-definition identity ledger was poisoned".into())
    })?;
    if seen.arns.len().saturating_add(arns.len()) > MAX_CHILDREN {
        return Err(AwsScanError::ChildLimit);
    }
    let mut page_ids = BTreeSet::new();
    let mut bytes = seen.bytes;
    for arn in arns {
        definition_identity(arn)?;
        if seen.arns.contains(arn) || !page_ids.insert(arn.as_str()) {
            return Err(AwsScanError::InvalidData(
                "ListTaskDefinitions repeated a revision ARN".into(),
            ));
        }
        bytes = bytes.checked_add(arn.len()).ok_or_else(|| {
            AwsScanError::InvalidData("task-definition identity byte count overflow".into())
        })?;
        if bytes > MAX_IDENTITY_BYTES {
            return Err(AwsScanError::InvalidData(
                "task-definition identity history exceeds 8 MiB".into(),
            ));
        }
    }
    seen.arns.extend(arns.iter().cloned());
    seen.bytes = bytes;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn collect_with<L, LFut, D, DFut>(
    ctx: &Context,
    list: L,
    describe: D,
) -> Result<(), AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListTaskDefinitionsOutput, ApiError>> + Send,
    D: Fn(String) -> DFut + Send + Sync,
    DFut: Future<Output = Result<DescribeTaskDefinitionOutput, ApiError>> + Send,
{
    let seen = Mutex::new(ListedIdentities::default());
    paginate(
        ctx,
        ctx.scope(LIST_SCOPE, Some("status=ACTIVE"), Source::Direct),
        |token, budget| {
            let list = &list;
            let describe = &describe;
            let seen = &seen;
            async move {
                let output = list(token).await?;
                remember_list_page(seen, output.task_definition_arns(), budget)?;
                collect_bounded(
                    output.task_definition_arns().iter().cloned(),
                    DESCRIBE_CONCURRENCY,
                    |arn| async move { describe_with(ctx, &arn, || describe(arn.clone())).await },
                )
                .await?;
                Ok(PageBody {
                    listed_count: output.task_definition_arns().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(task_definition_arn = arn))]
async fn describe_with<F, Fut>(ctx: &Context, arn: &str, fetch: F) -> Result<(), AwsScanError>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result<DescribeTaskDefinitionOutput, ApiError>> + Send,
{
    paginate(
        ctx,
        ctx.scope(DEFINITION_KIND, Some(arn), Source::Direct),
        |token, budget| {
            let fetch = &fetch;
            async move {
                if token.is_some() {
                    return Err(AwsScanError::InvalidData(
                        "DescribeTaskDefinition is not paginated".into(),
                    ));
                }
                require_budget(1, budget)?;
                let output = fetch().await?;
                map_definition(ctx, arn, &output)
            }
        },
    )
    .await
}

struct ArnParts<'a> {
    service: &'a str,
    region: &'a str,
    account: &'a str,
    resource: &'a str,
}

fn arn_parts(arn: &str) -> Result<ArnParts<'_>, AwsScanError> {
    validate_identifier(arn, "task-definition related ARN")?;
    let mut parts = arn.splitn(6, ':');
    let (Some("arn"), Some(partition), Some(service), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(AwsScanError::InvalidData(
            "task-definition response contains an invalid ARN".into(),
        ));
    };
    if partition.is_empty() || service.is_empty() || account.is_empty() || resource.is_empty() {
        return Err(AwsScanError::InvalidData(
            "task-definition response contains an incomplete ARN".into(),
        ));
    }
    Ok(ArnParts {
        service,
        region,
        account,
        resource,
    })
}

struct DefinitionIdentity<'a> {
    account: &'a str,
    region: &'a str,
    family: &'a str,
    revision: i32,
}

fn definition_identity(arn: &str) -> Result<DefinitionIdentity<'_>, AwsScanError> {
    let parts = arn_parts(arn)?;
    let (family, revision) = parts
        .resource
        .strip_prefix("task-definition/")
        .and_then(|value| value.rsplit_once(':'))
        .ok_or_else(|| {
            AwsScanError::InvalidData(
                "task-definition identity lacks family and explicit revision".into(),
            )
        })?;
    let revision = revision.parse::<i32>().map_err(|_| {
        AwsScanError::InvalidData("task-definition revision is not a positive integer".into())
    })?;
    if parts.service != "ecs"
        || parts.region.is_empty()
        || family.is_empty()
        || family.contains(':')
        || revision <= 0
    {
        return Err(AwsScanError::InvalidData(
            "invalid ECS task-definition revision ARN".into(),
        ));
    }
    Ok(DefinitionIdentity {
        account: parts.account,
        region: parts.region,
        family,
        revision,
    })
}

fn consume(budget: &mut usize, count: usize) -> Result<(), AwsScanError> {
    *budget = budget.checked_sub(count).ok_or_else(|| {
        AwsScanError::InvalidData("task-definition mapped component capacity exceeded".into())
    })?;
    Ok(())
}

fn millis(value: Option<&DateTime>) -> Result<Option<i64>, AwsScanError> {
    value
        .map(|value| value.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(format!("task-definition timestamp: {error}")))
}

fn option_keys(values: Option<&HashMap<String, String>>) -> Vec<&str> {
    let mut keys: Vec<_> = values
        .into_iter()
        .flat_map(|values| values.keys().map(String::as_str))
        .collect();
    keys.sort_unstable();
    keys
}

fn secret_references(values: &[Secret]) -> Vec<Value> {
    values
        .iter()
        .map(|value| json!({"name":value.name(),"value_from":value.value_from()}))
        .collect()
}

fn map_logs(config: &LogConfiguration, budget: &mut usize) -> Result<Value, AwsScanError> {
    consume(
        budget,
        config
            .options()
            .map_or(0, HashMap::len)
            .saturating_add(config.secret_options().len()),
    )?;
    let awslogs: BTreeMap<_, _> = config
        .options()
        .into_iter()
        .flat_map(|values| values.iter())
        .filter(|(key, _)| {
            matches!(
                key.as_str(),
                "awslogs-group" | "awslogs-region" | "awslogs-stream-prefix"
            )
        })
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    Ok(
        json!({"log_driver":config.log_driver().as_str(),"option_keys":option_keys(config.options()),
        "awslogs":awslogs,"secret_references":secret_references(config.secret_options()),
        "other_option_values_collected":false,"secret_values_collected":false}),
    )
}

fn map_container(
    container: &ContainerDefinition,
    budget: &mut usize,
) -> Result<Value, AwsScanError> {
    let count = [
        container.port_mappings().len(),
        container.mount_points().len(),
        container.volumes_from().len(),
        container.environment().len(),
        container.environment_files().len(),
        container.secrets().len(),
        container.depends_on().len(),
        container.links().len(),
        container.dns_servers().len(),
        container.dns_search_domains().len(),
        container.extra_hosts().len(),
        container.credential_specs().len(),
        container.resource_requirements().len(),
        container.docker_labels().map_or(0, HashMap::len),
        container
            .firelens_configuration()
            .and_then(|value| value.options())
            .map_or(0, HashMap::len),
    ]
    .into_iter()
    .fold(0usize, usize::saturating_add);
    consume(budget, count)?;
    let logs = container
        .log_configuration()
        .map(|config| map_logs(config, budget))
        .transpose()?;
    Ok(json!({
        "name":container.name(),"image":container.image(),"essential":container.essential(),"cpu":container.cpu(),
        "memory":container.memory(),"memory_reservation":container.memory_reservation(),
        "port_mappings":container.port_mappings().iter().map(|port| json!({
            "name":port.name(),"container_port":port.container_port(),"host_port":port.host_port(),
            "protocol":port.protocol().map(|value|value.as_str()),"app_protocol":port.app_protocol().map(|value|value.as_str()),
            "container_port_range":port.container_port_range(),
        })).collect::<Vec<_>>(),
        "mount_points":container.mount_points().iter().map(|mount| json!({"source_volume":mount.source_volume(),"container_path":mount.container_path(),"read_only":mount.read_only()})).collect::<Vec<_>>(),
        "volumes_from":container.volumes_from().iter().map(|value| json!({"source_container":value.source_container(),"read_only":value.read_only()})).collect::<Vec<_>>(),
        "depends_on":container.depends_on().iter().map(|value| json!({"container_name":value.container_name(),"condition":value.condition().as_str()})).collect::<Vec<_>>(),
        "links":container.links(),"hostname":container.hostname(),"disable_networking":container.disable_networking(),
        "dns_servers":container.dns_servers(),"dns_search_domains":container.dns_search_domains(),
        "extra_hosts":container.extra_hosts().iter().map(|value| json!({"hostname":value.hostname(),"ip_address":value.ip_address()})).collect::<Vec<_>>(),
        "privileged":container.privileged(),"readonly_root_filesystem":container.readonly_root_filesystem(),"user":container.user(),
        "start_timeout":container.start_timeout(),"stop_timeout":container.stop_timeout(),
        "version_consistency":container.version_consistency().map(|value|value.as_str()),
        "environment_names":container.environment().iter().filter_map(|value|value.name()).collect::<Vec<_>>(),
        "environment_values_collected":false,
        "environment_files":container.environment_files().iter().map(|value| json!({"value":value.value(),"type":value.r#type().as_str()})).collect::<Vec<_>>(),
        "secret_references":secret_references(container.secrets()),"secret_values_collected":false,
        "repository_credentials_reference":container.repository_credentials().map(|value|value.credentials_parameter()),
        "credential_specs":container.credential_specs(),
        "command_present":!container.command().is_empty(),"entry_point_present":!container.entry_point().is_empty(),"command_values_collected":false,
        "health_check":container.health_check().map(|value|json!({"command_present":!value.command().is_empty(),"command_values_collected":false,
            "interval":value.interval(),"timeout":value.timeout(),"retries":value.retries(),"start_period":value.start_period()})),
        "docker_label_keys":option_keys(container.docker_labels()),"docker_label_values_collected":false,
        "log_configuration":logs,
        "firelens_configuration":container.firelens_configuration().map(|value|json!({"type":value.r#type().as_str(),"option_keys":option_keys(value.options()),"option_values_collected":false})),
        "resource_requirements":container.resource_requirements().iter().map(|value|json!({"type":value.r#type().as_str(),"value":value.value()})).collect::<Vec<_>>(),
    }))
}

fn map_volume(volume: &Volume, budget: &mut usize) -> Result<Value, AwsScanError> {
    let docker = volume.docker_volume_configuration();
    consume(
        budget,
        docker
            .and_then(|value| value.driver_opts())
            .map_or(0, HashMap::len)
            .saturating_add(
                docker
                    .and_then(|value| value.labels())
                    .map_or(0, HashMap::len),
            ),
    )?;
    Ok(
        json!({"name":volume.name(),"configured_at_launch":volume.configured_at_launch(),
            "host":volume.host().map(|value|json!({"source_path":value.source_path()})),
            "docker_volume_configuration":docker.map(|value|json!({"driver":value.driver(),"scope":value.scope().map(|value|value.as_str()),
                "autoprovision":value.autoprovision(),"driver_option_keys":option_keys(value.driver_opts()),"label_keys":option_keys(value.labels()),
                "driver_option_values_collected":false,"label_values_collected":false})),
            "efs_volume_configuration":volume.efs_volume_configuration().map(|value|json!({
                "file_system_id":value.file_system_id(),"root_directory":value.root_directory(),
                "transit_encryption":value.transit_encryption().map(|value|value.as_str()),"transit_encryption_port":value.transit_encryption_port(),
                "authorization_config":value.authorization_config().map(|value|json!({"access_point_id":value.access_point_id(),"iam":value.iam().map(|value|value.as_str())})),
            })),
            "fsx_windows_file_server_volume_configuration":volume.fsx_windows_file_server_volume_configuration().map(|value|json!({
                "file_system_id":value.file_system_id(),"root_directory":value.root_directory(),
                "authorization_config":value.authorization_config().map(|value|json!({"credentials_parameter":value.credentials_parameter(),"domain":value.domain()})),
            })),
            "s3files_volume_configuration":volume.s3files_volume_configuration().map(|value|json!({
                "file_system_arn":value.file_system_arn(),"access_point_arn":value.access_point_arn(),
                "root_directory":value.root_directory(),"transit_encryption_port":value.transit_encryption_port(),
            })),
        }),
    )
}

fn returned_arn_edge(
    ctx: &Context,
    resource: &Resource,
    arn: &str,
    relation: &str,
    kind: &str,
) -> Result<Edge, AwsScanError> {
    let parts = arn_parts(arn)?;
    if kind == "aws_iam_role"
        && (parts.service != "iam"
            || !parts.region.is_empty()
            || !parts
                .resource
                .strip_prefix("role/")
                .is_some_and(|name| !name.is_empty()))
    {
        return Err(AwsScanError::InvalidData(
            "task-definition role is not an IAM role ARN".into(),
        ));
    }
    if kind == "aws_s3files_file_system"
        && (parts.service != "s3files"
            || parts.region.is_empty()
            || !parts
                .resource
                .strip_prefix("file-system/fs-")
                .is_some_and(|id| !id.is_empty()))
    {
        return Err(AwsScanError::InvalidData(
            "task-definition S3 Files volume is not a file-system ARN".into(),
        ));
    }
    let mut edge = ctx.edge(resource, arn.to_owned(), relation);
    edge.properties = json!({"target_resource_id":arn,"target_kind":kind,"target_account_id":parts.account,
        "target_region":parts.region,"owner_resolution":"aws_returned_arn"});
    Ok(edge)
}

fn file_system_edge(
    ctx: &Context,
    resource: &Resource,
    id: &str,
    service: &str,
    relation: &str,
    kind: &str,
) -> Result<Edge, AwsScanError> {
    let id = required(Some(id), "volume file-system ID")?;
    let mut edge = ctx.edge(
        resource,
        ctx.arn(service, &format!("file-system/{id}")),
        relation,
    );
    edge.properties = json!({"target_resource_id":id,"target_kind":kind,"target_account_id":ctx.account,
        "target_region":ctx.region,"owner_resolution":"assumed_scanning_account"});
    Ok(edge)
}

fn map_definition(
    ctx: &Context,
    requested: &str,
    output: &DescribeTaskDefinitionOutput,
) -> Result<PageBody, AwsScanError> {
    let definition = output.task_definition().ok_or_else(|| {
        AwsScanError::InvalidData("DescribeTaskDefinition lacks taskDefinition".into())
    })?;
    let arn = required(definition.task_definition_arn(), "task-definition ARN")?;
    if arn != requested {
        return Err(AwsScanError::InvalidData(
            "DescribeTaskDefinition returned a different revision than requested".into(),
        ));
    }
    let identity = definition_identity(&arn)?;
    if definition.family() != Some(identity.family) || definition.revision() != identity.revision {
        return Err(AwsScanError::InvalidData(
            "task-definition ARN disagrees with family or revision fields".into(),
        ));
    }
    let mut budget = MAX_COMPONENTS;
    consume(
        &mut budget,
        [
            definition.container_definitions().len(),
            definition.volumes().len(),
            definition.requires_attributes().len(),
            definition.requires_compatibilities().len(),
            definition.compatibilities().len(),
            definition.placement_constraints().len(),
            definition.inference_accelerators().len(),
            output.tags().len(),
            definition
                .proxy_configuration()
                .map_or(0, |value| value.properties().len()),
        ]
        .into_iter()
        .fold(0usize, usize::saturating_add),
    )?;
    let containers = definition
        .container_definitions()
        .iter()
        .map(|value| map_container(value, &mut budget))
        .collect::<Result<Vec<_>, _>>()?;
    let volumes = definition
        .volumes()
        .iter()
        .map(|value| map_volume(value, &mut budget))
        .collect::<Result<Vec<_>, _>>()?;
    let mut resource = ctx.resource(DEFINITION_KIND, arn.clone(), Some(format!("{}:{}", identity.family, identity.revision)), json!({
        "task_definition_arn":arn,"family":definition.family(),"revision":definition.revision(),"status":definition.status().map(|value|value.as_str()),
        "discovery_status_filter":"ACTIVE","task_role_arn":definition.task_role_arn(),"execution_role_arn":definition.execution_role_arn(),
        "network_mode":definition.network_mode().map(|value|value.as_str()),"cpu":definition.cpu(),"memory":definition.memory(),
        "requires_compatibilities":definition.requires_compatibilities().iter().map(|value|value.as_str()).collect::<Vec<_>>(),
        "compatibilities":definition.compatibilities().iter().map(|value|value.as_str()).collect::<Vec<_>>(),
        "runtime_platform":definition.runtime_platform().map(|value|json!({"cpu_architecture":value.cpu_architecture().map(|value|value.as_str()),"operating_system_family":value.operating_system_family().map(|value|value.as_str())})),
        "container_definitions":containers,"volumes":volumes,
        "ephemeral_storage":definition.ephemeral_storage().map(|value|json!({"size_in_gi_b":value.size_in_gib()})),
        "pid_mode":definition.pid_mode().map(|value|value.as_str()),"ipc_mode":definition.ipc_mode().map(|value|value.as_str()),
        "requires_attribute_names":definition.requires_attributes().iter().map(|value|value.name()).collect::<Vec<_>>(),
        "placement_constraints":definition.placement_constraints().iter().map(|value|json!({"type":value.r#type().map(|value|value.as_str()),"expression":value.expression()})).collect::<Vec<_>>(),
        "inference_accelerators":definition.inference_accelerators().iter().map(|value|json!({"device_name":value.device_name(),"device_type":value.device_type()})).collect::<Vec<_>>(),
        "proxy_configuration":definition.proxy_configuration().map(|value|json!({"type":value.r#type().map(|value|value.as_str()),"container_name":value.container_name(),
            "property_names":value.properties().iter().filter_map(|value|value.name()).collect::<Vec<_>>(),"property_values_collected":false})),
        "registered_at":millis(definition.registered_at())?,"deregistered_at":millis(definition.deregistered_at())?,
        "delete_requested_at":millis(definition.delete_requested_at())?,"registered_by":definition.registered_by(),"enable_fault_injection":definition.enable_fault_injection(),
        "environment_values_collected":false,"secret_values_collected":false,
    }), tags(output.tags()))?;
    resource.account_id = identity.account.to_owned();
    resource.region = identity.region.to_owned();
    resource.validate()?;
    let mut edges = BTreeMap::new();
    for (arn, relation, field) in [
        (
            definition.task_role_arn(),
            "USES_TASK_ROLE",
            "task_role_arn",
        ),
        (
            definition.execution_role_arn(),
            "USES_EXECUTION_ROLE",
            "execution_role_arn",
        ),
    ] {
        if let Some(arn) = arn {
            merge_reference_edge(
                &mut edges,
                returned_arn_edge(ctx, &resource, arn, relation, "aws_iam_role")?,
                json!({"field":field}),
            )?;
        }
    }
    for (index, volume) in definition.volumes().iter().enumerate() {
        let evidence = json!({"volume_index":index,"volume_name":volume.name(),"configuration":"declared_volume"});
        if let Some(efs) = volume.efs_volume_configuration() {
            merge_reference_edge(
                &mut edges,
                file_system_edge(
                    ctx,
                    &resource,
                    efs.file_system_id(),
                    "elasticfilesystem",
                    "MOUNTS_EFS",
                    "aws_efs_file_system",
                )?,
                evidence.clone(),
            )?;
        }
        if let Some(fsx) = volume.fsx_windows_file_server_volume_configuration() {
            merge_reference_edge(
                &mut edges,
                file_system_edge(
                    ctx,
                    &resource,
                    fsx.file_system_id(),
                    "fsx",
                    "MOUNTS_FSX",
                    "aws_fsx_file_system",
                )?,
                evidence.clone(),
            )?;
        }
        if let Some(files) = volume.s3files_volume_configuration() {
            merge_reference_edge(
                &mut edges,
                returned_arn_edge(
                    ctx,
                    &resource,
                    files.file_system_arn(),
                    "MOUNTS_S3_FILES",
                    "aws_s3files_file_system",
                )?,
                evidence,
            )?;
        }
    }
    let edges = edges.into_values().collect::<Vec<_>>();
    for edge in &edges {
        edge.validate()?;
    }
    Ok(PageBody {
        resources: vec![resource],
        edges,
        listed_count: 1,
        next_token: None,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use aws_sdk_ecs::types::{
        Compatibility, CpuArchitecture, DockerVolumeConfiguration, EfsVolumeConfiguration,
        EnvironmentFile, EnvironmentFileType, FSxWindowsFileServerAuthorizationConfig,
        FSxWindowsFileServerVolumeConfiguration, FirelensConfiguration, FirelensConfigurationType,
        HealthCheck, KeyValuePair, LogDriver, MountPoint, NetworkMode, OsFamily, PortMapping,
        ProxyConfiguration, RepositoryCredentials, RuntimePlatform, S3FilesVolumeConfiguration,
        TaskDefinition,
    };
    use inventory_core::CollectionEvent;
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use tokio::sync::mpsc;

    use super::*;

    type TestError = Box<dyn std::error::Error>;
    type TestResult = Result<(), TestError>;
    const ARN: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:7";
    const SECOND: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:8";
    const ROLE: &str = "arn:aws:iam::222222222222:role/application/web";
    const SECRET_REF: &str = "arn:aws:secretsmanager:us-east-1:111111111111:secret:database-ref";

    fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(32);
        Ok((
            Context {
                config: aws_types::SdkConfig::builder().build(),
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "111111111111".into(),
                partition: "aws".into(),
                region: "us-east-1".into(),
                max_pages,
                page_size: 100,
            },
            receiver,
        ))
    }

    fn definition(arn: &str, revision: i32) -> TaskDefinition {
        TaskDefinition::builder()
            .task_definition_arn(arn)
            .family("web")
            .revision(revision)
            .status(TaskDefinitionStatus::Active)
            .build()
    }

    fn output(definition: TaskDefinition) -> DescribeTaskDefinitionOutput {
        DescribeTaskDefinitionOutput::builder()
            .task_definition(definition)
            .build()
    }

    fn resource(body: &PageBody) -> Result<&Resource, TestError> {
        body.resources
            .first()
            .ok_or_else(|| "missing task definition".into())
    }

    fn drain(
        receiver: &mut mpsc::Receiver<CollectionEvent>,
    ) -> Result<Vec<CollectionEvent>, TestError> {
        let mut events = Vec::new();
        for _ in 0..32 {
            match receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                    return Ok(events);
                }
            }
        }
        Err("synthetic event fixture exceeded its bound".into())
    }

    #[test]
    fn revision_identity_and_architecture_fields_use_returned_arn() -> TestResult {
        let (ctx, _events) = context(3)?;
        let arn = "arn:aws:ecs:eu-west-1:222222222222:task-definition/web:7";
        let model = TaskDefinition::builder()
            .task_definition_arn(arn)
            .family("web")
            .revision(7)
            .status(TaskDefinitionStatus::Inactive)
            .task_role_arn(ROLE)
            .execution_role_arn(ROLE)
            .network_mode(NetworkMode::Awsvpc)
            .cpu("256")
            .memory("512")
            .requires_compatibilities(Compatibility::Fargate)
            .runtime_platform(
                RuntimePlatform::builder()
                    .cpu_architecture(CpuArchitecture::Arm64)
                    .operating_system_family(OsFamily::Linux)
                    .build(),
            )
            .container_definitions(
                ContainerDefinition::builder()
                    .name("web")
                    .image("example.test/web:v7")
                    .essential(true)
                    .port_mappings(
                        PortMapping::builder()
                            .container_port(8080)
                            .host_port(8080)
                            .build(),
                    )
                    .mount_points(
                        MountPoint::builder()
                            .source_volume("data")
                            .container_path("/data")
                            .read_only(true)
                            .build(),
                    )
                    .build(),
            )
            .registered_at(DateTime::from_secs(1_700_000_000))
            .build();
        let body = map_definition(&ctx, arn, &output(model))?;
        let resource = resource(&body)?;
        assert_eq!(resource.id, arn);
        assert_eq!(resource.name.as_deref(), Some("web:7"));
        assert_eq!(resource.account_id, "222222222222");
        assert_eq!(resource.region, "eu-west-1");
        assert_eq!(resource.properties.get("status"), Some(&json!("INACTIVE")));
        assert_eq!(
            resource.properties.get("discovery_status_filter"),
            Some(&json!("ACTIVE"))
        );
        assert_eq!(
            resource.properties.get("registered_at"),
            Some(&json!(1_700_000_000_000i64))
        );
        assert_eq!(
            resource
                .properties
                .pointer("/runtime_platform/cpu_architecture"),
            Some(&json!("ARM64"))
        );
        assert_eq!(
            resource
                .properties
                .pointer("/container_definitions/0/port_mappings/0/container_port"),
            Some(&json!(8080))
        );
        assert_eq!(
            resource
                .properties
                .pointer("/container_definitions/0/mount_points/0/read_only"),
            Some(&json!(true))
        );
        assert_eq!(body.edges.len(), 2);
        assert!(body.edges.iter().all(|edge| edge.to_id == ROLE
            && edge.properties.get("owner_resolution") == Some(&json!("aws_returned_arn"))));
        assert!(body.edges.iter().all(|edge| edge.relation != "IN_VPC"));
        Ok(())
    }

    #[test]
    fn description_requires_exact_family_and_revision_identity() -> TestResult {
        let (ctx, _events) = context(3)?;
        for model in [
            definition(SECOND, 8),
            definition(ARN, 8),
            TaskDefinition::builder()
                .task_definition_arn(ARN)
                .family("wrong")
                .revision(7)
                .build(),
        ] {
            assert!(matches!(
                map_definition(&ctx, ARN, &output(model)),
                Err(AwsScanError::InvalidData(_))
            ));
        }
        assert!(
            map_definition(&ctx, ARN, &DescribeTaskDefinitionOutput::builder().build()).is_err()
        );
        for invalid in [
            "web:7",
            "arn:aws:ecs:us-east-1:111111111111:task-definition/web",
            "arn:aws:ecs:us-east-1:111111111111:task-definition/web:0",
            "arn:aws:lambda:us-east-1:111111111111:task-definition/web:7",
        ] {
            assert!(definition_identity(invalid).is_err());
        }
        Ok(())
    }

    #[test]
    fn environment_and_unstructured_option_values_are_not_retained() -> TestResult {
        let (ctx, _events) = context(3)?;
        let marker = "sensitive-fixture-value-never-retain";
        let container = ContainerDefinition::builder()
            .name("web")
            .environment(
                KeyValuePair::builder()
                    .name("DATABASE_PASSWORD")
                    .value(marker)
                    .build(),
            )
            .command(marker)
            .entry_point(marker)
            .health_check(
                HealthCheck::builder()
                    .command(marker)
                    .interval(30)
                    .build()?,
            )
            .docker_labels("custom-label", marker)
            .environment_files(
                EnvironmentFile::builder()
                    .value("arn:aws:s3:::config-bucket/web.env")
                    .r#type(EnvironmentFileType::S3)
                    .build()?,
            )
            .secrets(
                Secret::builder()
                    .name("PASSWORD_REF")
                    .value_from(SECRET_REF)
                    .build()?,
            )
            .repository_credentials(
                RepositoryCredentials::builder()
                    .credentials_parameter(SECRET_REF)
                    .build()?,
            )
            .log_configuration(
                LogConfiguration::builder()
                    .log_driver(LogDriver::Awslogs)
                    .options("awslogs-group", "/ecs/web")
                    .options("custom-token", marker)
                    .secret_options(
                        Secret::builder()
                            .name("log-token")
                            .value_from(SECRET_REF)
                            .build()?,
                    )
                    .build()?,
            )
            .firelens_configuration(
                FirelensConfiguration::builder()
                    .r#type(FirelensConfigurationType::Fluentbit)
                    .options("custom-token", marker)
                    .build()?,
            )
            .build();
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .container_definitions(container)
            .proxy_configuration(
                ProxyConfiguration::builder()
                    .container_name("proxy")
                    .properties(
                        KeyValuePair::builder()
                            .name("ProxyProperty")
                            .value(marker)
                            .build(),
                    )
                    .build()?,
            )
            .volumes(
                Volume::builder()
                    .name("local")
                    .docker_volume_configuration(
                        DockerVolumeConfiguration::builder()
                            .driver("local")
                            .driver_opts("credential", marker)
                            .labels("secret-label", marker)
                            .build(),
                    )
                    .build(),
            )
            .build();
        let body = map_definition(&ctx, ARN, &output(model))?;
        let properties = &resource(&body)?.properties;
        let encoded = serde_json::to_string(properties)?;
        assert!(!encoded.contains(marker));
        assert!(encoded.contains(SECRET_REF));
        assert_eq!(
            properties.pointer("/container_definitions/0/environment_names"),
            Some(&json!(["DATABASE_PASSWORD"]))
        );
        assert_eq!(
            properties.pointer("/container_definitions/0/log_configuration/awslogs/awslogs-group"),
            Some(&json!("/ecs/web"))
        );
        assert_eq!(
            properties.pointer("/container_definitions/0/command_present"),
            Some(&json!(true))
        );
        assert_eq!(
            properties.pointer("/container_definitions/0/environment_values_collected"),
            Some(&json!(false))
        );
        assert_eq!(
            properties.pointer("/proxy_configuration/property_names"),
            Some(&json!(["ProxyProperty"]))
        );
        assert_eq!(
            properties.pointer("/volumes/0/docker_volume_configuration/driver_option_keys"),
            Some(&json!(["credential"]))
        );
        Ok(())
    }

    #[test]
    fn volume_references_aggregate_and_preserve_authoritative_or_assumed_owners() -> TestResult {
        let (ctx, _events) = context(3)?;
        let efs = EfsVolumeConfiguration::builder()
            .file_system_id("fs-efs")
            .root_directory("/shared")
            .build()?;
        let files = "arn:aws:s3files:us-west-2:222222222222:file-system/fs-files";
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .volumes(
                Volume::builder()
                    .name("first")
                    .efs_volume_configuration(efs.clone())
                    .build(),
            )
            .volumes(
                Volume::builder()
                    .name("second")
                    .efs_volume_configuration(efs)
                    .build(),
            )
            .volumes(
                Volume::builder()
                    .name("windows")
                    .fsx_windows_file_server_volume_configuration(
                        FSxWindowsFileServerVolumeConfiguration::builder()
                            .file_system_id("fs-windows")
                            .root_directory("share")
                            .authorization_config(
                                FSxWindowsFileServerAuthorizationConfig::builder()
                                    .credentials_parameter(SECRET_REF)
                                    .domain("example.test")
                                    .build()?,
                            )
                            .build()?,
                    )
                    .build(),
            )
            .volumes(
                Volume::builder()
                    .name("files")
                    .s3files_volume_configuration(
                        S3FilesVolumeConfiguration::builder()
                            .file_system_arn(files)
                            .root_directory("/")
                            .build()?,
                    )
                    .build(),
            )
            .build();
        let body = map_definition(&ctx, ARN, &output(model))?;
        assert_eq!(body.edges.len(), 3);
        let efs = body
            .edges
            .iter()
            .find(|edge| edge.relation == "MOUNTS_EFS")
            .ok_or("missing EFS reference")?;
        assert_eq!(
            efs.to_id,
            "arn:aws:elasticfilesystem:us-east-1:111111111111:file-system/fs-efs"
        );
        assert_eq!(
            efs.properties.get("owner_resolution"),
            Some(&json!("assumed_scanning_account"))
        );
        assert_eq!(
            efs.properties
                .get("evidence")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        let files_edge = body
            .edges
            .iter()
            .find(|edge| edge.relation == "MOUNTS_S3_FILES")
            .ok_or("missing S3 Files reference")?;
        assert_eq!(files_edge.to_id, files);
        assert_eq!(
            files_edge.properties.get("target_account_id"),
            Some(&json!("222222222222"))
        );
        assert_eq!(
            files_edge.properties.get("target_region"),
            Some(&json!("us-west-2"))
        );
        assert_eq!(
            files_edge.properties.get("owner_resolution"),
            Some(&json!("aws_returned_arn"))
        );
        assert_eq!(resource(&body)?.properties.pointer("/volumes/2/fsx_windows_file_server_volume_configuration/authorization_config/credentials_parameter"), Some(&json!(SECRET_REF)));
        Ok(())
    }

    #[test]
    fn malformed_reference_arns_and_timestamp_overflow_are_rejected() -> TestResult {
        let (ctx, _events) = context(3)?;
        for role in [
            "web-role",
            "arn:aws:iam::111111111111:role/",
            "arn:aws:sts::111111111111:assumed-role/web/session",
        ] {
            let model = TaskDefinition::builder()
                .task_definition_arn(ARN)
                .family("web")
                .revision(7)
                .task_role_arn(role)
                .build();
            assert!(map_definition(&ctx, ARN, &output(model)).is_err());
        }
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .registered_at(DateTime::from_secs(i64::MAX))
            .build();
        assert!(map_definition(&ctx, ARN, &output(model)).is_err());
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .volumes(
                Volume::builder()
                    .s3files_volume_configuration(
                        S3FilesVolumeConfiguration::builder()
                            .file_system_arn(ROLE)
                            .build()?,
                    )
                    .build(),
            )
            .build();
        assert!(map_definition(&ctx, ARN, &output(model)).is_err());
        Ok(())
    }

    #[test]
    fn mapped_component_cap_prevents_unbounded_response_expansion() -> TestResult {
        let (ctx, _events) = context(3)?;
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .set_container_definitions(Some(vec![
                ContainerDefinition::builder().build();
                MAX_COMPONENTS.saturating_add(1)
            ]))
            .build();
        assert!(matches!(
            map_definition(&ctx, ARN, &output(model)),
            Err(AwsScanError::InvalidData(_))
        ));
        let environment =
            vec![KeyValuePair::builder().name("N").value("omitted").build(); MAX_COMPONENTS];
        let model = TaskDefinition::builder()
            .task_definition_arn(ARN)
            .family("web")
            .revision(7)
            .container_definitions(
                ContainerDefinition::builder()
                    .set_environment(Some(environment))
                    .build(),
            )
            .build();
        assert!(matches!(
            map_definition(&ctx, ARN, &output(model)),
            Err(AwsScanError::InvalidData(_))
        ));
        Ok(())
    }

    #[test]
    fn identity_ledger_rejects_duplicates_api_overflow_and_child_budget_before_work() -> TestResult
    {
        let seen = Mutex::new(ListedIdentities::default());
        assert!(remember_list_page(&seen, &[ARN.into(), ARN.into()], MAX_CHILDREN).is_err());
        remember_list_page(&seen, &[ARN.into()], MAX_CHILDREN)?;
        assert!(remember_list_page(&seen, &[ARN.into()], MAX_CHILDREN).is_err());
        assert!(matches!(
            remember_list_page(&seen, &[SECOND.into()], 0),
            Err(AwsScanError::ChildLimit)
        ));
        assert!(remember_list_page(&seen, &vec![SECOND.into(); 101], MAX_CHILDREN).is_err());
        let at_byte_limit = Mutex::new(ListedIdentities {
            arns: BTreeSet::new(),
            bytes: MAX_IDENTITY_BYTES,
        });
        assert!(remember_list_page(&at_byte_limit, &[ARN.into()], MAX_CHILDREN).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn empty_list_page_with_token_continues_to_a_revision_and_terminal_page() -> TestResult {
        let (ctx, mut receiver) = context(3)?;
        collect_with(
            &ctx,
            |token| async move {
                Ok(if token.is_none() {
                    ListTaskDefinitionsOutput::builder()
                        .next_token("next")
                        .build()
                } else {
                    ListTaskDefinitionsOutput::builder()
                        .task_definition_arns(ARN)
                        .build()
                })
            },
            |arn| async move { Ok(output(definition(&arn, 7))) },
        )
        .await?;
        let events = drain(&mut receiver)?;
        assert_eq!(events.len(), 3);
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.page_number == 1 && page.resources.is_empty() && page.next_token.as_deref() == Some("next"))));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.page_number == 2 && page.next_token.is_none())));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == DEFINITION_KIND && page.scope.parent_id.as_deref() == Some(ARN) && page.resources.len() == 1)));
        Ok(())
    }

    #[tokio::test]
    async fn denied_description_is_separate_from_successful_sibling_and_list_coverage() -> TestResult
    {
        let (ctx, mut receiver) = context(3)?;
        collect_with(
            &ctx,
            |_| async {
                Ok(ListTaskDefinitionsOutput::builder()
                    .task_definition_arns(ARN)
                    .task_definition_arns(SECOND)
                    .build())
            },
            |arn| async move {
                if arn == ARN {
                    Err(ApiError::Denied("fixture denied".into()))
                } else {
                    Ok(output(definition(&arn, 8)))
                }
            },
        )
        .await?;
        let events = drain(&mut receiver)?;
        assert_eq!(events.len(), 3);
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == DEFINITION_KIND && failure.scope.parent_id.as_deref() == Some(ARN) && failure.category == "access_denied")));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(SECOND) && page.resources.len() == 1)));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.is_none())));
        assert!(!events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(ARN))));
        Ok(())
    }

    #[tokio::test]
    async fn malformed_description_is_contained_to_its_revision_scope() -> TestResult {
        let (ctx, mut receiver) = context(3)?;
        collect_with(
            &ctx,
            |_| async {
                Ok(ListTaskDefinitionsOutput::builder()
                    .task_definition_arns(ARN)
                    .task_definition_arns(SECOND)
                    .build())
            },
            |arn| async move { Ok(output(definition(&arn, 8))) },
        )
        .await?;
        let events = drain(&mut receiver)?;
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.parent_id.as_deref() == Some(ARN) && failure.category == "invalid_response")));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.parent_id.as_deref() == Some(SECOND) && page.resources.len() == 1)));
        Ok(())
    }

    #[tokio::test]
    async fn denied_list_does_not_describe_or_claim_empty_success() -> TestResult {
        let (ctx, mut receiver) = context(3)?;
        let calls = AtomicUsize::new(0);
        collect_with(
            &ctx,
            |_| async { Err(ApiError::Denied("fixture list denied".into())) },
            |arn| {
                calls.fetch_add(1, Ordering::Relaxed);
                async move { Ok(output(definition(&arn, 7))) }
            },
        )
        .await?;
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        let events = drain(&mut receiver)?;
        assert_eq!(events.len(), 1);
        assert!(
            matches!(events.first(), Some(CollectionEvent::Failure(failure)) if failure.scope.collector == LIST_SCOPE && failure.category == "access_denied")
        );
        Ok(())
    }

    #[tokio::test]
    async fn page_cap_retains_token_and_explicit_incomplete_scope() -> TestResult {
        let (ctx, mut receiver) = context(1)?;
        collect_with(
            &ctx,
            |_| async {
                Ok(ListTaskDefinitionsOutput::builder()
                    .task_definition_arns(ARN)
                    .next_token("more")
                    .build())
            },
            |arn| async move { Ok(output(definition(&arn, 7))) },
        )
        .await?;
        let events = drain(&mut receiver)?;
        assert_eq!(events.len(), 3);
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == LIST_SCOPE && failure.category == "page_limit")));
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.as_deref() == Some("more"))));
        assert!(!events.iter().any(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == LIST_SCOPE && page.next_token.is_none())));
        Ok(())
    }

    #[tokio::test]
    async fn repeated_revision_across_pages_does_not_replay_description_scope() -> TestResult {
        let (ctx, mut receiver) = context(3)?;
        let calls = AtomicUsize::new(0);
        collect_with(
            &ctx,
            |token| async move {
                Ok(ListTaskDefinitionsOutput::builder()
                    .task_definition_arns(ARN)
                    .set_next_token(token.is_none().then(|| "next".into()))
                    .build())
            },
            |arn| {
                calls.fetch_add(1, Ordering::Relaxed);
                async move { Ok(output(definition(&arn, 7))) }
            },
        )
        .await?;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let events = drain(&mut receiver)?;
        assert!(events.iter().any(|event| matches!(event, CollectionEvent::Failure(failure) if failure.scope.collector == LIST_SCOPE && failure.category == "invalid_response")));
        assert_eq!(events.iter().filter(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == DEFINITION_KIND)).count(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn malformed_continuation_is_not_silently_declared_complete() -> TestResult {
        let (ctx, mut receiver) = context(3)?;
        collect_with(
            &ctx,
            |_| async { Ok(ListTaskDefinitionsOutput::builder().next_token("").build()) },
            |arn| async move { Ok(output(definition(&arn, 7))) },
        )
        .await?;
        let events = drain(&mut receiver)?;
        assert_eq!(events.len(), 1);
        assert!(
            matches!(events.first(), Some(CollectionEvent::Failure(failure)) if failure.category == "invalid_response")
        );
        Ok(())
    }
}
