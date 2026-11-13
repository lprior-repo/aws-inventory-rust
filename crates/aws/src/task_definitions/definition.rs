//! definition operations.
use super::*;

pub(super) fn map_definition(
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
