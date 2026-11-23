use super::*;

#[tokio::test(flavor = "current_thread")]
async fn given_list_and_singular_structure_when_compiled_then_one_canonical_identity_and_sqlite_schema_are_consumable()
-> TestResult {
    for root_detail in [false, true] {
        let detail_items = if root_detail { "$" } else { "Configuration" };
        let review = reviewed_overlay(
            arn("FunctionArn")?,
            arn("FunctionArn")?,
            "FunctionArn",
            detail_items,
            &[],
        )?;
        let root = fixture(
            &model(
                "n#FunctionConfiguration",
                "n#FunctionConfiguration",
                root_detail,
            ),
            &review,
        )?;

        let bytes = compile(&root, Some("FunctionName"))??;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let fields = smithy::Registry::parse(FIELDS)?;
        let arn = fields
            .id("FunctionArn")
            .ok_or("FunctionArn registry entry missing")?;
        let name = fields
            .id("FunctionName")
            .ok_or("FunctionName registry entry missing")?;
        let functions = fields
            .id("Functions")
            .ok_or("Functions registry entry missing")?;
        let configuration = fields
            .id("Configuration")
            .ok_or("Configuration registry entry missing")?;
        let list = catalog
            .lookup("lambda", "ListFunctions")
            .ok_or("list plan missing")?;
        let detail = catalog
            .lookup("lambda", "GetFunctionConfiguration")
            .ok_or("detail plan missing")?;

        assert_eq!(catalog.plans().len(), 2);
        assert_eq!((list.kind().as_str(), detail.kind().as_str()), (KIND, KIND));
        assert_eq!(
            list.identity(),
            &IdentityPlan::Arn(vec![arn].into_boxed_slice())
        );
        assert_eq!(detail.identity(), list.identity());
        assert_eq!(
            list.items(),
            &ItemPlan::Path(vec![functions].into_boxed_slice())
        );
        let expected_detail = if root_detail {
            ItemPlan::Root
        } else {
            ItemPlan::Path(vec![configuration].into_boxed_slice())
        };
        assert_eq!(detail.items(), &expected_detail);
        let binding = detail.bindings().first().ok_or("detail binding missing")?;
        assert_eq!(detail.bindings().len(), 1);
        assert!(
            matches!(&binding.source, BindingSource::Operation { service, operation, path }
            if service.as_ref() == "lambda" && operation.as_ref() == "ListFunctions" && path.as_ref() == [arn])
        );
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog
            .schemas()
            .first()
            .ok_or("canonical schema missing")?;
        assert_eq!(
            (schema.kind().as_str(), schema.table().as_str()),
            (KIND, TABLE)
        );
        let columns = schema
            .columns()
            .iter()
            .map(|column| (column.path(), column.scalar()))
            .collect::<Vec<_>>();
        assert_eq!(
            columns,
            vec![(std::slice::from_ref(&name), ScalarType::Text)]
        );

        assert_canonical_sqlite(&root.path().join("canonical.sqlite"), &bytes, &catalog).await?;
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_compatible_scalar_views_of_one_kind_when_compiled_then_one_selected_scalar_type_is_retained()
-> TestResult {
    for (list_target, detail_target, scalar) in [
        ("smithy.api#String", "n#TextItems", ScalarType::Text),
        ("smithy.api#Long", "n#IntegerItems", ScalarType::Integer),
    ] {
        let review = reviewed_overlay(key("$")?, key("$")?, "$", "Configuration", &[])?;
        let root = fixture(&model(list_target, detail_target, false), &review)?;

        let bytes = compile(&root, None)??;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;

        assert_eq!(catalog.plans().len(), 2);
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog
            .schemas()
            .first()
            .ok_or("canonical scalar schema missing")?;
        assert_eq!(schema.kind().as_str(), KIND);
        let selected = schema
            .columns()
            .iter()
            .filter(|column| column.path().is_empty())
            .map(|column| (column.name(), column.scalar()))
            .collect::<Vec<_>>();
        assert_eq!(selected, vec![("value", scalar)]);
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_incompatible_scalar_views_of_one_kind_when_compiled_then_capability_error_rejects_schema_merge()
-> TestResult {
    for (list_target, detail_target) in [
        ("smithy.api#String", "n#IntegerItems"),
        ("smithy.api#Long", "n#TextItems"),
    ] {
        let review = reviewed_overlay(key("$")?, key("$")?, "$", "Configuration", &[])?;
        let root = fixture(&model(list_target, detail_target, false), &review)?;

        let result = compile(&root, None)?;

        let error = match result {
            Err(error) => error,
            Ok(_) => return Err("incompatible scalar views compiled successfully".into()),
        };
        assert!(
            matches!(
                &error,
                runtime_catalog::RuntimeCatalogCompileError::KindContext { kind, path, reason }
                    if kind == KIND && path == "column"
                        && *reason == "operations of this kind disagree on the item type"
            ),
            "{error}"
        );
        root.close()?;
    }
    Ok(())
}

#[test]
fn given_unknown_enum_in_fixed_list_when_reviewed_then_precompile_gate_reports_exact_invalid_value()
-> TestResult {
    for runtimes in [["provided.al2023", "NODE"], ["unknown", "nodejs22.x"]] {
        let review = reviewed_overlay(
            arn("FunctionArn")?,
            arn("FunctionArn")?,
            "FunctionArn",
            "Configuration",
            &runtimes,
        )?;
        let root = fixture(
            &model("n#FunctionConfiguration", "n#FunctionConfiguration", false),
            &review,
        )?;

        let violations = match root.check().map_err(|error| *error) {
            Err(overlay::CheckError::Violations(violations)) => violations,
            Err(other) => return Err(other.into()),
            Ok(_) => return Err("unknown enum fixed-list value passed precompile gate".into()),
        };

        let diagnostics = violations
            .iter()
            .map(|violation| {
                (
                    violation.service(),
                    violation.operation(),
                    violation.key(),
                    violation.value(),
                    violation.rule(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            diagnostics,
            vec![
                (
                    "lambda",
                    "GetFunctionConfiguration",
                    "bind.from",
                    "ListFunctions",
                    overlay::ViolationRule::BindingSourceNotCallable
                ),
                (
                    "lambda",
                    "ListFunctions",
                    "fixed",
                    "Runtimes",
                    overlay::ViolationRule::InvalidValue
                ),
            ]
        );
        root.close()?;
    }
    Ok(())
}
