use super::*;

pub(super) fn map_logs(
    config: &LogConfiguration,
    budget: &mut usize,
) -> Result<Value, AwsScanError> {
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

pub(super) fn map_container(
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

pub(super) fn map_volume(volume: &Volume, budget: &mut usize) -> Result<Value, AwsScanError> {
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
