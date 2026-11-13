//! volume references aggregate and preserve authoritative or assumed owners behavior.
use super::*;

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
fn identity_ledger_rejects_duplicates_api_overflow_and_child_budget_before_work() -> TestResult {
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
