use super::*;

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
    assert!(map_definition(&ctx, ARN, &DescribeTaskDefinitionOutput::builder().build()).is_err());
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
