use super::*;

fn options() -> PlanArgs {
    PlanArgs {
        profile: vec!["one".into(), "two".into()],
        region: vec!["us-east-1".into(), "us-west-2".into()],
        service: vec!["all".into()],
        cloud_control: true,
        resource_type: Vec::new(),
        config_aggregator: None,
        resource_explorer_view: None,
        discover_sources: false,
        accounts: crate::cli::AccountArgs::default(),
    }
}

#[test]
fn full_plan_has_schema_backed_breadth_and_explicit_cost_uncertainty() -> Result<()> {
    let plan = create(options())?;
    assert_eq!(
        plan.pointer("/coverage/direct_resource_kinds")
            .and_then(Value::as_u64),
        Some(32)
    );
    let count = plan
        .pointer("/coverage/cloud_control_resource_types")
        .and_then(Value::as_u64)
        .context("missing count")?;
    assert_eq!(count, 418);
    assert_eq!(
        plan.pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"),
        Some(&Value::Null)
    );
    assert_eq!(
        plan.pointer("/request_accounting/first_root_list_requests_if_all_scopes_reached"),
        Some(&json!(1588))
    );
    assert_eq!(
        plan.pointer("/request_accounting/total_requests"),
        Some(&Value::Null)
    );
    assert_eq!(
        plan.pointer("/coverage/live_validated"),
        Some(&Value::Bool(false))
    );
    Ok(())
}

#[test]
fn selected_children_share_one_explicit_parent_prerequisite() -> Result<()> {
    let mut selected = options();
    selected.resource_type = vec![
        "AWS::EKS::Addon".into(),
        "AWS::EKS::Nodegroup".into(),
        "AWS::EKS::Addon".into(),
    ];
    let plan = create(selected)?;
    assert_eq!(
        plan.pointer("/coverage/cloud_control_resource_types"),
        Some(&json!(3))
    );
    assert_eq!(
        plan.pointer("/coverage/cloud_control_root_types"),
        Some(&json!(1))
    );
    assert_eq!(
        plan.pointer("/coverage/cloud_control_parent_dependent_types"),
        Some(&json!(2))
    );
    assert_eq!(
        plan.pointer("/coverage/implicit_parent_types"),
        Some(&json!(["AWS::EKS::Cluster"]))
    );
    assert_eq!(
        plan.pointer("/request_accounting/first_root_list_requests_if_all_scopes_reached"),
        Some(&json!(4))
    );
    assert_eq!(
        plan.pointer("/request_accounting/child_list_requests"),
        Some(&Value::Null)
    );
    let addon = array(&plan, "cloud_control")?
        .iter()
        .find(|row| row.get("type_name") == Some(&json!("AWS::EKS::Addon")))
        .context("addon plan")?;
    assert_eq!(
        addon.pointer("/discovery/resource_model_required"),
        Some(&json!(true))
    );
    assert_eq!(
        addon.pointer("/parent_workflow/bindings"),
        Some(&json!([{"source_pointer":"/Name","model_property":"ClusterName"}]))
    );
    Ok(())
}

#[test]
fn root_only_request_count_keeps_the_old_exact_contract() -> Result<()> {
    let mut selected = options();
    selected.resource_type = vec!["AWS::EKS::Cluster".into()];
    let plan = create(selected)?;
    assert_eq!(
        plan.pointer("/coverage/implicit_parent_types"),
        Some(&json!([]))
    );
    assert_eq!(
        plan.pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"),
        Some(&json!(4))
    );
    assert_eq!(
        plan.pointer("/request_accounting/child_list_requests"),
        Some(&json!(0))
    );
    Ok(())
}

#[test]
fn selectors_deduplicate_and_unknown_types_are_errors() -> Result<()> {
    let mut selected = options();
    selected.service = vec!["ec2".into(), "network".into(), "vpc".into()];
    selected.cloud_control = false;
    assert_eq!(
        create(selected)?
            .pointer("/coverage/direct_resource_kinds")
            .and_then(Value::as_u64),
        Some(12)
    );
    let mut invalid = options();
    invalid.resource_type = vec!["AWS::Invented::Missing".into()];
    assert!(create(invalid).is_err());
    let mut invalid = options();
    invalid.service = vec!["all".into(), "unknown".into()];
    assert!(create(invalid).is_err());
    Ok(())
}

#[test]
fn supplemental_only_and_unknown_region_count_remain_explicit() -> Result<()> {
    let mut selected = options();
    selected.service = vec!["config".into()];
    selected.config_aggregator = Some("existing".into());
    selected.cloud_control = false;
    let plan = create(selected)?;
    assert_eq!(
        plan.pointer("/coverage/direct_resource_kinds")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(array(&plan, "supplemental")?.len(), 1);
    let mut dynamic = options();
    dynamic.region.clear();
    assert_eq!(
        create(dynamic)?
            .pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"),
        Some(&Value::Null)
    );
    Ok(())
}

#[test]
fn global_and_organization_plans_keep_account_count_unknown_until_discovery() -> Result<()> {
    let mut selected = options();
    selected.profile = vec!["management".into()];
    selected.service = vec![
        "s3".into(),
        "iam".into(),
        "route53".into(),
        "cloudfront".into(),
    ];
    selected.discover_sources = true;
    selected.accounts.organization_role = Some("audit/InventoryReader".into());
    selected.accounts.external_id_env = Some("NOT_LOADED_DURING_PLAN".into());
    let plan = create(selected)?;
    assert_eq!(
        plan.pointer("/coverage/direct_resource_kinds"),
        Some(&json!(7))
    );
    assert_eq!(
        plan.pointer("/targets/resolved_target_count"),
        Some(&Value::Null)
    );
    assert_eq!(
        plan.pointer("/request_accounting/first_cloud_control_list_requests_if_all_scopes_reached"),
        Some(&Value::Null)
    );
    assert_eq!(
        plan.pointer("/supplemental/0/creates_infrastructure"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn source_discovery_allows_source_selectors_without_loading_credentials() -> Result<()> {
    let mut selected = options();
    selected.service = vec!["config".into(), "resource-explorer".into()];
    selected.cloud_control = false;
    selected.discover_sources = true;
    let plan = create(selected)?;
    assert_eq!(
        plan.pointer("/coverage/direct_resource_kinds"),
        Some(&json!(0))
    );
    assert_eq!(array(&plan, "supplemental")?.len(), 1);
    Ok(())
}
