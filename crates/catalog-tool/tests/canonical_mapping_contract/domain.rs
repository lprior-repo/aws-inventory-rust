use super::*;

#[test]
fn given_one_kind_with_key_and_arn_views_when_compiled_then_identity_domain_split_is_rejected() -> TestResult {
    let review = reviewed_overlay(key("FunctionName")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
    let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", false), &review)?;
    assert!(matches!(compile(&root, Some("FunctionName"))?, Err(runtime_catalog::RuntimeCatalogCompileError::Capability)));
    root.close()?;
    Ok(())
}

#[test]
fn given_template_and_direct_arn_views_when_compiled_then_the_shared_arn_domain_is_consumable() -> TestResult {
    let template = overlay::Identity::Template("arn:{partition}:lambda:{region}:{account}:function:{FunctionName}".to_owned());
    let review = reviewed_overlay(template, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
    let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", false), &review)?;
    let bytes = compile(&root, Some("FunctionName"))??;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.plans().len(), 2);
    assert!(matches!(catalog.lookup("lambda", "ListFunctions").ok_or("list plan absent")?.identity(), IdentityPlan::Template(_)));
    assert!(matches!(catalog.lookup("lambda", "GetFunctionConfiguration").ok_or("detail plan absent")?.identity(), IdentityPlan::Arn(_)));
    root.close()?;
    Ok(())
}

#[test]
fn given_present_incompatible_display_path_in_one_view_when_compiled_then_it_is_not_treated_as_absent() -> TestResult {
    for conflicting_list in [false, true] {
        let mut source = model("n#FunctionConfiguration", "n#DetailConfiguration", false);
        let shapes = source.get_mut("shapes").and_then(serde_json::Value::as_object_mut).ok_or("shapes absent")?;
        let detail_name = if conflicting_list { "smithy.api#String" } else { "n#Runtimes" };
        shapes.insert("n#DetailConfiguration".to_owned(), json!({"type":"structure","members":{
            "FunctionArn":{"target":"smithy.api#String"},"FunctionName":{"target":detail_name}}}));
        if conflicting_list {
            let members = shapes.get_mut("n#FunctionConfiguration").and_then(|shape| shape.get_mut("members"))
                .and_then(serde_json::Value::as_object_mut).ok_or("list members absent")?;
            members.insert("FunctionName".to_owned(), json!({"target":"n#Runtimes"}));
        }
        let review = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
        let root = fixture(&source, &review)?;
        assert!(matches!(compile(&root, Some("FunctionName"))?, Err(runtime_catalog::RuntimeCatalogCompileError::Capability)));
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_display_path_absent_from_sparse_detail_when_compiled_then_other_view_supplies_it() -> TestResult {
    let mut source = model("n#FunctionConfiguration", "n#DetailConfiguration", false);
    source.get_mut("shapes").and_then(serde_json::Value::as_object_mut).ok_or("shapes absent")?
        .insert("n#DetailConfiguration".to_owned(), json!({"type":"structure","members":{"FunctionArn":{"target":"smithy.api#String"}}}));
    let review = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
    let root = fixture(&source, &review)?;
    let bytes = compile(&root, Some("FunctionName"))??;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.name_paths().len(), 1);
    assert_eq!(catalog.plans().len(), 2);
    root.close()?;
    Ok(())
}

#[test]
fn given_fixed_and_derived_value_for_one_input_when_reviewed_then_overwrite_is_rejected() -> TestResult {
    let reviewed = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
    let operations = reviewed.operations().iter().map(|(name, original)| {
        let mut plan = original.clone();
        if name.as_str() == "GetFunctionConfiguration" {
            plan.fixed.insert(overlay::MemberPath::new("FunctionName")?,
                overlay::FixedValue::String("hardcoded".to_owned()));
        }
        Ok((name.as_str(), plan))
    }).collect::<TestResult<Vec<_>>>()?;
    let review = overlay::ServiceOverlay::new("lambda", operations)?;
    let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", false), &review)?;
    let violations = match root.check() {
        Err(overlay::CheckError::Violations(violations)) => violations,
        Err(other) => return Err(other.into()),
        Ok(_) => return Err("conflicting input writers were admitted".into()),
    };
    assert!(violations.iter().any(|violation| violation.operation() == "GetFunctionConfiguration"
        && violation.key() == "input" && violation.value() == "FunctionName"
        && violation.rule() == overlay::ViolationRule::InvalidPath));
    root.close()?;
    Ok(())
}

#[test]
fn given_inactive_mapped_service_when_compiling_subset_then_coverage_remains_without_model_lookups() -> TestResult {
    let review = reviewed_overlay(arn("FunctionArn")?, arn("FunctionArn")?, "FunctionArn", "Configuration", &[])?;
    let root = fixture(&model("n#FunctionConfiguration", "n#FunctionConfiguration", false), &review)?;
    let bytes = compile_mapping(&root, Some("FunctionName"), true)??;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    assert_eq!(catalog.plans().len(), 2);
    assert_eq!(catalog.coverage().len(), 2);
    let inactive = catalog.schemas().iter().find(|schema| schema.kind().as_str() == "AWS::EC2::VPC").ok_or("inactive coverage schema missing")?;
    assert!(inactive.columns().is_empty());
    assert!(catalog.coverage().iter().any(|entry| entry.mapped
        && entry.kind.as_ref().is_some_and(|kind| kind.as_str() == "AWS::EC2::VPC")));
    assert_eq!(catalog.name_paths().len(), 1);
    assert_eq!(catalog.name_paths().first().ok_or("active display path missing")?.kind.as_str(), KIND);
    root.close()?;
    Ok(())
}
