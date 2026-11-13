//! catalog has 397 reviewed regional read list types and safe columns behavior.
use super::*;

#[test]
fn catalog_has_397_reviewed_regional_read_list_types_and_safe_columns() -> TestResult {
    let catalog = root_catalog()?;
    assert_eq!(catalog.len(), 397);
    let names: BTreeSet<&str> = catalog
        .iter()
        .map(|entry| entry.type_name.as_str())
        .collect();
    assert_eq!(names.len(), 397);
    let services: BTreeSet<&str> = catalog.iter().map(|entry| entry.service.as_str()).collect();
    assert_eq!(services.len(), 76);
    for absent in [
        "AWS::ElasticLoadBalancingV2::Listener",
        "AWS::ElasticLoadBalancingV2::ListenerRule",
        "AWS::RDS::DBProxyTargetGroup",
        "AWS::EKS::FargateProfile",
        "AWS::IAM::Role",
        "AWS::S3::Bucket",
    ] {
        assert!(!names.contains(absent));
    }
    let reserved = [
        "id",
        "arn",
        "name",
        "region",
        "account_id",
        "kind",
        "source",
        "source_priority",
        "observed_at",
        "properties",
        "tags",
        "resource_json",
        "scan_id",
        "resource_identifier",
        "provider_properties",
        "cloudformation_type",
    ];
    for entry in &catalog {
        assert!(entry.read_supported && entry.list_supported && !entry.list_requires_model);
        assert_eq!(entry.scope, "regional");
        assert!(entry.columns.len() <= 200);
        assert!(
            entry
                .columns
                .iter()
                .all(|column| !reserved.contains(&column.name.as_str()))
        );
    }
    assert_eq!(
        descriptor("AWS::Logs::LogGroup")?.list_model_validation,
        "empty_object_accepted"
    );
    Ok(())
}

#[test]
fn selection_rejects_unknown_types_and_deduplicates_exact_names() -> TestResult {
    assert_eq!(validate_selection(&[])?.len(), 418);
    let selection =
        validate_selection(&["AWS::EC2::Instance".into(), "AWS::EC2::Instance".into()])?;
    assert_eq!(selection.len(), 1);
    assert!(validate_selection(&["AWS::IAM::Role".into()]).is_err());
    assert!(validate_selection(&vec!["AWS::EC2::Instance".into(); 419]).is_err());
    Ok(())
}

#[test]
fn canonical_identity_matches_native_and_returned_owner_is_preserved() -> TestResult {
    let (ctx, _events) = context(3)?;
    let ec2 = descriptor("AWS::EC2::Instance")?;
    let body = map_get(
        &ctx,
        &ec2,
        "i-abc",
        &get(description(
            "i-abc",
            json!({"InstanceId":"i-abc","VpcId":"vpc-a","SubnetId":"subnet-a","InstanceType":"t3.micro","UserData":"NEVER_STORE"}),
        )),
    )?;
    let resource = first(&body)?;
    assert_eq!(
        resource.id,
        "arn:aws:ec2:us-east-1:111111111111:instance/i-abc"
    );
    assert_eq!(resource.source, Source::CloudControl);
    assert_eq!(
        resource.properties.get("instance_type"),
        Some(&json!("t3.micro"))
    );
    assert_eq!(body.edges.len(), 2);
    assert!(!serde_json::to_string(resource)?.contains("NEVER_STORE"));
    let lambda = descriptor("AWS::Lambda::Function")?;
    let arn = "arn:aws:lambda:eu-west-1:222222222222:function:handler";
    let body = map_get(
        &ctx,
        &lambda,
        "handler",
        &get(description(
            "handler",
            json!({"FunctionName":"handler","Arn":arn}),
        )),
    )?;
    let resource = first(&body)?;
    assert_eq!(resource.id, arn);
    assert_eq!(resource.account_id, "222222222222");
    assert_eq!(resource.region, "eu-west-1");
    Ok(())
}

#[test]
fn composite_eip_identity_uses_allocation_property_and_exact_identifier_is_retained() -> TestResult
{
    let (ctx, _events) = context(3)?;
    let kind = descriptor("AWS::EC2::EIP")?;
    let id = "203.0.113.5|eipalloc-abcd";
    let body = map_get(
        &ctx,
        &kind,
        id,
        &get(description(
            id,
            json!({"PublicIp":"203.0.113.5","AllocationId":"eipalloc-abcd"}),
        )),
    )?;
    let resource = first(&body)?;
    assert_eq!(
        resource.id,
        "arn:aws:ec2:us-east-1:111111111111:elastic-ip/eipalloc-abcd"
    );
    assert_eq!(
        resource.properties.get("resource_identifier"),
        Some(&json!(id))
    );
    let empty = description(id, json!({}));
    let first_fallback = map_description(&ctx, &kind, id, &empty, false)?;
    let second_fallback = map_description(&ctx, &kind, id, &empty, false)?;
    assert_eq!(first(&first_fallback)?.id, first(&second_fallback)?.id);
    assert!(
        first(&first_fallback)?
            .id
            .starts_with("awscc:aws:us-east-1:111111111111:AWS::EC2::EIP:")
    );
    assert!(!first(&first_fallback)?.id.contains("203.0.113.5"));
    Ok(())
}

#[test]
fn fallback_identity_namespaces_identical_provider_ids_by_type() -> TestResult {
    let (ctx, _events) = context(3)?;
    let first_kind = descriptor("AWS::AppConfig::Application")?;
    let second_kind = descriptor("AWS::AppConfig::DeploymentStrategy")?;
    let value = description("same-id", json!({"Name":"same-name"}));
    let left = map_description(&ctx, &first_kind, "same-id", &value, false)?;
    let right = map_description(&ctx, &second_kind, "same-id", &value, false)?;
    assert_ne!(first(&left)?.id, first(&right)?.id);
    assert!(left.edges.is_empty() && right.edges.is_empty());
    Ok(())
}

#[test]
fn self_arn_is_used_instead_of_a_borrowed_role_or_secret_arn() -> TestResult {
    let (ctx, _events) = context(3)?;
    let kind = descriptor("AWS::Batch::JobDefinition")?;
    let arn = "arn:aws:batch:us-east-1:111111111111:job-definition/build:7";
    let body = map_description(
        &ctx,
        &kind,
        "build:7",
        &description(
            "build:7",
            json!({"JobDefinitionArn":arn,"RoleArn":"arn:aws:iam::111111111111:role/build"}),
        ),
        true,
    )?;
    assert_eq!(first(&body)?.id, arn);
    let redshift = descriptor("AWS::Redshift::Cluster")?;
    assert!(
        !redshift
            .arn_pointers
            .iter()
            .any(|value| value == "/ClusterNamespaceArn" || value == "/MasterPasswordSecretArn")
    );
    Ok(())
}

#[test]
fn wildcard_write_only_fields_and_common_nested_credentials_are_removed() -> TestResult {
    let kind = descriptor("AWS::EC2::SecurityGroup")?;
    let (safe, paths) = sanitized_properties(&kind, Some(&json!({"GroupId":"sg-a","SecurityGroupIngress":[{"SourceSecurityGroupName":"HIDDEN","IpProtocol":"tcp"},{"SourceSecurityGroupName":"HIDDEN_TWO","FromPort":443}],"Metadata":{"client_secret":"HIDDEN_THREE","Name":"visible"}}).to_string()))?;
    assert!(
        safe.pointer("/SecurityGroupIngress/0/SourceSecurityGroupName")
            .is_none()
    );
    assert_eq!(
        safe.pointer("/SecurityGroupIngress/0/IpProtocol"),
        Some(&json!("tcp"))
    );
    assert!(
        safe.pointer("/SecurityGroupIngress/1/SourceSecurityGroupName")
            .is_none()
    );
    assert!(safe.pointer("/Metadata/client_secret").is_none());
    assert_eq!(safe.pointer("/Metadata/Name"), Some(&json!("visible")));
    assert!(paths.contains(&"/SecurityGroupIngress/0/SourceSecurityGroupName".into()));
    assert!(!safe.to_string().contains("HIDDEN"));
    Ok(())
}
