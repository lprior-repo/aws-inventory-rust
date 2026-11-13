//! environment and unstructured option values are not retained behavior.
use super::*;

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
