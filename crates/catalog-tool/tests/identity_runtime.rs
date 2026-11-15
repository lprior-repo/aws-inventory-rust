#[path = "support/identity.rs"]
mod fixture;

use fixture::{TestResult, compiled, compiled_fixed};
use inventory_core::catalog::runtime::{BindingSource, BindingValue, IdentityPart, IdentityPlan, RuntimeCatalog};
use inventory_core::{inventory::ScalarType, record::FieldId};

#[test]
fn given_enum_identity_when_compiled_then_text_identity_paths_roundtrip() -> TestResult {
    let path = vec![FieldId::new(6)?].into_boxed_slice();
    [
        ("key=['Name']", IdentityPlan::Key(vec![path.clone()].into_boxed_slice())),
        ("arn='Name'", IdentityPlan::Arn(path.clone())),
        ("template='name/{Name}'", IdentityPlan::Template(vec![
            IdentityPart::Literal("name/".to_owned()), IdentityPart::Field(path),
        ].into_boxed_slice())),
    ].into_iter().try_for_each(|(identity, expected)| {
        let bytes = compiled(identity, "active-wire", false)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
        assert_eq!(plan.identity(), &expected);
        assert!(matches!(&plan.bindings().first().ok_or("fixed binding absent")?.source,
            BindingSource::Fixed(BindingValue::String(value)) if value == "active-wire"));
        Ok(())
    })
}

#[test]
fn given_enum_self_identity_when_compiled_then_selected_text_schema_roundtrips() -> TestResult {
    [
        ("key=['$']", IdentityPlan::Key(vec![Box::new([]) as Box<[FieldId]>].into_boxed_slice())),
        ("arn='$'", IdentityPlan::Arn(Box::new([]))),
        ("template='prefix/{$}'", IdentityPlan::Template(vec![
            IdentityPart::Literal("prefix/".to_owned()), IdentityPart::Field(Box::new([])),
        ].into_boxed_slice())),
    ].into_iter().try_for_each(|(identity, expected)| {
        let bytes = compiled(identity, "second-wire", true)?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
        assert_eq!(plan.identity(), &expected);
        let schema = catalog.schemas().iter().find(|schema| schema.kind().as_str() == "AWS::ApiOnly::Identity").ok_or("schema absent")?;
        let column = schema.columns().first().ok_or("scalar column absent")?;
        assert!(column.path().is_empty());
        assert_eq!(column.scalar(), ScalarType::Text);
        Ok(())
    })
}

#[test]
fn given_template_when_compiled_then_context_tags_and_field_order_survive_loader() -> TestResult {
    let bytes = compiled("template='arn:{partition}:s3:{region}:{account}:bucket/{Name}'", "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    assert_eq!(plan.identity(), &IdentityPlan::Template(vec![
        IdentityPart::Literal("arn:".to_owned()),
        IdentityPart::Partition,
        IdentityPart::Literal(":s3:".to_owned()),
        IdentityPart::Region,
        IdentityPart::Literal(":".to_owned()),
        IdentityPart::Account,
        IdentityPart::Literal(":bucket/".to_owned()),
        IdentityPart::Field(vec![FieldId::new(6)?].into_boxed_slice()),
    ].into_boxed_slice()));
    Ok(())
}

#[test]
fn given_unknown_enum_literal_when_checked_and_augmented_then_compilation_fails() {
    ["ACTIVE", "unknown", "active-wire "].into_iter().for_each(|literal| {
        assert!(compiled("key=['Name']", literal, false).is_err());
    });
}

#[test]
fn given_enum_list_filter_when_compiled_then_exact_wire_literals_are_retained() -> TestResult {
    let bytes = compiled_fixed("key=['Name']", "Mode='active-wire',Modes=['second-wire','active-wire']", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    assert!(plan.bindings().iter().any(|binding| matches!(&binding.source,
        BindingSource::Fixed(BindingValue::Strings(values)) if values.as_ref() == ["second-wire", "active-wire"])));
    assert!(compiled_fixed("key=['Name']", "Mode='active-wire',Modes=['second-wire','SECOND']", false).is_err());
    assert!(compiled_fixed("key=['Name']", "Mode='active-wire',Modes=['unknown']", false).is_err());
    Ok(())
}

#[test]
fn given_invalid_template_paths_when_checked_then_compile_denies_them() {
    ["template='{unknown}'", "template='{Name}{'", "template='{Items}'", "template='{$}'", "template='{account.bad}'"]
        .into_iter().for_each(|identity| assert!(compiled(identity, "active-wire", false).is_err()));
}

#[test]
fn given_singleton_structure_selection_when_compiled_then_identity_is_relative_to_item() -> TestResult {
    let bytes = fixture::compiled_singleton("key=['Name']")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    assert_eq!(plan.items(), &inventory_core::catalog::runtime::ItemPlan::Path(
        vec![FieldId::new(3)?].into_boxed_slice(),
    ));
    assert_eq!(plan.identity(), &IdentityPlan::Key(
        vec![vec![FieldId::new(6)?].into_boxed_slice()].into_boxed_slice(),
    ));
    Ok(())
}
