use super::*;

#[test]
fn skip_and_batch_cannot_target_a_deferred_operation() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = BASIC.replace(
        "\"n#ListThings\":{\"type\":\"operation\",\"output\":",
        "\"n#ListThings\":{\"type\":\"operation\",\"input\":{\"target\":\"n#Input\"},\"output\":",
    );
    model = model.replace(
        "\"n#Output\":",
        "\"n#ListOther\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#GetThing\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#Input\":{\"type\":\"structure\",\"members\":{\"Ids\":{\"target\":\"n#Ids\"}}},\"n#Ids\":{\"type\":\"list\",\"member\":{\"target\":\"smithy.api#String\"}},\"n#Output\":",
    );
    model = model.replace(
        "\"n#Service\":{\"type\":\"service\",\"operations\":[{\"target\":\"n#ListThings\"}]}",
        "\"n#Service\":{\"type\":\"service\",\"operations\":[{\"target\":\"n#ListThings\"},{\"target\":\"n#ListOther\"},{\"target\":\"n#GetThing\"}]}",
    );
    let root = fixture(&model, "")?;
    let text = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\nskip=['ListOther']\n[operation.ListThings.batch]\nmember='Ids'\nmax=10\nreplaces='ListOther'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n[operation.ListOther]\nclass='defer'\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
    for key in ["skip", "batch.replaces"] {
        assert!(
            errors.iter().any(|error| error.key() == key
                && error.value() == "ListOther"
                && error.rule() == ViolationRule::BindingSourceNotCallable),
            "{key}: {errors:?}"
        );
    }
    let valid = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\nskip=['GetThing']\n[operation.ListThings.batch]\nmember='Ids'\nmax=10\nreplaces='GetThing'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n[operation.GetThing]\nclass='expand'\nitems='Items'\n[operation.GetThing.paginator]\nkind='none'\n[operation.GetThing.identity]\nkey=['Id']\n[operation.ListOther]\nclass='defer'\n"
    );
    overlay(root.path(), &valid)?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    assert_eq!(
        checked
            .services()
            .first()
            .ok_or("missing service")?
            .callable()
            .len(),
        2
    );
    Ok(())
}

#[test]
fn incompatible_map_token_value_types_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Next":{"target":"n#StringMap"}}},
"n#Output":{"type":"structure","members":{"Next":{"target":"n#IntegerMap"},"Items":{"target":"n#Things"}}},
"n#StringMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#StringMapInner"}},
"n#IntegerMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#IntegerMapInner"}},
"n#StringMapInner":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#IntegerMapInner":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#Integer"}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;
    let root = fixture(model, "")?;
    let text = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='token'\ninput='Next'\noutput='Next'\n[operation.ListThings.identity]\nkey=['Id']\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
    assert!(
        errors.iter().any(|error| error.key() == "paginator.token"
            && error.value() == "Next"
            && error.rule() == ViolationRule::InvalidValue),
        "{errors:?}"
    );
    Ok(())
}

#[test]
fn item_metadata_paths_must_resolve_on_selected_items() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture(BASIC, "")?;
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\nexclude_from_hash=['Absent']\nqueryable=['Absent']\ntags='Absent'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Absent']\n"
        ),
    )?;
    let errors = violations(root.path())?;
    let actual: Vec<_> = errors
        .iter()
        .map(|error| (error.key(), error.value(), error.rule()))
        .collect();
    assert_eq!(
        actual,
        [
            ("exclude_from_hash", "Absent", ViolationRule::InvalidPath),
            ("identity", "Absent", ViolationRule::InvalidPath),
            ("queryable", "Absent", ViolationRule::InvalidPath),
            ("tags", "Absent", ViolationRule::InvalidPath),
        ]
    );
    Ok(())
}

#[test]
fn each_missing_plan_component_reports_its_exact_incomplete_plan()
-> Result<(), Box<dyn std::error::Error>> {
    for (text, key, value) in [
        (
            format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n"
            ),
            "items",
            "missing",
        ),
        (
            format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.identity]\nkey=['Id']\n"
            ),
            "paginator",
            "missing explicit none",
        ),
        (
            format!(
                "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='none'\n"
            ),
            "identity",
            "missing",
        ),
    ] {
        let root = fixture(BASIC, "")?;
        overlay(root.path(), &text)?;
        let errors = violations(root.path())?;
        assert!(
            errors.iter().any(|error| error.key() == key
                && error.value() == value
                && error.rule() == ViolationRule::IncompletePlan),
            "{key}: {errors:?}"
        );
    }
    Ok(())
}

#[test]
fn identity_template_skips_known_placeholders_and_checks_later_item_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture(BASIC, "")?;
    let prefix = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\n"
    );
    overlay(
        root.path(),
        &format!("{prefix}template='{{region}}/{{account}}/{{partition}}'\n"),
    )?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    assert_eq!(
        checked
            .services()
            .first()
            .ok_or("missing service")?
            .callable()
            .len(),
        1
    );
    overlay(
        root.path(),
        &format!("{prefix}template='{{region}}/{{Nope}}'\n"),
    )?;
    let errors = violations(root.path())?;
    assert_eq!(
        errors
            .iter()
            .map(|error| (error.key(), error.value(), error.rule()))
            .collect::<Vec<_>>(),
        [("identity", "Nope", ViolationRule::InvalidPath)]
    );
    Ok(())
}

#[test]
fn callable_plan_materializes_model_derived_items_and_paginator()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","operations":[{"target":"n#ListThings"}]},
"n#ListThings":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"},"traits":{"smithy.api#paginated":{"inputToken":"NextToken","outputToken":"NextToken","items":"Items"}}},
"n#Input":{"type":"structure","members":{"NextToken":{"target":"smithy.api#String"}}},
"n#Output":{"type":"structure","members":{"NextToken":{"target":"smithy.api#String"},"Items":{"target":"n#Things"}}},
"n#Things":{"type":"list","member":{"target":"n#Thing"}},
"n#Thing":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}}}"#;
    let root = fixture(model, "")?;
    overlay(
        root.path(),
        &format!(
            "{HEADER}[operation.ListThings]\nclass='enumerate'\n[operation.ListThings.identity]\nkey=['Id']\n"
        ),
    )?;
    let checked = overlay::check(root.path(), &source(root.path()))?;
    let callable = checked
        .services()
        .first()
        .ok_or("missing service")?
        .callable()
        .first()
        .ok_or("missing callable")?;
    assert_eq!(
        callable.plan().items,
        Some(ItemSelector::Member(overlay::MemberPath::new("Items")?))
    );
    let Some(overlay::Paginator::Token(pair)) = &callable.plan().paginator else {
        return Err("missing model-derived paginator".into());
    };
    assert_eq!(pair.input, overlay::MemberPath::new("NextToken")?);
    assert_eq!(pair.output, overlay::MemberPath::new("NextToken")?);
    assert_eq!(
        checked.report(),
        "totals\tservices_with_overlay=1\tcallable_enumerate=1\tcallable_expand=0\tdenied_by_overlay=0\tdeferred=0\tservices_without_overlay=0\tunreviewed_enumerate=0\tunreviewed_expand=0\tunreviewed_unclassified=0\noperation\tthings\tListThings\tenumerate\tclass=overlay,items=model,page_size=missing,paginator=model,identity=overlay,bindings=missing,scope=missing,batch=missing\n"
    );
    Ok(())
}
#[test]
fn references_reject_unknown_and_deferred_operations() -> Result<(), Box<dyn std::error::Error>> {
    let model = BASIC
        .replace(
            "\"n#ListThings\"}]",
            "\"n#ListThings\"},{\"target\":\"n#ListOther\"},{\"target\":\"n#GetThing\"}]",
        )
        .replace(
            "\"n#ListThings\":{\"type\":\"operation\",\"output\":",
            "\"n#GetThing\":{\"type\":\"operation\",\"input\":{\"target\":\"n#GetInput\"},\"output\":{\"target\":\"n#Output\"}},\"n#GetInput\":{\"type\":\"structure\",\"members\":{\"Id\":{\"target\":\"smithy.api#String\",\"traits\":{\"smithy.api#required\":{}}}}},\"n#ListThings\":{\"type\":\"operation\",\"input\":{\"target\":\"n#Input\"},\"output\":",
        )
        .replace(
            "\"n#Output\":",
            "\"n#ListOther\":{\"type\":\"operation\",\"output\":{\"target\":\"n#Output\"}},\"n#Input\":{\"type\":\"structure\",\"members\":{\"Ids\":{\"target\":\"n#Ids\"}}},\"n#Ids\":{\"type\":\"list\",\"member\":{\"target\":\"smithy.api#String\"}},\"n#Output\":",
        );
    let root = fixture(&model, "")?;
    let text = format!(
        "{HEADER}[operation.ListThings]\nclass='enumerate'\nitems='Items'\nskip=['MissingSkip']\n[operation.ListThings.batch]\nmember='Ids'\nmax=10\nreplaces='MissingBatch'\n[operation.ListThings.paginator]\nkind='none'\n[operation.ListThings.identity]\nkey=['Id']\n[operation.ListOther]\nclass='defer'\n[operation.GetThing]\nclass='expand'\nitems='Items'\n[operation.GetThing.bind]\nId={{from='ListOther',path='Id'}}\n[operation.GetThing.paginator]\nkind='none'\n[operation.GetThing.identity]\nkey=['Id']\n"
    );
    overlay(root.path(), &text)?;
    let errors = violations(root.path())?;
    for (key, target) in [("skip", "MissingSkip"), ("batch.replaces", "MissingBatch")] {
        assert!(
            errors.iter().any(|error| error.key() == key
                && error.value() == target
                && error.rule() == ViolationRule::UnknownOperation),
            "{key}: {errors:?}"
        );
    }
    assert!(errors.iter().any(|error| error.key() == "bind.from"
        && error.value() == "ListOther"
        && error.rule() == ViolationRule::BindingSourceNotCallable));
    Ok(())
}
