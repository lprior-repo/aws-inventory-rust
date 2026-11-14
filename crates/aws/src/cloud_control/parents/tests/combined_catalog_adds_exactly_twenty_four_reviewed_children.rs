use super::*;

fn selection(names: &[&str]) -> Selection {
    Selection::Types(
        names
            .iter()
            .map(|name| inventory_core::ResourceType::new(*name))
            .collect(),
    )
}

#[test]
fn combined_catalog_adds_exactly_twenty_four_reviewed_children() -> TestResult {
    let types = crate::cloud_control::catalog()?;
    assert_eq!(types.len(), 421);
    assert_eq!(root_catalog()?.len(), 397);
    assert_eq!(parent_catalog()?.len(), 24);
    assert_eq!(
        types.iter().filter(|row| row.list_requires_model).count(),
        24
    );
    let groups = groups(&types)?;
    assert_eq!(groups.len(), 397);
    assert_eq!(
        groups
            .iter()
            .filter(|group| !group.children.is_empty())
            .count(),
        6
    );
    assert_eq!(
        groups
            .iter()
            .map(|group| group.children.len())
            .sum::<usize>(),
        24
    );
    assert!(
        validate_selection(&selection(&["AWS::ElasticLoadBalancingV2::ListenerRule"])).is_err()
    );
    assert!(validate_selection(&selection(&["AWS::ECS::TaskSet"])).is_err());
    Ok(())
}

#[test]
fn child_only_selection_adds_each_prerequisite_once_and_retains_explicit_roots() -> TestResult {
    let types = selected(&[
        "AWS::EKS::Addon",
        "AWS::EKS::Nodegroup",
        "AWS::EKS::Addon",
        "AWS::EKS::Cluster",
    ])?;
    let names: BTreeSet<_> = types.iter().map(|row| row.type_name.as_str()).collect();
    assert_eq!(
        names,
        BTreeSet::from([
            "AWS::EKS::Cluster",
            "AWS::EKS::Addon",
            "AWS::EKS::Nodegroup"
        ])
    );
    assert_eq!(one_group(&types)?.children.len(), 2);
    assert_eq!(selected(&["AWS::EKS::Cluster"])?.len(), 1);
    assert_eq!(
        one_group(&selected(&["AWS::EKS::Cluster"])?)?
            .children
            .len(),
        0
    );
    let children = parent_catalog()?;
    assert!(groups(&children).is_err());
    Ok(())
}

#[test]
fn every_reviewed_binding_builds_the_literal_expected_model() -> TestResult {
    let fixtures = [
        (
            "AWS::EKS::Cluster",
            json!({"Name":"alpha"}),
            json!({"ClusterName":"alpha"}),
            5,
        ),
        (
            "AWS::ElasticLoadBalancingV2::LoadBalancer",
            json!({"LoadBalancerArn":"arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/api/0123456789abcdef"}),
            json!({"LoadBalancerArn":"arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/api/0123456789abcdef"}),
            1,
        ),
        (
            "AWS::ApiGateway::RestApi",
            json!({"RestApiId":"api123"}),
            json!({"RestApiId":"api123"}),
            8,
        ),
        (
            "AWS::ApiGatewayV2::Api",
            json!({"ApiId":"http123"}),
            json!({"ApiId":"http123"}),
            5,
        ),
        (
            "AWS::Logs::LogGroup",
            json!({"LogGroupName":"/aws/lambda/api"}),
            json!({"LogGroupName":"/aws/lambda/api"}),
            2,
        ),
        (
            "AWS::AppConfig::Application",
            json!({"ApplicationId":"app123"}),
            json!({"ApplicationId":"app123"}),
            3,
        ),
    ];
    let all = crate::cloud_control::catalog()?;
    for (root, properties, expected, count) in fixtures {
        let group = groups(&all)?
            .into_iter()
            .find(|g| g.descriptor.type_name == root)
            .ok_or("missing expected root")?;
        assert_eq!(group.children.len(), count);
        let resource = fixture_resource(properties);
        for child in group.children {
            let context = parent_context(&child, &resource)?;
            let child_expected =
                if child.descriptor.type_name == "AWS::AppConfig::ExperimentDefinition" {
                    json!({"ApplicationIdentifier":"app123"})
                } else {
                    expected.clone()
                };
            assert_eq!(
                serde_json::from_str::<Value>(&context.encoded_model)?,
                child_expected
            );
            assert!(
                context
                    .scope_parent
                    .starts_with(&format!("{}:", child.descriptor.type_name))
            );
            assert_eq!(context.resource_id, resource.id);
            assert!(!context.encoded_model.contains("provider_properties"));
        }
    }
    Ok(())
}

#[test]
fn parent_context_rejects_missing_wrong_type_empty_control_and_oversized_values() -> TestResult {
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child = group.children.first().ok_or("missing child")?;
    for properties in [
        json!({}),
        json!({"Name":null}),
        json!({"Name":1}),
        json!({"Name":""}),
        json!({"Name":"bad\nname"}),
        json!({"Name":"x".repeat(1_025)}),
    ] {
        assert!(parent_context(child, &fixture_resource(properties)).is_err());
    }
    assert!(parent_context(child, &fixture_resource(json!({"Name":"x".repeat(1_024)}))).is_ok());
    Ok(())
}

#[test]
fn invalid_binding_cannot_reference_redacted_or_non_string_properties() -> TestResult {
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child = group.children.first().ok_or("missing child")?;
    let mut plan = child.plan.clone();
    for source in ["/DoesNotExist", "/Tags", "/Name/*"] {
        plan.bindings = vec![Binding {
            source_pointer: source.into(),
            model_property: "ClusterName".into(),
        }];
        assert!(validate_bindings(&plan, group.descriptor, child.descriptor).is_err());
    }
    for target in [
        "ConfigurationValues",
        "Tags",
        "Name/Injected",
        "",
        "Cluster_Name",
    ] {
        plan.bindings = vec![Binding {
            source_pointer: "/Name".into(),
            model_property: target.into(),
        }];
        assert!(validate_bindings(&plan, group.descriptor, child.descriptor).is_err());
    }
    Ok(())
}
